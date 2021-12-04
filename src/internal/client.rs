use std::str::FromStr;

use anyhow::Result;

use crate::utils::generate_challenge;

use super::enums::{ResponseTypes, Scopes};

pub struct Client {
    client_id: String,
    scope: Scopes,
    response_type: ResponseTypes,
    redirect_uri: String,
}

impl Client {
    pub fn new(scope: &str, response_type: &str, redirect_uri: &str) -> Result<Self> {
        Ok(Client {
            client_id: generate_challenge(),
            scope: Scopes::from_str(scope)?,
            response_type: ResponseTypes::from_str(response_type)?,
            redirect_uri: redirect_uri.to_string(),
        })
    }
}
