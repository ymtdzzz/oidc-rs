use std::{iter::FromIterator, str::FromStr};

use anyhow::{anyhow, Result};
use rocket::form::{self, DataField, Errors, FromFormField, ValueField};

#[derive(PartialEq, Debug)]
pub struct Scopes {
    pub scopes: Vec<Scope>,
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

#[derive(PartialEq, Debug)]
pub enum Scope {
    OpenID,
    Profile,
    Address,
    Phone,
    Email,
}

impl FromStr for Scope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openid" => Ok(Scope::OpenID),
            "profile" => Ok(Scope::Profile),
            "address" => Ok(Scope::Address),
            "phone" => Ok(Scope::Phone),
            "email" => Ok(Scope::Email),
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
            Scope::Email => String::from("email"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ResponseTypes {
    pub types: Vec<ResponseType>,
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

#[derive(PartialEq, Debug)]
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

#[derive(PartialEq, Debug)]
pub enum GrantType {
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

#[async_trait]
impl<'r> FromFormField<'r> for GrantType {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Self::from_str(field.value).map_err(|e| {
            Errors::from(form::Error::validation(format!(
                "invalid grant_type: {}",
                e
            )))
        })
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        GrantType::from_str(
            field
                .request
                .query_value::<&str>("grant_type")
                .ok_or(Errors::from(form::Error::validation("invalid grant_type")))??,
        )
        .map_err(|e| {
            Errors::from(form::Error::validation(format!(
                "invalid grant_type: {}",
                e
            )))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_from_str_ok() {
        let result = Scopes::from_str("openid profile address phone email");
        let expected = Scopes {
            scopes: vec![
                Scope::OpenID,
                Scope::Profile,
                Scope::Address,
                Scope::Phone,
                Scope::Email,
            ],
        };
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn scopes_from_str_ng() {
        let result = Scopes::from_str("openid profile phone hogehoge");
        assert!(result.is_err());
    }

    #[test]
    fn scopes_to_str_ok() {
        let input = Scopes {
            scopes: vec![
                Scope::OpenID,
                Scope::Profile,
                Scope::Address,
                Scope::Phone,
                Scope::Email,
            ],
        };
        let result = input.to_string();
        let expected = String::from("openid profile address phone email");
        assert_eq!(expected, result);
    }

    #[test]
    fn response_type_from_str_ok() {
        let result = ResponseTypes::from_str("code");
        let expected = ResponseTypes {
            types: vec![ResponseType::Code],
        };
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn response_type_from_str_ng() {
        let result = ResponseTypes::from_str("aaaaa");
        assert!(result.is_err());
    }

    #[test]
    fn response_type_to_str_ok() {
        let input = ResponseTypes {
            types: vec![ResponseType::Code],
        };
        let result = input.to_string();
        let expected = String::from("code");
        assert_eq!(expected, result);
    }

    #[test]
    fn grant_type_from_str_ok() {
        let result = GrantType::from_str("authorization_code");
        let expected = GrantType::AuthorizationCode;
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn grant_type_from_str_ng() {
        let result = GrantType::from_str("aaaaa");
        assert!(result.is_err());
    }
}
