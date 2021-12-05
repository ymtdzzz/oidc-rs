use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

struct TokenRequest {
    grant_type: GrantType,
    code: String,
    redirect_uri: String,
    client_id: String,
}

impl TokenRequest {
    pub fn new(
        grant_type: &str,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
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
        })
    }
}

enum GrantType {
    AuthorizationCode,
}

impl FromStr for GrantType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "authorization_code" => Ok(GrantType::AuthorizationCode),
            _ => Err(anyhow!("Unsupported grant_type")),
        }
    }
}

fn validate_grant_type(grant_type: &str) -> Result<()> {
    for s in grant_type.split_whitespace() {
        GrantType::from_str(s)?;
    }
    Ok(())
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
struct ErrorTokenResponse {
    error: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_grant_type_ok() {
        let result = validate_grant_type("authorization_code");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_grant_type_err_unsupported_type() {
        let result = validate_grant_type("authorization_code invalid_grant_type");
        let expected: Result<TokenRequest> = Err(anyhow!("Unsupported grant_type"));
        assert_eq!(
            expected.err().unwrap().to_string(),
            result.err().unwrap().to_string()
        );
    }
}
