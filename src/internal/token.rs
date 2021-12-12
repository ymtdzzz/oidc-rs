use std::{fmt, str::FromStr};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::enums::GrantType;

#[derive(FromForm)]
pub struct TokenRequest {
    grant_type: GrantType,
    code: String,
    redirect_uri: String,
    client_id: String,
    client_secret: String,
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

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    pub fn new(
        grant_type: &str,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<TokenRequest> {
        // https://openid.net/specs/openid-connect-core-1_0.html#TokenRequestValidation
        // TODO:
        //  - Authenticate the Client if it was issued Client Credentials or if it uses another Client Authentication method, per Section 9.
        //  - Ensure the Authorization Code was issued to the authenticated Client.
        //  - Verify that the Authorization Code is valid.
        //  - If possible, verify that the Authorization Code has not been previously used.
        //  - Ensure that the redirect_uri parameter value is identical to the redirect_uri parameter value that was included in the initial Authorization Request. If the redirect_uri parameter value is not present when there is only one registered redirect_uri value, the Authorization Server MAY return an error (since the Client should have included the parameter) or MAY proceed without an error (since OAuth 2.0 permits the parameter to be omitted in this case).  - Verify that the Authorization Code used was issued in response to an OpenID Connect Authentication Request (so that an ID Token will be returned from the Token Endpoint).
        Ok(TokenRequest {
            grant_type: GrantType::from_str(grant_type)?,
            code: code.to_string(),
            redirect_uri: redirect_uri.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        })
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
