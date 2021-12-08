use std::convert::Infallible;

use rocket::{
    outcome::try_outcome,
    request::{self, FromRequest, Outcome},
    Request,
};

#[derive(FromForm)]
pub struct ClientParams {
    pub scope: String,
    pub response_type: String,
    pub redirect_uri: String,
}

#[derive(FromForm)]
pub struct AuthenticationParams {
    pub scope: String,
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}

#[derive(FromForm)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
    pub login_challenge: String,
}

#[derive(FromForm, Debug)]
pub struct ConsentGetParams {
    pub consent_challenge: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for ConsentGetParams {
    type Error = Infallible;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let param = try_outcome!(request.guard::<ConsentGetParams>().await);
        Outcome::Success(param)
    }
}

#[derive(FromForm)]
pub struct ConsentParams {
    pub consent: String,
    pub consent_challenge: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for ConsentParams {
    type Error = Infallible;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let param = try_outcome!(request.guard::<ConsentParams>().await);
        Outcome::Success(param)
    }
}

#[derive(FromForm)]
pub struct TokenParams {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
}
