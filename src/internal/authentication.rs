use core::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::Serialize;

/// AuthenticationRequest represents a authentication request
/// https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest
pub struct AuthenticationRequest {
    scope: Scope,
    response_type: ResponseType,
    client_id: String,
    redirect_uri: String,
    state: String,
    // nonce: String,
    // display: String,
    // prompt: String,
    // max_age: u64,
    // ui_locales: String,
    // id_token_hint: String,
    // login_hint: String,
    // acr_values: String,
    // Claims https://openid.net/specs/openid-connect-core-1_0.html#ClaimsParameter
    // claims: Claims,
}

impl AuthenticationRequest {
    pub fn new(
        scope: &str,
        response_type: &str,
        client_id: &str,
        redirect_uri: &str,
        state: &str,
    ) -> Result<Self> {
        // https://openid.net/specs/openid-connect-core-1_0.html#AuthRequestValidation
        // TODO:
        //  - The Authorization Server MUST validate all the OAuth 2.0 parameters according to the OAuth 2.0 specification.
        //  - Verify that a scope parameter is present and contains the openid scope value. (If no openid scope value is present, the request may still be a valid OAuth 2.0 request, but is not an OpenID Connect request.)
        //  - The Authorization Server MUST verify that all the REQUIRED parameters are present and their usage conforms to this specification.
        //  - If the sub (subject) Claim is requested with a specific value for the ID Token, the Authorization Server MUST only send a positive response if the End-User identified by that sub value has an active session with the Authorization Server or has been Authenticated as a result of the request. The Authorization Server MUST NOT reply with an ID Token or Access Token for a different user, even if they have an active session with the Authorization Server. Such a request can be made either using an id_token_hint parameter or by requesting a specific Claim Value as described in Section 5.5.1, if the claims parameter is supported by the implementation.

        Ok(AuthenticationRequest {
            scope: Scope::from_str(scope)?,
            response_type: ResponseType::from_str(response_type)?,
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            state: state.to_string(),
        })
    }
}

enum Scope {
    OpenID,
    Profile,
    Address,
    Phone,
}

impl FromStr for Scope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openid" => Ok(Scope::OpenID),
            "profile" => Ok(Scope::Profile),
            "address" => Ok(Scope::Address),
            "phone" => Ok(Scope::Phone),
            _ => Err(anyhow!("Unsupported scope")),
        }
    }
}

enum ResponseType {
    Code,
}

impl FromStr for ResponseType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(ResponseType::Code),
            _ => Err(anyhow!("Unsupported response_type")),
        }
    }
}

fn validate_scope(scope: &str) -> Result<()> {
    let mut openid_found = false;
    for s in scope.split_whitespace() {
        match Scope::from_str(s) {
            Ok(s) => {
                if matches!(s, Scope::OpenID) {
                    openid_found = true;
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    if !openid_found {
        return Err(anyhow!("scope openid is required"));
    }
    Ok(())
}

fn validate_response_type(response_type: &str) -> Result<()> {
    for s in response_type.split_whitespace() {
        ResponseType::from_str(s)?;
    }
    Ok(())
}

/// SuccessfulAuthenticationResponse represents a successful authentication response
/// https://openid.net/specs/openid-connect-core-1_0.html#AuthResponse
#[derive(Serialize)]
struct SuccessfulAuthenticationResponse {
    next: String,
    code: String,
    state: String,
}

/// AuthorizationError represents an error code for ErrorAuthenticationResponse
enum AuthorizationError {
    InteractionRequired,
    LoginRequired,
    AccountSelectionRequired,
    ConsentRequired,
    InvalidRequestUri,
    InvalidRequestObject,
    RequestNotSupported,
    RequestUriNotSupported,
    RegistrationNotSupported,
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthorizationError::InteractionRequired => write!(f, "interaction_required"),
            AuthorizationError::LoginRequired => write!(f, "login_required"),
            AuthorizationError::AccountSelectionRequired => write!(f, "account_selection_required"),
            AuthorizationError::ConsentRequired => write!(f, "consent_required"),
            AuthorizationError::InvalidRequestUri => write!(f, "invalid_request_uri"),
            AuthorizationError::InvalidRequestObject => write!(f, "invalid_request_object"),
            AuthorizationError::RequestNotSupported => write!(f, "request_not_supported"),
            AuthorizationError::RequestUriNotSupported => write!(f, "request_uri_not_supported"),
            AuthorizationError::RegistrationNotSupported => write!(f, "registration_not_supported"),
        }
    }
}

impl Serialize for AuthorizationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize)]
struct ErrorAuthenticationResponse {
    error: AuthorizationError,
    error_description: String,
    error_uri: String,
    state: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_scope_ok() {
        let result = validate_scope("profile openid  phone");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_scope_err_openid_notfound() {
        let result = validate_scope("profile address");
        let expected: Result<()> = Err(anyhow!("scope openid is required"));
        assert_eq!(
            expected.err().unwrap().to_string(),
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn validate_scope_err_unsupported_scope() {
        let result = validate_scope("openid wrongscope");
        let expected: Result<()> = Err(anyhow!("Unsupported scope"));
        assert_eq!(
            expected.err().unwrap().to_string(),
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn validate_response_type_ok() {
        let result = validate_response_type("code");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_response_type_err_unsupported_type() {
        let result = validate_response_type("code hoge");
        let expected: Result<()> = Err(anyhow!("Unsupported response_type"));
        assert_eq!(
            expected.err().unwrap().to_string(),
            result.err().unwrap().to_string()
        );
    }
}
