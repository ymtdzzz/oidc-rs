use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::Responder,
    Response,
};
use rocket_dyn_templates::Template;
use thiserror::Error;

use crate::server::context::ErrorContext;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Database error")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Bad request")]
    BadRequest,
    #[error("Validation error")]
    ValidationError(Template),
    #[error("Session error")]
    SessionError,
    #[error("Challenge error")]
    ChallengeError,
    #[error("Unauthorized error")]
    UnauthorizedError,
    #[error("JWT error")]
    JWTError(#[from] jsonwebtoken::errors::Error),
}

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::DatabaseError(e) => {
                let body = format!("Internal error: {}", e);
                let res = Response::build()
                    .status(Status::InternalServerError)
                    .header(ContentType::Plain)
                    .sized_body(body.len(), Cursor::new(body))
                    .finalize();
                Ok(res)
            }
            Self::BadRequest => {
                let res = Response::build().status(Status::BadRequest).finalize();
                Ok(res)
            }
            Self::ValidationError(template) => template.respond_to(request),
            Self::SessionError => Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from(
                        "Session doesn't exist or is invalid. Please retry from login page.",
                    ),
                },
            )
            .respond_to(request),
            Self::ChallengeError => Template::render(
                "error",
                &ErrorContext {
                    error_msg: String::from("Challenge is incorrect."),
                },
            )
            .respond_to(request),
            Self::UnauthorizedError => {
                let res = Response::build().status(Status::Unauthorized).finalize();
                Ok(res)
            }
            Self::JWTError(e) => {
                let body = format!("Internal error: {}", e);
                let res = Response::build()
                    .status(Status::InternalServerError)
                    .header(ContentType::Plain)
                    .sized_body(body.len(), Cursor::new(body))
                    .finalize();
                Ok(res)
            }
        }
    }
}
