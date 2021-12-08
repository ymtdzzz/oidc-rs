use std::convert::TryInto;

use chrono::{Duration, Utc};
use diesel::MysqlConnection;
use rocket::{
    figment::{
        util::map,
        value::{Map, Value},
    },
    form::Form,
    http::CookieJar,
    serde::json::Json,
};
use rocket_dyn_templates::Template;

use crate::{
    error::CustomError,
    internal::{
        authentication::AuthenticationRequest,
        token::{IdToken, SuccessfulTokenResponse},
        userinfo::{SuccessfulUserinfoResponse, UserinfoRequest},
    },
    models::{AuthChallenge, AuthCode, Client, NewToken, Session},
    repository::{
        self, create_auth_code, create_client, create_session, find_auth_challenge, find_session,
    },
    utils::generate_challenge,
};

use self::{
    context::{ConsentContext, ErrorContext, LoginContext},
    request::{
        AuthenticationParams, ClientParams, ConsentGetParams, ConsentParams, LoginParams,
        TokenParams,
    },
    response::RedirectWithCookie,
};

pub mod context;
mod request;
mod response;

#[database("oidc_db")]
pub struct DBPool(MysqlConnection);

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[get("/client?<clientparam..>")]
async fn get_client(
    clientparam: Option<ClientParams>,
    conn: DBPool,
) -> Result<String, CustomError> {
    conn.run(move |c| match clientparam {
        Some(param) => {
            let client_id = generate_challenge();
            let client_secret = generate_challenge();
            create_client(
                Client {
                    client_id: client_id.clone(),
                    client_secret: client_secret.clone(),
                    scope: param.scope,
                    response_type: param.response_type,
                    redirect_uri: param.redirect_uri,
                },
                c,
            )?;
            Ok(format!(
                "client_id: {}, client_secret: {}",
                client_id, client_secret
            ))
        }
        None => Err(CustomError::BadRequest),
    })
    .await
}

#[get("/authenticate?<authparam..>")]
async fn get_authenticate(
    authparam: Option<AuthenticationParams>,
    conn: DBPool,
) -> Result<Template, CustomError> {
    match authparam {
        Some(param) => {
            conn.run(move |c| {
                let client = repository::find_client(&param.client_id, c)?;
                let auth_req = AuthenticationRequest::new(
                    &param.scope,
                    &param.response_type,
                    &param.client_id,
                    &param.redirect_uri,
                    param.state,
                    Some(&client.try_into()?),
                ).map_err(|_e| CustomError::BadRequest)?;
                let challenge = generate_challenge();
                repository::create_auth_challenge(
                    AuthChallenge::from_auth_request(&challenge, auth_req).expect("failed to convert from the AuthenticationRequest into the model AuthChallenge"),
                    c,
                )?;
                Ok(Template::render(
                    "login",
                    &LoginContext {
                        error_msg: None,
                        login_challenge: challenge,
                    },
                ))
            })
            .await
        }
        None => Err(CustomError::BadRequest),
    }
}

#[post("/authenticate", data = "<loginparam>")]
async fn post_authenticate(
    loginparam: Form<LoginParams>,
    conn: DBPool,
) -> Result<RedirectWithCookie, CustomError> {
    conn.run(move |c| {
        if find_auth_challenge(&loginparam.login_challenge, c).is_err() {
            return Err(CustomError::ValidationError(Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from("Challenge is incorrect."),
                },
            )));
        }
        if loginparam.username == "foobar".to_string() && loginparam.password == "1234" {
            let session_id = generate_challenge();
            create_session(
                Session {
                    session_id: session_id.clone(),
                },
                c,
            )?;
            Ok(RedirectWithCookie {
                key: String::from("session_id"),
                value: session_id,
                next: format!(
                    "/authorization?consent_challenge={}",
                    &loginparam.login_challenge
                ),
            })
        } else {
            Err(CustomError::ValidationError(Template::render(
                "login",
                &LoginContext {
                    error_msg: Some(String::from("username or password is incorrect")),
                    login_challenge: loginparam.login_challenge.to_string(),
                },
            )))
        }
    })
    .await
}

