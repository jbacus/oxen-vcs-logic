// Integration tests for error handling
// Verifies that errors are properly propagated and HTTP status codes are correct

use auxin_server::error::{AppError, AppResult};

#[test]
fn test_not_implemented_error() {
    let error = AppError::NotImplemented("Test operation not implemented".to_string());
    let error_str = error.to_string();
    assert!(error_str.contains("Not implemented"));
    assert!(error_str.contains("Test operation"));
}

#[test]
fn test_not_found_error() {
    let error = AppError::NotFound("Repository not found".to_string());
    let error_str = error.to_string();
    assert!(error_str.contains("Not found"));
    assert!(error_str.contains("Repository"));
}

#[test]
fn test_bad_request_error() {
    let error = AppError::BadRequest("Invalid input".to_string());
    let error_str = error.to_string();
    assert!(error_str.contains("Bad request"));
}

#[test]
fn test_unauthorized_error() {
    let error = AppError::Unauthorized("Access denied".to_string());
    let error_str = error.to_string();
    assert!(error_str.contains("Unauthorized"));
}

#[test]
fn test_internal_error() {
    let error = AppError::Internal("Something went wrong".to_string());
    let error_str = error.to_string();
    assert!(error_str.contains("Internal error"));
}

#[test]
fn test_error_status_codes() {
    use actix_web::http::StatusCode;
    use actix_web::ResponseError;

    assert_eq!(
        AppError::NotFound("test".to_string()).status_code(),
        StatusCode::NOT_FOUND
    );

    assert_eq!(
        AppError::BadRequest("test".to_string()).status_code(),
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        AppError::Unauthorized("test".to_string()).status_code(),
        StatusCode::UNAUTHORIZED
    );

    assert_eq!(
        AppError::Internal("test".to_string()).status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );

    assert_eq!(
        AppError::NotImplemented("test".to_string()).status_code(),
        StatusCode::NOT_IMPLEMENTED
    );
}

#[test]
fn test_error_response_format() {
    use actix_web::ResponseError;

    let error = AppError::NotImplemented("VCS operation not available in mock mode".to_string());
    let response = error.error_response();

    // Verify response has correct status
    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::NOT_IMPLEMENTED
    );
}
