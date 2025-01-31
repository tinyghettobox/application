use std::fmt::{Display, Formatter};

use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;

#[derive(Debug)]
pub struct Problem {
    pub status: u16,
    pub message: String,
    #[allow(dead_code)]
    pub root_cause: Option<String>,
}

impl Problem {
    pub fn new(status: u16, message: String, root_cause: Option<String>) -> Problem {
        Problem {
            status,
            message,
            root_cause,
        }
    }

    pub fn internal_error(message: String, root_cause: Option<String>) -> Problem {
        Problem::new(500, message, root_cause)
    }

    pub fn _bad_request(message: String, root_cause: Option<String>) -> Problem {
        Problem::new(400, message, root_cause)
    }

    pub fn _not_found(message: String, root_cause: Option<String>) -> Problem {
        Problem::new(404, message, root_cause)
    }
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem: {}", self.message)
    }
}

impl ResponseError for Problem {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .body(self.message.clone())
    }
}