#[get("/authorization?<consentgetparam..>")]
async fn get_authorization<'a>(
    consentgetparam: Option<ConsentGetParams>,
    jar: &'a CookieJar<'_>,
    conn: DBPool,
) -> Result<Template, CustomError> {
    let mut session_id: Option<String> = None;
    if let Some(session) = jar.get("session_id") {
        session_id = Some(session.value().to_string());
    }
    conn.run(move |c| {
        let mut is_session_error = false;
        // login check
        if let Some(id) = session_id {
            if find_session(&id, c).is_err() {
                is_session_error = true;
            }
        } else {
            is_session_error = true;
        }
        if is_session_error {
            return Err(CustomError::SessionError);
        }
        // challenge check
        match consentgetparam {
            Some(param) => {
                if find_auth_challenge(&param.consent_challenge, c).is_err() {
                    return Err(CustomError::ChallengeError);
                }
                Ok(Template::render(
                    "consent",
                    &ConsentContext {
                        consent_challenge: param.consent_challenge,
                    },
                ))
            }
            None => Err(CustomError::BadRequest),
        }
    })
    .await
}

#[post("/authorization", data = "<consentparam>")]
async fn post_authorization<'a>(
    consentparam: Form<ConsentParams>,
    jar: &'a CookieJar<'_>,
    conn: DBPool,
) -> Result<String, CustomError> {
    let mut session_id: Option<String> = None;
    if let Some(session) = jar.get("session_id") {
        session_id = Some(session.value().to_string());
    }
    conn.run(move |c| {
        let mut is_session_error = false;
        // login check
        if let Some(id) = session_id {
            if find_session(&id, c).is_err() {
                is_session_error = false;
            }
        } else {
            is_session_error = false;
        }
        if is_session_error {
            return Err(CustomError::SessionError);
        }
        // challenge check
        let challenge = find_auth_challenge(&consentparam.consent_challenge, c);
        if challenge.is_err() {
            return Err(CustomError::ChallengeError);
        }
        let challenge = challenge.unwrap();
        let auth_code = generate_challenge();
        create_auth_code(AuthCode {
            code: auth_code.clone(),
            client_id: challenge.client_id.clone(),
            user_id: String::from("userid"), // dummy user id
            scope: challenge.scope.clone(),
        }, c)?;
        Ok(format!(
            "consent_challenge: {}, consent: {}, auth_code: {} // TODO: redirect to RP callback including authorization code",
            consentparam.consent_challenge, consentparam.consent, auth_code
        ))
    }).await
}

#[post("/token", data = "<tokenparam>")]
async fn post_token(
    tokenparam: Form<TokenParams>,
    conn: DBPool,
) -> Result<Json<SuccessfulTokenResponse>, CustomError> {
    conn.run(move |c| {
        let client =
            repository::find_client(&tokenparam.client_id, c).expect("failed to find the client");
        // check client credential
        if client.client_secret != tokenparam.client_secret {
            return Err(CustomError::BadRequest);
        }
        // check auth code
        if repository::find_auth_code(&tokenparam.code, c).is_err() {
            return Err(CustomError::BadRequest);
        }
        let access_token = generate_challenge();
        repository::create_token(
            NewToken {
                access_token: access_token.clone(),
            },
            c,
        )?;
        let now = Utc::now();
        let exp = now + Duration::hours(12);
        let claim = IdToken {
            iss: "http://localhost:5000".to_string(),
            sub: "userid".to_string(), // dummy id
            aud: tokenparam.client_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            nonce: "nonce".to_string(), // TODO: set nonce
        };
        let id_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claim,
            &jsonwebtoken::EncodingKey::from_base64_secret("c2VjcmV0a2V5").unwrap(),
        )?;
        Ok(Json(SuccessfulTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            refresh_token: None,
            expires_in: 3600,
            id_token,
        }))
    })
    .await
}

#[get("/userinfo")]
async fn get_userinfo(
    inforeq: UserinfoRequest,
    conn: DBPool,
) -> Result<Json<SuccessfulUserinfoResponse>, CustomError> {
    conn.run(move |c| {
        let token = repository::find_token(&inforeq.bearer, c)?;
        if token.is_valid() && token.access_token == inforeq.bearer {
            return Ok(Json(SuccessfulUserinfoResponse {
                sub: "userid".to_string(),
                name: "tarou tanaka".to_string(),
            }));
        }
        Err(CustomError::UnauthorizedError)
    })
    .await
}

#[launch]
pub fn run() -> _ {
    let db: Map<_, Value> = map! {
        "pool_size" => 10.into(),
    };
    let figment = rocket::Config::figment().merge(("databases", map!["oidc_db" => db]));

    rocket::custom(figment)
        .mount(
            "/",
            routes![
                index,
                get_client,
                get_authenticate,
                post_authenticate,
                get_authorization,
                post_authorization,
                post_token,
                get_userinfo,
            ],
        )
        .attach(DBPool::fairing())
        .attach(Template::fairing())
}
