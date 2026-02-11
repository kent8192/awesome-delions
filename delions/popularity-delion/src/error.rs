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

#[cfg(test)]
mod tests {
	use super::*;

	use rstest::rstest;

	#[rstest]
	fn display_item_not_found() {
		let err = PopularityError::ItemNotFound(ItemId(42));
		assert_eq!(err.to_string(), "item not found: 42");
	}

	#[rstest]
	fn display_category_not_found() {
		let err = PopularityError::CategoryNotFound(Category("x".to_string()));
		assert_eq!(err.to_string(), "category not found: x");
	}

	#[rstest]
	fn display_invalid_time_window() {
		let err = PopularityError::InvalidTimeWindow;
		assert_eq!(
			err.to_string(),
			"invalid time window: start must be before end"
		);
	}

	#[rstest]
	fn display_no_events() {
		let err = PopularityError::NoEvents;
		assert_eq!(err.to_string(), "no events provided");
	}

	#[rstest]
	fn display_invalid_decay_parameter() {
		let err = PopularityError::InvalidDecayParameter("msg".to_string());
		assert_eq!(err.to_string(), "invalid decay parameter: msg");
	}
}
