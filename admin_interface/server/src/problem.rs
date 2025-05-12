use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub title: String,
    pub detail: String,
    pub status: u16,
}

impl Problem {
    pub fn new<T: ToString, D: ToString>(title: T, status: u16, detail: D) -> Self {
        Problem {
            title: title.to_string(),
            status,
            detail: detail.to_string(),
        }
    }
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem {}: {}", self.title, self.detail)
    }
}

impl ResponseError for Problem {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(self)
    }
}

#[macro_export]
macro_rules! to_problem {
    ($title:expr, $status:expr) => {
        |error| crate::problem::Problem::new($title.to_string(), $status, format!("{}", error))
    };
}

#[macro_export]
macro_rules! problem {
    ($title:expr, $status:expr) => {
        crate::problem::Problem::new($title.to_string(), $status, String::new())
    };
    ($title:expr, $status:expr, $detail:expr) => {
        crate::problem::Problem::new($title.to_string(), $status, $detail)
    };
    ($title:expr, $status:expr, $detail:expr, $($arg:tt)*) => {
        crate::problem::Problem::new($title.to_string(), $status, format!($detail, $($arg)*))
    };
}
