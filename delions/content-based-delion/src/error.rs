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
