use crate::types::{ItemId, UserId};

/// Errors that can occur in content-based recommendation operations.
#[derive(Debug, thiserror::Error)]
pub enum ContentBasedError {
	#[error("item not found: {0}")]
	ItemNotFound(ItemId),

	#[error("user not found: {0}")]
	UserNotFound(UserId),

	#[error("empty corpus")]
	EmptyCorpus,

	#[error("dimension mismatch: expected {expected}, got {actual}")]
	DimensionMismatch { expected: usize, actual: usize },

	#[error("empty feature vector")]
	EmptyFeatureVector,

	#[error("vocabulary not built yet")]
	VocabularyNotBuilt,
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	fn display_item_not_found() {
		let err = ContentBasedError::ItemNotFound(ItemId(42));
		assert_eq!(err.to_string(), "item not found: 42");
	}

	#[rstest]
	fn display_user_not_found() {
		let err = ContentBasedError::UserNotFound(UserId(7));
		assert_eq!(err.to_string(), "user not found: 7");
	}

	#[rstest]
	fn display_empty_corpus() {
		let err = ContentBasedError::EmptyCorpus;
		assert_eq!(err.to_string(), "empty corpus");
	}

	#[rstest]
	fn display_vocabulary_not_built() {
		let err = ContentBasedError::VocabularyNotBuilt;
		assert_eq!(err.to_string(), "vocabulary not built yet");
	}

	#[rstest]
	fn display_empty_feature_vector() {
		let err = ContentBasedError::EmptyFeatureVector;
		assert_eq!(err.to_string(), "empty feature vector");
	}

	#[rstest]
	fn display_dimension_mismatch() {
		let err = ContentBasedError::DimensionMismatch {
			expected: 3,
			actual: 5,
		};
		assert_eq!(err.to_string(), "dimension mismatch: expected 3, got 5");
	}
}
