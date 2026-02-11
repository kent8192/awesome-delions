use std::fmt;

use serde::{Deserialize, Serialize};

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

/// A single user-item rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
	pub user_id: UserId,
	pub item_id: ItemId,
	pub value: f64,
}

/// A recommended item with its predicted score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
	pub item_id: ItemId,
	pub score: f64,
}

/// Configuration for latent factor model training.
#[derive(Debug, Clone)]
pub struct ModelConfig {
	/// Number of latent factors.
	pub n_factors: usize,
	/// Learning rate (unused by SVD/ALS, reserved for future SGD).
	pub learning_rate: f64,
	/// Regularization parameter (lambda).
	pub regularization: f64,
	/// Maximum number of iterations.
	pub max_iterations: usize,
	/// Convergence tolerance.
	pub tolerance: f64,
}

impl Default for ModelConfig {
	fn default() -> Self {
		Self {
			n_factors: 20,
			learning_rate: 0.01,
			regularization: 0.1,
			max_iterations: 100,
			tolerance: 1e-4,
		}
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	fn user_id_display() {
		assert_eq!(UserId(42).to_string(), "42");
	}

	#[rstest]
	fn item_id_display() {
		assert_eq!(ItemId(99).to_string(), "99");
	}

	#[rstest]
	fn model_config_default() {
		// Act
		let config = ModelConfig::default();

		// Assert
		assert_eq!(config.n_factors, 20);
		assert_eq!(config.learning_rate, 0.01);
		assert_eq!(config.regularization, 0.1);
		assert_eq!(config.max_iterations, 100);
		assert_eq!(config.tolerance, 1e-4);
	}
}
