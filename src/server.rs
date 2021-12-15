use std::str::FromStr;

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
        authentication::{
            AuthenticationRequest, AuthenticationRequestParam, SuccessfulAuthenticationResponse,
        },
        enums::{Scope, Scopes},
        token::{Basic, IdToken, SuccessfulTokenResponse, TokenRequest},
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
    request::{ClientParams, ConsentGetParams, ConsentParams, LoginParams},
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
    authparam: AuthenticationRequestParam,
    conn: DBPool,
) -> Result<Template, CustomError> {
    conn.run(move |c| {
        let client = repository::find_client(&authparam.client_id(), c)?;
        let authparam = AuthenticationRequest::from(authparam, &client)?;
        let state = authparam.state().clone();
        let challenge = generate_challenge();
        repository::create_auth_challenge(
            AuthChallenge::from_auth_request(&challenge, authparam),
            c,
        )?;
        Ok(Template::render(
            "login",
            &LoginContext {
                error_msg: None,
                login_challenge: challenge,
                state,
            },
        ))
    })
    .await
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
            let mut next = format!(
                "/authorization?consent_challenge={}",
                &loginparam.login_challenge
            );
            if let Some(s) = &loginparam.state {
                next = format!("{}&state={}", next, s);
            }
            Ok(RedirectWithCookie {
                key: String::from("session_id"),
                value: session_id,
                next,
            })
        } else {
            Err(CustomError::ValidationError(Template::render(
                "login",
                &LoginContext {
                    error_msg: Some(String::from("username or password is incorrect")),
                    login_challenge: loginparam.login_challenge.to_string(),
                    state: loginparam.state.clone(),
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
                        state: param.state,
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
) -> Result<SuccessfulAuthenticationResponse, CustomError> {
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
        create_auth_code(
            AuthCode {
                code: auth_code.clone(),
                client_id: challenge.client_id.clone(),
                user_id: String::from("userid"), // dummy user id
                scope: challenge.scope.clone(),
                nonce: challenge.nonce.unwrap_or("".to_string()),
            },
            c,
        )?;
        Ok(SuccessfulAuthenticationResponse::new(
            &challenge.redirect_uri,
            &auth_code,
            &consentparam.state,
        ))
    })
    .await
}

#[post("/token", data = "<tokenparam>")]
async fn post_token(
    tokenparam: Form<TokenRequest>,
    basic: Basic,
    conn: DBPool,
) -> Result<Json<SuccessfulTokenResponse>, CustomError> {
    conn.run(move |c| {
        // check auth code
        let auth_code = repository::find_auth_code(tokenparam.code(), c)?;
        let client = repository::find_client(&auth_code.client_id, c)?;
        // check client credential
        if basic.client_id != client.client_id || basic.client_secret != client.client_secret {
            return Err(CustomError::UnauthorizedError);
        }
        let access_token = generate_challenge();
        repository::create_token(
            NewToken {
                access_token: access_token.clone(),
                user_id: auth_code.user_id,
                scope: auth_code.scope,
            },
            c,
        )?;
        let now = Utc::now();
        let exp = now + Duration::hours(12);
        let claim = IdToken {
            iss: "http://example.com".to_string(),
            sub: "userid".to_string(), // dummy id
            aud: auth_code.client_id,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            nonce: auth_code.nonce,
        };
        let jwt_header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let id_token = jsonwebtoken::encode(
            &jwt_header,
            &claim,
            &jsonwebtoken::EncodingKey::from_rsa_pem(include_bytes!("private-key.pem")).unwrap(),
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
            let scopes = Scopes::from_str(&token.scope).unwrap();
            let mut res = SuccessfulUserinfoResponse {
                sub: "userid".to_string(),
                name: "tarou tanaka".to_string(),
                email: None,
                email_verified: None,
                address: None,
                phone_number: None,
                phone_number_verified: None,
            };
            for s in scopes.scopes.iter() {
                match s {
                    &Scope::All => {
                        res.email = Some(String::from("test@example.com"));
                        res.email_verified = Some(true);
                        res.address = Some(String::from("address"));
                        res.phone_number = Some(String::from("111-1234-5678"));
                        res.phone_number_verified = Some(true);
                    }
                    &Scope::Phone => {
                        res.phone_number = Some(String::from("111-1234-5678"));
                        res.phone_number_verified = Some(true);
                    }
                    &Scope::Email => {
                        res.email = Some(String::from("test@example.com"));
                        res.email_verified = Some(true);
                    }
                    &Scope::Address => {
                        res.address = Some(String::from("address"));
                    }
                    _ => {}
                }
            }
            return Ok(Json(res));
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
