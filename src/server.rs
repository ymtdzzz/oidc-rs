use rocket::{form::Form, http::RawStr, response::status::BadRequest};

use self::request::AuthorizationParams;

mod handler;
mod request;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/authorize?<authparam..>")]
fn get_authorize(authparam: Option<AuthorizationParams>) -> Result<String, BadRequest<String>> {
    match authparam {
        Some(param) => Ok(format!("scope: {}", param.scope)),
        None => Err(BadRequest(Some("ng".to_string()))),
    }
}

#[launch]
pub fn run() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![get_authorize])
}
