use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Unified error type for all API responses.
///
/// Produces JSON in the format:
/// ```json
/// {"error": {"code": "not_found", "message": "Human-readable description"}}
/// ```
#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl AppError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: msg.into(),
            details: None,
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: msg.into(),
            details: None,
        }
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "forbidden",
            message: msg.into(),
            details: None,
        }
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "validation_error",
            message: msg.into(),
            details: None,
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "conflict",
            message: msg.into(),
            details: None,
        }
    }

    pub fn gone(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::GONE,
            code: "gone",
            message: msg.into(),
            details: None,
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: msg.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut body = serde_json::json!({
            "error": {
                "code": self.code,
                "message": self.message,
            }
        });
        if let Some(details) = self.details {
            body["error"]["details"] = details;
        }
        (self.status, axum::Json(body)).into_response()
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(e: sea_orm::DbErr) -> Self {
        tracing::error!("Database error: {e}");
        Self::internal(format!("Database error: {e}"))
    }
}

/// Helper for JSON serialization in error details
#[derive(Serialize)]
pub struct FieldError {
    pub field: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}
