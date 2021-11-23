#[derive(FromForm)]
pub struct AuthorizationParams {
    pub scope: String,
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
}

#[derive(FromForm)]
pub struct TokenParams {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
}
