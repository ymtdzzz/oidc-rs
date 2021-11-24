#[derive(FromForm)]
pub struct AuthenticationParams {
    pub scope: String,
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
}

#[derive(FromForm)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
    pub login_challenge: String,
}

#[derive(FromForm)]
pub struct ConsentParams {
    pub consent: String,
    pub consent_challenge: String,
}

#[derive(FromForm)]
pub struct TokenParams {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
}
