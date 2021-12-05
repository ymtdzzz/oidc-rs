use std::convert::TryInto;

use crate::{
    error::CustomError,
    internal::{authentication::AuthenticationRequest, client::Client as OIDCClient},
    schema::*,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "client"]
pub struct Client {
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    pub response_type: String,
    pub redirect_uri: String,
}

impl TryInto<OIDCClient> for Client {
    type Error = CustomError;

    fn try_into(self) -> Result<OIDCClient, Self::Error> {
        OIDCClient::new(&self.scope, &self.response_type, &self.redirect_uri)
            .map_err(|_e| CustomError::BadRequest)
    }
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "auth_challenges"]
pub struct AuthChallenge {
    pub challenge: String,
    pub client_id: String,
    pub scope: String,
    pub response_type: String,
    pub redirect_uri: String,
}

impl AuthChallenge {
    pub fn from_auth_request(challenge: &str, req: AuthenticationRequest) -> Result<Self> {
        Ok(AuthChallenge {
            challenge: challenge.to_string(),
            client_id: req.client_id().to_string(),
            scope: req.scope().to_string(),
            response_type: req.response_type().to_string(),
            redirect_uri: req.redirect_uri().to_string(),
        })
    }
}

impl TryInto<AuthenticationRequest> for AuthChallenge {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<AuthenticationRequest, Self::Error> {
        AuthenticationRequest::new(
            &self.scope,
            &self.response_type,
            &self.client_id,
            &self.redirect_uri,
            None,
            None,
        )
    }
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "session"]
pub struct Session {
    pub session_id: String,
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "auth_code"]
pub struct AuthCode {
    pub code: String,
}

#[derive(Queryable)]
pub struct Token {
    pub access_token: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "tokens"]
pub struct NewToken {
    pub access_token: String,
}
