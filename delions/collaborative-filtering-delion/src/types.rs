use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl fmt::Display for UserId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Unique identifier for an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

impl fmt::Display for ItemId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// A single rating given by a user to an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
	pub user_id: UserId,
	pub item_id: ItemId,
	pub value: f64,
}

/// A recommendation result with an item and its predicted score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
	pub item_id: ItemId,
	pub score: f64,
}

/// Configuration for collaborative filtering algorithms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeFilteringConfig {
	/// Number of nearest neighbors to consider.
	pub k_neighbors: usize,
	/// Minimum similarity threshold for including a neighbor.
	pub min_similarity: f64,
}

impl Default for CollaborativeFilteringConfig {
	fn default() -> Self {
		Self {
			k_neighbors: 20,
			min_similarity: 0.0,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;

	#[rstest]
	fn user_id_display() {
		assert_eq!(UserId(42).to_string(), "42");
	}

	#[rstest]
	fn item_id_display() {
		assert_eq!(ItemId(99).to_string(), "99");
	}

	#[rstest]
	fn config_default_values() {
		let config = CollaborativeFilteringConfig::default();
		assert_eq!(config.k_neighbors, 20);
		assert!((config.min_similarity - 0.0).abs() < f64::EPSILON);
	}
}
