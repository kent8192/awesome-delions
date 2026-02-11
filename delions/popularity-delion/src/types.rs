use std::fmt;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::error::PopularityError;

/// Unique identifier for an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

impl fmt::Display for ItemId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Category label for grouping items.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Category(pub String);

impl fmt::Display for Category {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// A scored item produced by a popularity scorer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularityScore {
	pub item_id: ItemId,
	pub score: f64,
}

/// A recommended item with its computed score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
	pub item_id: ItemId,
	pub score: f64,
}

/// A time range used to filter interaction events.
#[derive(Debug, Clone)]
pub struct TimeWindow {
	pub start: SystemTime,
	pub end: SystemTime,
}

impl TimeWindow {
	/// Creates a new `TimeWindow` validating that `start` is before `end`.
	///
	/// # Errors
	///
	/// Returns [`PopularityError::InvalidTimeWindow`] if `start >= end`.
	pub fn new(start: SystemTime, end: SystemTime) -> Result<Self, PopularityError> {
		if start >= end {
			return Err(PopularityError::InvalidTimeWindow);
		}
		Ok(Self { start, end })
	}

	/// Returns `true` if the given time falls within this window (inclusive).
	#[must_use]
	pub fn contains(&self, time: SystemTime) -> bool {
		time >= self.start && time <= self.end
	}

	/// Returns the duration of this time window.
	///
	/// # Panics
	///
	/// Panics if the internal time calculation fails (should never happen since
	/// the constructor validates `start < end`).
	#[must_use]
	pub fn duration(&self) -> Duration {
		self.end
			.duration_since(self.start)
			// Safety: validated in constructor that start < end
			.expect("end should be after start")
	}
}

/// The kind of user interaction with an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionKind {
	View,
	Rating,
	Purchase,
	Click,
}

/// A recorded interaction event for an item.
#[derive(Debug, Clone)]
pub struct InteractionEvent {
	pub item_id: ItemId,
	pub timestamp: SystemTime,
	pub kind: InteractionKind,
}

/// Metadata associated with an item, including its category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
	pub id: ItemId,
	pub category: Category,
}

#[cfg(test)]
mod tests {
	use super::*;

	use rstest::rstest;

	#[rstest]
	fn item_id_display() {
		// Act
		let display = ItemId(42).to_string();

		// Assert
		assert_eq!(display, "42");
	}

	#[rstest]
	fn category_display() {
		// Act
		let display = Category("electronics".to_string()).to_string();

		// Assert
		assert_eq!(display, "electronics");
	}

	#[rstest]
	fn time_window_valid() {
		// Arrange
		let start = SystemTime::UNIX_EPOCH;
		let end = start + Duration::from_secs(3600);

		// Act
		let window = TimeWindow::new(start, end);

		// Assert
		assert!(window.is_ok());
	}

	#[rstest]
	fn time_window_invalid_start_equals_end() {
		// Arrange
		let time = SystemTime::UNIX_EPOCH;

		// Act
		let result = TimeWindow::new(time, time);

		// Assert
		assert!(matches!(result, Err(PopularityError::InvalidTimeWindow)));
	}

	#[rstest]
	fn time_window_invalid_start_after_end() {
		// Arrange
		let start = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
		let end = SystemTime::UNIX_EPOCH;

		// Act
		let result = TimeWindow::new(start, end);

		// Assert
		assert!(matches!(result, Err(PopularityError::InvalidTimeWindow)));
	}

	#[rstest]
	fn time_window_contains() {
		// Arrange
		let start = SystemTime::UNIX_EPOCH;
		let end = start + Duration::from_secs(3600);
		let window = TimeWindow::new(start, end).unwrap();
		let inside = start + Duration::from_secs(1800);
		let before = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
		let after = end + Duration::from_secs(1);

		// Assert
		assert!(window.contains(start));
		assert!(window.contains(end));
		assert!(window.contains(inside));
		assert!(!window.contains(before));
		assert!(!window.contains(after));
	}

	#[rstest]
	fn time_window_duration() {
		// Arrange
		let start = SystemTime::UNIX_EPOCH;
		let end = start + Duration::from_secs(3600);
		let window = TimeWindow::new(start, end).unwrap();

		// Act
		let duration = window.duration();

		// Assert
		assert_eq!(duration, Duration::from_secs(3600));
	}
}
