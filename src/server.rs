use rocket::{
    form::Form,
    response::{
        content::{self, Html},
        status::BadRequest,
        Redirect, Responder,
    },
};

use crate::internal::authentication::AuthenticationRequest;

use self::request::{AuthenticationParams, LoginParams};

mod request;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/authenticate?<authparam..>")]
fn get_authenticate(
    authparam: Option<AuthenticationParams>,
) -> Result<Html<String>, BadRequest<String>> {
    match authparam {
        Some(param) => {
            let auth_req = AuthenticationRequest::new(
                &param.scope,
                &param.response_type,
                &param.client_id,
                &param.redirect_uri,
                &param.state,
            )
            .map_err(|e| BadRequest(Some("Request is malformed: ".to_owned() + &e.to_string())))?;
            // TODO: save challenge and pass it to hidden value
            let challenge = "challenge";
            let response = r#"
                <html>
                    <form action="/authenticate" method="POST">
                        <label for="username">username</label>
                        <input name="username" id="username" value="">
                        <label for="password">password</label>
                        <input name="password" id="password" type="password" value="">
                        <input name="challenge" type="hidden" value="TODO: challenge"><br>
                        <p>username: foobar, password: 1234</p>
                        <button>Login</button>
                    </form>
                </html>
            "#
            .to_string();
            Ok(Html(response))
        }
        None => Err(BadRequest(Some("Request is malformed".to_string()))),
    }
}

#[post("/authenticate", data = "<loginparam>")]
fn post_authenticate(loginparam: Form<LoginParams>) -> Result<Redirect, Html<String>> {
    Ok(Redirect::to("/authorization"))
}

#[launch]
pub fn run() -> _ {
    rocket::build().mount("/", routes![index, get_authenticate, post_authenticate])
}
