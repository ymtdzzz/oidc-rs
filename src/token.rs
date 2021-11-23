use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
struct TokenRequest {
    #[validate(custom = "validate_grant_type")]
    grant_type: String,
    code: String,
    #[validate(url)]
    redirect_uri: String,
    client_id: String,
}

impl TokenRequest {
    pub fn validate(&self) -> Result<()> {
        // https://openid.net/specs/openid-connect-core-1_0.html#TokenRequestValidation
        // TODO:
        //  - Authenticate the Client if it was issued Client Credentials or if it uses another Client Authentication method, per Section 9.
        //  - Ensure the Authorization Code was issued to the authenticated Client.
        //  - Verify that the Authorization Code is valid.
        //  - If possible, verify that the Authorization Code has not been previously used.
        //  - Ensure that the redirect_uri parameter value is identical to the redirect_uri parameter value that was included in the initial Authorization Request. If the redirect_uri parameter value is not present when there is only one registered redirect_uri value, the Authorization Server MAY return an error (since the Client should have included the parameter) or MAY proceed without an error (since OAuth 2.0 permits the parameter to be omitted in this case).
        //  - Verify that the Authorization Code used was issued in response to an OpenID Connect Authentication Request (so that an ID Token will be returned from the Token Endpoint).
        Ok(())
    }
}

enum GrantType {
    AuthorizationCode,
}

impl FromStr for GrantType {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "authorization_code" => Ok(GrantType::AuthorizationCode),
            _ => Err(ValidationError::new("Unsupported grant_type")),
        }
    }
}

fn validate_grant_type(grant_type: &str) -> Result<(), ValidationError> {
    for s in grant_type.split_whitespace() {
        GrantType::from_str(s)?;
    }
    Ok(())
}

struct SuccessfulTokenResponse {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expires_in: u64,
    id_token: String,
}

struct ErrorTokenResponse {
    error: String,
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
        let expected = Err(ValidationError::new("Unsupported grant_type"));
        assert_eq!(expected, result);
    }
}
