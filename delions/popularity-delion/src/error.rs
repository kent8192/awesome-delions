use crate::types::{Category, ItemId};

/// Errors that can occur in popularity scoring and recommendation operations.
#[derive(Debug, thiserror::Error)]
pub enum PopularityError {
	#[error("item not found: {0}")]
	ItemNotFound(ItemId),

	#[error("category not found: {0}")]
	CategoryNotFound(Category),

	#[error("invalid time window: start must be before end")]
	InvalidTimeWindow,

	#[error("no events provided")]
	NoEvents,

	#[error("invalid decay parameter: {0}")]
	InvalidDecayParameter(String),
}
