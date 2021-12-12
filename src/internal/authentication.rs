use core::fmt;
use std::str::FromStr;

use crate::{error::CustomError, models::Client};
use anyhow::Result;
use rocket::{
    http::{hyper::header::LOCATION, Header, Status},
    response::Responder,
    Request, Response,
};
use serde::Serialize;

use super::enums::{ResponseTypes, Scopes};

/// AuthenticationRequest represents a authentication request
/// https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest
pub struct AuthenticationRequest {
    scope: Scopes,
    response_type: ResponseTypes,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
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
    pub fn scope(&self) -> &Scopes {
        &self.scope
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn response_type(&self) -> &ResponseTypes {
        &self.response_type
    }

    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    pub fn state(&self) -> &Option<String> {
        &self.state
    }

    pub fn new(
        scope: &str,
        response_type: &str,
        client_id: &str,
        redirect_uri: &str,
        state: &Option<String>,
    ) -> Result<Self, CustomError> {
        Ok(Self {
            scope: Scopes::from_str(scope).map_err(|_e| {
                CustomError::AuthenticationError(ErrorAuthenticationResponse::new(
                    redirect_uri,
                    AuthorizationError::RequestNotSupported,
                    state,
                ))
            })?,
            response_type: ResponseTypes::from_str(response_type).map_err(|_e| {
                CustomError::AuthenticationError(ErrorAuthenticationResponse::new(
                    redirect_uri,
                    AuthorizationError::RequestNotSupported,
                    state,
                ))
            })?,
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            state: state.to_owned(),
        })
    }

    pub fn from(param: AuthenticationRequestParam, _client: &Client) -> Result<Self, CustomError> {
        // TODO: validation
        // TODO: validate with client
        let scope = Scopes::from_str(&param.scope).map_err(|_e| {
            CustomError::AuthenticationError(ErrorAuthenticationResponse::new(
                "hoge",
                AuthorizationError::RequestNotSupported,
                &param.state,
            ))
        })?;
        let response_type = ResponseTypes::from_str(&param.response_type).map_err(|_e| {
            CustomError::AuthenticationError(ErrorAuthenticationResponse::new(
                "hoge",
                AuthorizationError::RequestNotSupported,
                &param.state,
            ))
        })?;

        Ok(AuthenticationRequest {
            scope,
            response_type,
            client_id: param.client_id.to_string(),
            redirect_uri: param.redirect_uri.to_string(),
            state: param.state.map(|s| s.to_string()),
        })
    }
}

#[derive(FromForm)]
pub struct AuthenticationRequestParam {
    scope: String,
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
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

impl AuthenticationRequestParam {
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

/// SuccessfulAuthenticationResponse represents a successful authentication response
/// https://openid.net/specs/openid-connect-core-1_0.html#AuthResponse
#[derive(Serialize)]
pub struct SuccessfulAuthenticationResponse {
    next: String,
    code: String,
    state: Option<String>,
}

impl SuccessfulAuthenticationResponse {
    pub fn new(next: &str, code: &str, state: &Option<String>) -> Self {
        Self {
            next: next.to_string(),
            code: code.to_string(),
            state: state.to_owned(),
        }
    }
}

impl<'r> Responder<'r, 'static> for SuccessfulAuthenticationResponse {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut next = format!("{}?code={}", self.next, self.code);
        if let Some(s) = self.state {
            next = format!("{}&state={}", next, s);
        }
        Response::build()
            .status(Status::Found)
            .header(Header::new(LOCATION.as_str(), next))
            .ok()
    }
}

/// AuthorizationError represents an error code for ErrorAuthenticationResponse
#[derive(Debug)]
pub enum AuthorizationError {
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

#[derive(Serialize, Debug)]
pub struct ErrorAuthenticationResponse {
    next: String,
    error: AuthorizationError,
    error_description: Option<String>,
    error_uri: Option<String>,
    state: Option<String>,
}

impl ErrorAuthenticationResponse {
    pub fn new(next: &str, error: AuthorizationError, state: &Option<String>) -> Self {
        Self {
            next: next.to_string(),
            error,
            error_description: None,
            error_uri: None,
            state: state.to_owned(),
        }
    }
}

impl<'r> Responder<'r, 'static> for ErrorAuthenticationResponse {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut next = format!("{}?error={}", self.next, self.error);
        if let Some(desc) = self.error_description {
            next = format!("{}&error_description={}", next, desc);
        }
        if let Some(euri) = self.error_uri {
            next = format!("{}&error_uri={}", next, euri);
        }
        if let Some(s) = self.state {
            next = format!("{}&state={}", next, s);
        }
        Response::build()
            .status(Status::Found)
            .header(Header::new(LOCATION.as_str(), next))
            .ok()
    }
}
