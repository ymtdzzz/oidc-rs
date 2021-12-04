use std::convert::TryInto;

use diesel::result::Error;
use diesel::MysqlConnection;
use rocket::{
    figment::{
        util::map,
        value::{Map, Value},
    },
    form::Form,
    http::CookieJar,
    response::status::BadRequest,
};
use rocket_dyn_templates::Template;

use crate::{
    internal::authentication::AuthenticationRequest,
    models::{AuthChallenge, AuthCode, Client, Session},
    repository::{
        self, create_auth_code, create_client, create_session, find_auth_challenge, find_session,
    },
    utils::generate_challenge,
};

use self::{
    context::{ConsentContext, ErrorContext, LoginContext},
    request::{AuthenticationParams, ClientParams, ConsentGetParams, ConsentParams, LoginParams},
    response::RedirectWithCookie,
};

mod context;
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
) -> Result<String, BadRequest<String>> {
    conn.run(move |c| match clientparam {
        Some(param) => {
            let client_id = generate_challenge();
            create_client(
                Client {
                    client_id: client_id.clone(),
                    scope: param.scope,
                    response_type: param.response_type,
                    redirect_uri: param.redirect_uri,
                },
                c,
            )
            .expect("failed to store the client");
            Ok(format!("client_id: {}", client_id))
        }
        None => Err(BadRequest(Some("Request is malformed".to_string()))),
    })
    .await
}

#[get("/authenticate?<authparam..>")]
async fn get_authenticate(
    authparam: Option<AuthenticationParams>,
    conn: DBPool,
) -> Result<Template, BadRequest<String>> {
    match authparam {
        Some(param) => {
            conn.run(move |c| {
                let client = repository::find_client(&param.client_id, c).expect("failed to get the client");
                let auth_req = AuthenticationRequest::new(
                    &param.scope,
                    &param.response_type,
                    &param.client_id,
                    &param.redirect_uri,
                    param.state,
                    Some(&client.try_into().unwrap()),
                ).expect("Request is malformed");
                let challenge = generate_challenge();
                repository::create_auth_challenge(
                    AuthChallenge::from_auth_request(&challenge, auth_req).expect("failed to convert from the AuthenticationRequest into the model AuthChallenge"),
                    c,
                )
                .expect("failed to store the challenge");
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
        None => Err(BadRequest(Some("Request is malformed".to_string()))),
    }
}

#[post("/authenticate", data = "<loginparam>")]
async fn post_authenticate(
    loginparam: Form<LoginParams>,
    conn: DBPool,
) -> Result<RedirectWithCookie, Template> {
    conn.run(move |c| {
        if find_auth_challenge(&loginparam.login_challenge, c).is_err() {
            return Err(Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from("Challenge is incorrect."),
                },
            ));
        }
        if loginparam.username == "foobar".to_string() && loginparam.password == "1234" {
            let session_id = generate_challenge();
            // TODO: error handling
            create_session(
                Session {
                    session_id: session_id.clone(),
                },
                c,
            )
            .expect("failed to store the session_id");
            Ok(RedirectWithCookie {
                key: String::from("session_id"),
                value: session_id,
                next: format!(
                    "/authorization?consent_challenge={}",
                    &loginparam.login_challenge
                ),
            })
        } else {
            Err(Template::render(
                "login",
                &LoginContext {
                    error_msg: Some(String::from("username or password is incorrect")),
                    login_challenge: loginparam.login_challenge.to_string(),
                },
            ))
        }
    })
    .await
}

#[get("/authorization?<consentgetparam..>")]
async fn get_authorization<'a>(
    consentgetparam: Option<ConsentGetParams>,
    jar: &'a CookieJar<'_>,
    conn: DBPool,
) -> Template {
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
            return Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from(
                        "Session doesn't exist or is invalid. Please retry from login page.",
                    ),
                },
            );
        }
        // challenge check
        match consentgetparam {
            Some(param) => {
                if find_auth_challenge(&param.consent_challenge, c).is_err() {
                    return Template::render(
                        "error",
                        &ErrorContext {
                            error_msg: String::from("Challenge is incorrect."),
                        },
                    );
                }
                Template::render(
                    "consent",
                    &ConsentContext {
                        consent_challenge: param.consent_challenge,
                    },
                )
            }
            None => Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from("Challenge is incorrect."),
                },
            ),
        }
    })
    .await
}

#[post("/authorization", data = "<consentparam>")]
async fn post_authorization<'a>(
    consentparam: Form<ConsentParams>,
    jar: &'a CookieJar<'_>,
    conn: DBPool,
) -> Result<String, Template> {
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
            return Err(Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from(
                        "Session doesn't exist or is invalid. Please retry from login page.",
                    ),
                },
            ));
        }
        // challenge check
        if find_auth_challenge(&consentparam.consent_challenge, c).is_err() {
            return Err(Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from("Challenge is incorrect."),
                },
            ));
        }
        // TODO: error handling
        let auth_code = generate_challenge();
        create_auth_code(AuthCode {
            code: auth_code.clone(),
        }, c).expect("failed to store the auth_code");
        Ok(format!(
            "consent_challenge: {}, consent: {}, auth_code: {} // TODO: redirect to RP callback including authorization code",
            consentparam.consent_challenge, consentparam.consent, auth_code
        ))
    }).await
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
            ],
        )
        .attach(DBPool::fairing())
        .attach(Template::fairing())
}
