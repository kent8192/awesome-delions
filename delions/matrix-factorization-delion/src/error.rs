use crate::types::{ItemId, UserId};

/// Errors that can occur during matrix factorization operations.
#[derive(Debug, thiserror::Error)]
pub enum MatrixFactorizationError {
	#[error("user not found: {0}")]
	UserNotFound(UserId),

	#[error("item not found: {0}")]
	ItemNotFound(ItemId),

	#[error("empty ratings")]
	EmptyRatings,

	#[error("convergence failure after {iterations} iterations (residual: {residual})")]
	ConvergenceFailure { iterations: usize, residual: f64 },

	#[error("invalid config: {0}")]
	InvalidConfig(String),

	#[error("singular matrix encountered")]
	SingularMatrix,

	#[error("numerical instability detected: {0}")]
	NumericalInstability(String),
}
