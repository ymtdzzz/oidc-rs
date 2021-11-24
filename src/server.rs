use rocket::{
    form::Form,
    response::{status::BadRequest, Redirect},
};
use rocket_dyn_templates::Template;

use crate::internal::authentication::AuthenticationRequest;

use self::{
    context::{ConsentContext, LoginContext},
    request::{AuthenticationParams, ConsentParams, LoginParams},
};

mod context;
mod request;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/authenticate?<authparam..>")]
fn get_authenticate(
    authparam: Option<AuthenticationParams>,
) -> Result<Template, BadRequest<String>> {
    match authparam {
        Some(param) => {
            let _auth_req = AuthenticationRequest::new(
                &param.scope,
                &param.response_type,
                &param.client_id,
                &param.redirect_uri,
                &param.state,
            )
            .map_err(|e| BadRequest(Some("Request is malformed: ".to_owned() + &e.to_string())))?;
            // TODO: save challenge and pass it to hidden value
            let challenge = "login_challenge".to_string();
            Ok(Template::render(
                "login",
                &LoginContext {
                    error_msg: None,
                    login_challenge: challenge,
                },
            ))
        }
        None => Err(BadRequest(Some("Request is malformed".to_string()))),
    }
}

#[post("/authenticate", data = "<loginparam>")]
fn post_authenticate(loginparam: Form<LoginParams>) -> Result<Redirect, Template> {
    // dummy authentication
    if loginparam.username == "foobar".to_string() && loginparam.password == "1234" {
        return Ok(Redirect::to("/authorization"));
    }
    // TODO: check if param login_challenge is correct
    Err(Template::render(
        "login",
        &LoginContext {
            error_msg: Some(String::from("username or password incorrect")),
            login_challenge: loginparam.login_challenge.to_string(),
        },
    ))
}

#[get("/authorization")]
fn get_authorization() -> Template {
    // TODO: login check
    // TODO: save challenge and pass it to hidden value
    let challenge = "consent_challenge".to_string();
    Template::render(
        "consent",
        &ConsentContext {
            consent_challenge: challenge,
        },
    )
}

#[post("/authorization", data = "<consentparam>")]
fn post_authorization(consentparam: Form<ConsentParams>) -> String {
    // TODO: check if param consent_challenge is correct
    format!(
        "consent_challenge: {}, consent: {} // TODO: redirect to RP callback including authorization code",
        consentparam.consent_challenge, consentparam.consent
    )
}

#[launch]
pub fn run() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                get_authenticate,
                post_authenticate,
                get_authorization,
                post_authorization,
            ],
        )
        .attach(Template::fairing())
}
