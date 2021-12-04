use serde::Serialize;

#[derive(Serialize)]
pub struct LoginContext {
    pub error_msg: Option<String>,
    pub login_challenge: String,
}

#[derive(Serialize)]
pub struct ConsentContext {
    pub consent_challenge: String,
}

#[derive(Serialize)]
pub struct ErrorContext {
    pub error_msg: String,
}
