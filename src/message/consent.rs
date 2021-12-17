use std::convert::Infallible;

use rocket::{
    outcome::try_outcome,
    request::{self, FromRequest, Outcome},
    Request,
};

#[derive(FromForm, Debug)]
pub struct ConsentGetParams {
    pub consent_challenge: String,
    pub state: Option<String>,
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
    pub state: Option<String>,
}

#[async_trait]
impl<'r> FromRequest<'r> for ConsentParams {
    type Error = Infallible;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let param = try_outcome!(request.guard::<ConsentParams>().await);
        Outcome::Success(param)
    }
}
