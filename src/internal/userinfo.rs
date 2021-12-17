use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    Request,
};
use serde::Serialize;

pub struct UserinfoRequest {
    pub bearer: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for UserinfoRequest {
    type Error = anyhow::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(h) => {
                let auth_headers = h.split_whitespace().collect::<Vec<&str>>();
                let prefix = auth_headers.get(0);
                if prefix.is_none() || prefix.unwrap() != &"Bearer" {
                    return Outcome::Failure((
                        Status::Unauthorized,
                        anyhow::anyhow!("invalid token"),
                    ));
                }
                match auth_headers.get(1) {
                    Some(token) => Outcome::Success(UserinfoRequest {
                        bearer: token.to_string(),
                    }),
                    None => {
                        Outcome::Failure((Status::Unauthorized, anyhow::anyhow!("invalid token")))
                    }
                }
            }
            None => Outcome::Failure((Status::Unauthorized, anyhow::anyhow!("invalid token"))),
        }
    }
}

#[derive(Serialize)]
pub struct Address {
    pub formatted: String,
    pub street_address: String,
    pub locality: String,
    pub region: String,
    pub postal_code: String,
    pub country: String,
}

#[derive(Serialize)]
pub struct SuccessfulUserinfoResponse {
    pub sub: String,
    pub name: String, // sample data
    pub email: String,
    pub email_verified: bool,
    pub address: Address,
    pub phone_number: String,
    pub phone_number_verified: bool,
}

#[derive(Serialize)]
pub struct UserinfoErrorResponse {
    pub error: String,
    pub error_description: String,
}
