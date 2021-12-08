use std::{iter::FromIterator, str::FromStr};

use anyhow::{anyhow, Result};
use rocket::form::{self, DataField, Errors, FromFormField, ValueField};

pub struct Scopes {
    scopes: Vec<Scope>,
}

impl FromStr for Scopes {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_whitespace()
            .map(|scope| Scope::from_str(scope))
            .collect()
    }
}

impl FromIterator<Scope> for Scopes {
    fn from_iter<T: IntoIterator<Item = Scope>>(iter: T) -> Self {
        let mut scopes = Scopes { scopes: vec![] };
        for s in iter {
            scopes.scopes.push(s)
        }
        scopes
    }
}

impl ToString for Scopes {
    fn to_string(&self) -> String {
        self.scopes
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[async_trait]
impl<'r> FromFormField<'r> for Scopes {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Self::from_str(field.value)
            .map_err(|e| Errors::from(form::Error::validation(format!("invalid scope: {}", e))))
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        let scopes = field
            .request
            .query_value::<Vec<&str>>("scope")
            .ok_or(Errors::from(form::Error::validation("invalid scope")))??;
        Ok(scopes
            .iter()
            .map(|s| Scope::from_str(s))
            .collect::<Result<Vec<Scope>>>()
            .map_err(|e| Errors::from(form::Error::validation(format!("invalid scope: {}", e))))?
            .into_iter()
            .collect::<Scopes>())
    }
}

pub enum Scope {
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

impl ToString for Scope {
    fn to_string(&self) -> String {
        match self {
            Scope::OpenID => String::from("openid"),
            Scope::Profile => String::from("profile"),
            Scope::Address => String::from("address"),
            Scope::Phone => String::from("phone"),
        }
    }
}

pub struct ResponseTypes {
    types: Vec<ResponseType>,
}

impl ToString for ResponseTypes {
    fn to_string(&self) -> String {
        self.types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl FromStr for ResponseTypes {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_whitespace()
            .map(|res_type| ResponseType::from_str(res_type))
            .collect()
    }
}

impl FromIterator<ResponseType> for ResponseTypes {
    fn from_iter<T: IntoIterator<Item = ResponseType>>(iter: T) -> Self {
        let mut res_types = ResponseTypes { types: vec![] };
        for res_type in iter {
            res_types.types.push(res_type)
        }
        res_types
    }
}

#[async_trait]
impl<'r> FromFormField<'r> for ResponseTypes {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Self::from_str(field.value)
            .map_err(|e| Errors::from(form::Error::validation(format!("invalid scope: {}", e))))
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        let scopes = field
            .request
            .query_value::<Vec<&str>>("response_type")
            .ok_or(Errors::from(form::Error::validation("invalid scope")))??;
        Ok(scopes
            .iter()
            .map(|s| ResponseType::from_str(s))
            .collect::<Result<Vec<ResponseType>>>()
            .map_err(|e| {
                Errors::from(form::Error::validation(format!(
                    "invalid response_type: {}",
                    e
                )))
            })?
            .into_iter()
            .collect::<ResponseTypes>())
    }
}

pub enum ResponseType {
    Code,
}

impl ToString for ResponseType {
    fn to_string(&self) -> String {
        match self {
            &ResponseType::Code => String::from("code"),
        }
    }
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

pub fn validate_scope(scope: &str) -> Result<()> {
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

pub fn validate_response_type(response_type: &str) -> Result<()> {
    for s in response_type.split_whitespace() {
        ResponseType::from_str(s)?;
    }
    Ok(())
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
