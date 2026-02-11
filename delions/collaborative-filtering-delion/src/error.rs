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
