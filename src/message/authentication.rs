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
    nonce: Option<String>,
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

    pub fn nonce(&self) -> &Option<String> {
        &self.nonce
    }

    pub fn new(
        scope: &str,
        response_type: &str,
        client_id: &str,
        redirect_uri: &str,
        state: &Option<String>,
        nonce: &Option<String>,
    ) -> Result<Self, CustomError> {
        Ok(Self {
            scope: Scopes::from_str(scope).or(Err(CustomError::AuthenticationError(
                ErrorAuthenticationResponse::new(
                    redirect_uri,
                    AuthorizationError::InvalidScope,
                    state,
                ),
            )))?,
            response_type: ResponseTypes::from_str(response_type).or(Err(
                CustomError::AuthenticationError(ErrorAuthenticationResponse::new(
                    redirect_uri,
                    AuthorizationError::UnsupportedResponseType,
                    state,
                )),
            ))?,
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            state: state.to_owned(),
            nonce: nonce.to_owned(),
        })
    }

    pub fn from(param: AuthenticationRequestParam, client: &Client) -> Result<Self, CustomError> {
        let redirect_uri = param.redirect_uri.unwrap_or("".to_string());
        let param_scope = param.scope.ok_or(CustomError::AuthenticationError(
            ErrorAuthenticationResponse::new(
                &redirect_uri,
                AuthorizationError::InvalidRequest,
                &param.state,
            ),
        ))?;
        let scope = Scopes::from_str(&param_scope).or(Err(CustomError::AuthenticationError(
            ErrorAuthenticationResponse::new(
                &redirect_uri,
                AuthorizationError::InvalidScope,
                &param.state,
            ),
        )))?;
        client
            .check_scopes(&scope)
            .or(Err(CustomError::AuthenticationError(
                ErrorAuthenticationResponse::new(
                    &redirect_uri,
                    AuthorizationError::InvalidScope,
                    &param.state,
                ),
            )))?;
        let param_res_type = param.response_type.ok_or(CustomError::AuthenticationError(
            ErrorAuthenticationResponse::new(
                &redirect_uri,
                AuthorizationError::InvalidRequest,
                &param.state,
            ),
        ))?;
        let response_type = ResponseTypes::from_str(&param_res_type).or(Err(
            CustomError::AuthenticationError(ErrorAuthenticationResponse::new(
                "hoge",
                AuthorizationError::UnsupportedResponseType,
                &param.state,
            )),
        ))?;
        client
            .check_restypes(&response_type)
            .or(Err(CustomError::AuthenticationError(
                ErrorAuthenticationResponse::new(
                    &redirect_uri,
                    AuthorizationError::UnsupportedResponseType,
                    &param.state,
                ),
            )))?;
        let param_client_id = param.client_id.ok_or(CustomError::AuthenticationError(
            ErrorAuthenticationResponse::new(
                &redirect_uri,
                AuthorizationError::InvalidRequest,
                &param.state,
            ),
        ))?;

        Ok(AuthenticationRequest {
            scope,
            response_type,
            client_id: param_client_id,
            redirect_uri,
            state: param.state.map(|s| s.to_string()),
            nonce: param.nonce.map(|s| s.to_string()),
        })
    }
}

#[derive(FromForm, Clone)]
pub struct AuthenticationRequestParam {
    pub scope: Option<String>,
    pub response_type: Option<String>,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
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
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
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
            AuthorizationError::InvalidRequest => write!(f, "invalid_request"),
            AuthorizationError::UnauthorizedClient => write!(f, "unauthorized_client"),
            AuthorizationError::AccessDenied => write!(f, "access_denied"),
            AuthorizationError::UnsupportedResponseType => write!(f, "unsupported_response_type"),
            AuthorizationError::InvalidScope => write!(f, "invalid_scope"),
            AuthorizationError::ServerError => write!(f, "server_error"),
            AuthorizationError::TemporarilyUnavailable => write!(f, "temporarily_unavailable"),
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
