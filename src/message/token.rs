use std::fmt;

use anyhow::Result;
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    Request,
};
use serde::{Deserialize, Serialize};

use super::enums::GrantType;

pub struct Basic {
    pub client_id: String,
    pub client_secret: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for Basic {
    type Error = anyhow::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(h) => {
                let auth_headers = h.split_whitespace().collect::<Vec<&str>>();
                let prefix = auth_headers.get(0);
                if prefix.is_none() || prefix.unwrap() != &"Basic" {
                    return Outcome::Failure((
                        Status::Unauthorized,
                        anyhow::anyhow!("invalid token"),
                    ));
                }
                match auth_headers.get(1) {
                    Some(token_base64) => {
                        if let Ok(token_bytes) = base64::decode(token_base64) {
                            if let Ok(token_decoded) = std::str::from_utf8(&token_bytes) {
                                let id_secret = token_decoded.split(":").collect::<Vec<&str>>();
                                if id_secret.get(0).is_some() && id_secret.get(1).is_some() {
                                    return Outcome::Success(Basic {
                                        client_id: id_secret.get(0).unwrap().to_string(),
                                        client_secret: id_secret.get(1).unwrap().to_string(),
                                    });
                                }
                            }
                        }
                        return Outcome::Failure((
                            Status::Unauthorized,
                            anyhow::anyhow!("invalid token"),
                        ));
                    }
                    None => {
                        Outcome::Failure((Status::Unauthorized, anyhow::anyhow!("invalid token")))
                    }
                }
            }
            None => Outcome::Failure((Status::Unauthorized, anyhow::anyhow!("invalid token"))),
        }
    }
}

#[derive(FromForm)]
pub struct TokenRequest {
    grant_type: GrantType,
    code: String,
    redirect_uri: String,
}

impl TokenRequest {
    pub fn grant_type(&self) -> &GrantType {
        &self.grant_type
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }
}

#[derive(Serialize)]
pub struct SuccessfulTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub id_token: String,
}

#[derive(Serialize)]
pub struct ErrorTokenResponse {
    pub error: TokenError,
}

#[derive(Serialize, Deserialize)]
pub struct IdToken {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub nonce: String,
}

pub enum TokenError {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &TokenError::InvalidRequest => write!(f, "invalid_request"),
            &TokenError::InvalidClient => write!(f, "invalid_client"),
            &TokenError::InvalidGrant => write!(f, "invalid_grant"),
            &TokenError::UnauthorizedClient => write!(f, "unauthorized_client"),
            &TokenError::UnsupportedGrantType => write!(f, "unsupported_grant_type"),
            &TokenError::InvalidScope => write!(f, "invalid_scope"),
        }
    }
}

impl Serialize for TokenError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}
