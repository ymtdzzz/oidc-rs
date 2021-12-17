#[derive(FromForm)]
pub struct ClientParams {
    pub scope: String,
    pub response_type: String,
    pub redirect_uri: String,
}
