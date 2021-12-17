use std::{convert::TryInto, str::FromStr};

use crate::{
    error::CustomError,
    message::{
        authentication::AuthenticationRequest,
        enums::{ResponseTypes, Scopes},
    },
    schema::*,
};
use anyhow::Result;
use chrono::{Duration, Utc};
use rocket::form::validate::Contains;
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

impl Client {
    pub fn check_scopes(&self, scopes: &Scopes) -> anyhow::Result<()> {
        let s = Scopes::from_str(&self.scope)?;
        for scope in &scopes.scopes {
            if !s.scopes.contains(scope) {
                return Err(anyhow::anyhow!("invalid scope"));
            }
        }
        Ok(())
    }

    pub fn check_restypes(&self, restypes: &ResponseTypes) -> anyhow::Result<()> {
        let r = ResponseTypes::from_str(&self.response_type)?;
        for restype in &restypes.types {
            if !r.types.contains(restype) {
                return Err(anyhow::anyhow!("invalid response type"));
            }
        }
        Ok(())
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
    pub state: Option<String>,
    pub nonce: Option<String>,
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
            nonce: req.nonce().to_owned(),
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
            &self.state,
            &self.nonce,
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
    pub nonce: String,
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

#[cfg(test)]
mod tests {
    use crate::message::enums::{ResponseType, Scope};

    use super::*;

    #[test]
    fn client_check_scopes_ok() {
        let input = Scopes {
            scopes: vec![Scope::OpenID, Scope::Profile],
        };
        let client = Client {
            client_id: String::default(),
            client_secret: String::default(),
            scope: String::from("openid email profile"),
            response_type: String::default(),
            redirect_uri: String::default(),
        };
        assert!(client.check_scopes(&input).is_ok());
    }

    #[test]
    fn client_check_sopes_ng() {
        let input = Scopes {
            scopes: vec![Scope::OpenID, Scope::Email],
        };
        let client = Client {
            client_id: String::default(),
            client_secret: String::default(),
            scope: String::from("openid profile"),
            response_type: String::default(),
            redirect_uri: String::default(),
        };
        assert!(client.check_scopes(&input).is_err());
    }

    #[test]
    fn client_check_restypes_ok() {
        let input = ResponseTypes {
            types: vec![ResponseType::Code],
        };
        let client = Client {
            client_id: String::default(),
            client_secret: String::default(),
            scope: String::default(),
            response_type: String::from("code"),
            redirect_uri: String::default(),
        };
        assert!(client.check_restypes(&input).is_ok());
    }

    #[test]
    fn client_check_restypes_ng() {
        let input = ResponseTypes {
            types: vec![ResponseType::Code],
        };
        let client = Client {
            client_id: String::default(),
            client_secret: String::default(),
            scope: String::default(),
            response_type: String::default(),
            redirect_uri: String::default(),
        };
        assert!(client.check_restypes(&input).is_err());
    }
}
