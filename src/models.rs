use std::convert::TryInto;

use crate::{error::CustomError, internal::authentication::AuthenticationRequest, schema::*};
use anyhow::Result;
use chrono::{Duration, Utc};
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

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "auth_challenges"]
pub struct AuthChallenge {
    pub challenge: String,
    pub client_id: String,
    pub scope: String,
    pub response_type: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}

impl AuthChallenge {
    pub fn from_auth_request(challenge: &str, req: AuthenticationRequest) -> Self {
        AuthChallenge {
            challenge: challenge.to_string(),
            client_id: req.client_id().to_string(),
            scope: req.scope().to_string(),
            response_type: req.response_type().to_string(),
            redirect_uri: req.redirect_uri().to_string(),
            state: req.state().to_owned(),
        }
    }
}

impl TryInto<AuthenticationRequest> for AuthChallenge {
    type Error = CustomError;

    fn try_into(self) -> Result<AuthenticationRequest, Self::Error> {
        AuthenticationRequest::new(
            &self.scope,
            &self.response_type,
            &self.client_id,
            &self.redirect_uri,
            &None,
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
    pub client_id: String,
    pub user_id: String,
    pub scope: String,
}

#[derive(Queryable)]
pub struct Token {
    pub access_token: String,
    pub user_id: String,
    pub scope: String,
    pub created_at: chrono::NaiveDateTime,
}

impl Token {
    pub fn is_valid(&self) -> bool {
        // Expiration: 1 hour
        let expired_at = self.created_at + Duration::hours(1);
        let now = Utc::now().naive_utc();
        expired_at >= now
    }
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "tokens"]
pub struct NewToken {
    pub access_token: String,
    pub user_id: String,
    pub scope: String,
}
