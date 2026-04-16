mod item_based;
mod user_based;

pub use item_based::ItemBasedRecommender;
pub use user_based::UserBasedRecommender;

use crate::error::CollaborativeFilteringError;
use crate::rating_matrix::SparseRatingMatrix;
use crate::types::{CollaborativeFilteringConfig, ItemId, Recommendation, UserId};

/// Trait for recommendation algorithms.
pub trait Recommender: Send + Sync {
	/// Predicts the rating a user would give to an item.
	///
	/// # Errors
	///
	/// Returns `CollaborativeFilteringError` if the user or item is not found,
	/// or if there is insufficient data to make a prediction.
	fn predict(
		&self,
		matrix: &SparseRatingMatrix,
		user_id: UserId,
		item_id: ItemId,
		config: &CollaborativeFilteringConfig,
	) -> Result<f64, CollaborativeFilteringError>;

	/// Recommends the top-n items for a user.
	///
	/// # Errors
	///
	/// Returns `CollaborativeFilteringError` if the user is not found.
	fn recommend(
		&self,
		matrix: &SparseRatingMatrix,
		user_id: UserId,
		n: usize,
		config: &CollaborativeFilteringConfig,
	) -> Result<Vec<Recommendation>, CollaborativeFilteringError>;
}
