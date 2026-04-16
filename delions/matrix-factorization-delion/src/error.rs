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

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	fn display_empty_ratings() {
		let err = MatrixFactorizationError::EmptyRatings;
		assert_eq!(err.to_string(), "empty ratings");
	}

	#[rstest]
	fn display_user_not_found() {
		let err = MatrixFactorizationError::UserNotFound(UserId(42));
		assert_eq!(err.to_string(), "user not found: 42");
	}

	#[rstest]
	fn display_item_not_found() {
		let err = MatrixFactorizationError::ItemNotFound(ItemId(99));
		assert_eq!(err.to_string(), "item not found: 99");
	}

	#[rstest]
	fn display_convergence_failure() {
		let err = MatrixFactorizationError::ConvergenceFailure {
			iterations: 50,
			residual: 0.5,
		};
		assert_eq!(
			err.to_string(),
			"convergence failure after 50 iterations (residual: 0.5)"
		);
	}

	#[rstest]
	fn display_invalid_config() {
		let err = MatrixFactorizationError::InvalidConfig("bad value".into());
		assert_eq!(err.to_string(), "invalid config: bad value");
	}

	#[rstest]
	fn display_singular_matrix() {
		let err = MatrixFactorizationError::SingularMatrix;
		assert_eq!(err.to_string(), "singular matrix encountered");
	}

	#[rstest]
	fn display_numerical_instability() {
		let err = MatrixFactorizationError::NumericalInstability("overflow detected".into());
		assert_eq!(
			err.to_string(),
			"numerical instability detected: overflow detected"
		);
	}
}
