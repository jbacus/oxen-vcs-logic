use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Conflict(String),
    Internal(String),
    NotImplemented(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let message = self.to_string();

        HttpResponse::build(status).json(serde_json::json!({
            "error": message,
        }))
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_display() {
        let error = AppError::NotFound("Resource".to_string());
        assert_eq!(error.to_string(), "Not found: Resource");
    }

    #[test]
    fn test_bad_request_display() {
        let error = AppError::BadRequest("Invalid input".to_string());
        assert_eq!(error.to_string(), "Bad request: Invalid input");
    }

    #[test]
    fn test_unauthorized_display() {
        let error = AppError::Unauthorized("Invalid token".to_string());
        assert_eq!(error.to_string(), "Unauthorized: Invalid token");
    }

    #[test]
    fn test_conflict_display() {
        let error = AppError::Conflict("Already exists".to_string());
        assert_eq!(error.to_string(), "Conflict: Already exists");
    }

    #[test]
    fn test_internal_display() {
        let error = AppError::Internal("Database error".to_string());
        assert_eq!(error.to_string(), "Internal error: Database error");
    }

    #[test]
    fn test_not_implemented_display() {
        let error = AppError::NotImplemented("Feature pending".to_string());
        assert_eq!(error.to_string(), "Not implemented: Feature pending");
    }

    #[test]
    fn test_not_found_status() {
        let error = AppError::NotFound("Resource".to_string());
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_bad_request_status() {
        let error = AppError::BadRequest("Invalid".to_string());
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_unauthorized_status() {
        let error = AppError::Unauthorized("No auth".to_string());
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_conflict_status() {
        let error = AppError::Conflict("Exists".to_string());
        assert_eq!(error.status_code(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_internal_status() {
        let error = AppError::Internal("Error".to_string());
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_not_implemented_status() {
        let error = AppError::NotImplemented("Pending".to_string());
        assert_eq!(error.status_code(), StatusCode::NOT_IMPLEMENTED);
    }

    #[test]
    fn test_error_response_format() {
        let error = AppError::NotFound("User".to_string());
        let response = error.error_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_app_result_err() {
        let result: AppResult<i32> = Err(AppError::NotFound("Item".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let error = AppError::NotFound("Test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("Test"));
    }
}
