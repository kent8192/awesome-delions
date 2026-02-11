use crate::types::{ItemId, UserId};

/// Errors that can occur during collaborative filtering operations.
#[derive(Debug, thiserror::Error)]
pub enum CollaborativeFilteringError {
	#[error("user not found: {0}")]
	UserNotFound(UserId),
	#[error("item not found: {0}")]
	ItemNotFound(ItemId),
	#[error("insufficient data for recommendation")]
	InsufficientData,
	#[error("empty vectors provided for similarity computation")]
	EmptyVectors,
	#[error("zero norm vector")]
	ZeroNorm,
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;

	#[rstest]
	fn user_not_found_display() {
		let err = CollaborativeFilteringError::UserNotFound(UserId(42));
		assert_eq!(err.to_string(), "user not found: 42");
	}

	#[rstest]
	fn item_not_found_display() {
		let err = CollaborativeFilteringError::ItemNotFound(ItemId(99));
		assert_eq!(err.to_string(), "item not found: 99");
	}

	#[rstest]
	fn empty_vectors_display() {
		let err = CollaborativeFilteringError::EmptyVectors;
		assert_eq!(
			err.to_string(),
			"empty vectors provided for similarity computation"
		);
	}

	#[rstest]
	fn zero_norm_display() {
		let err = CollaborativeFilteringError::ZeroNorm;
		assert_eq!(err.to_string(), "zero norm vector");
	}

	#[rstest]
	fn insufficient_data_display() {
		let err = CollaborativeFilteringError::InsufficientData;
		assert_eq!(err.to_string(), "insufficient data for recommendation");
	}
}
