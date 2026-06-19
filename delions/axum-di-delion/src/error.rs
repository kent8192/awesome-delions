use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
};
use reinhardt::di::DiError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiRejection {
	#[error("dependency injection context is missing from request extensions")]
	MissingContext,
	#[error("failed to resolve dependency: {message}")]
	Resolve { message: String },
}

impl DiRejection {
	pub(crate) fn resolve(err: &DiError) -> Self {
		Self::Resolve {
			message: err.to_string(),
		}
	}
}

impl From<DiError> for DiRejection {
	fn from(err: DiError) -> Self {
		Self::resolve(&err)
	}
}

impl IntoResponse for DiRejection {
	fn into_response(self) -> Response {
		(StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
	}
}
