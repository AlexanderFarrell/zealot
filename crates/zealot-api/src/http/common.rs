use axum::{http::StatusCode, response::IntoResponse};



pub enum HttpError {
    NotFound,
    Unauthorized,
    Internal,
    UserError{err: String},
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        match self {
            HttpError::NotFound => {
                (StatusCode::NOT_FOUND, "Not found").into_response()
            },
            HttpError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
            },
            HttpError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            },
            HttpError::UserError { err } => {
                (StatusCode::BAD_REQUEST, err.as_str()).into_response()
            }
        }
    }
}