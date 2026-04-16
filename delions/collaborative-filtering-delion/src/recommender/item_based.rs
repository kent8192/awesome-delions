use crate::error::CollaborativeFilteringError;
use crate::rating_matrix::SparseRatingMatrix;
use crate::recommender::Recommender;
use crate::similarity::Similarity;
use crate::types::{CollaborativeFilteringConfig, ItemId, Recommendation, UserId};

/// Item-based collaborative filtering recommender.
///
/// Finds similar items and uses the user's ratings on those items
/// to predict ratings for unrated items.
pub struct ItemBasedRecommender {
	similarity: Box<dyn Similarity>,
}

impl ItemBasedRecommender {
	/// Creates a new item-based recommender with the given similarity metric.
	#[must_use]
	pub fn new(similarity: Box<dyn Similarity>) -> Self {
		Self { similarity }
	}

	/// Computes similarity between two items based on co-rating users.
	fn item_similarity(
		&self,
		matrix: &SparseRatingMatrix,
		item_a: ItemId,
		item_b: ItemId,
	) -> Result<f64, CollaborativeFilteringError> {
		let ratings_a = matrix
			.get_item_ratings(item_a)
			.ok_or(CollaborativeFilteringError::ItemNotFound(item_a))?;
		let ratings_b = matrix
			.get_item_ratings(item_b)
			.ok_or(CollaborativeFilteringError::ItemNotFound(item_b))?;

		// Find co-rating users
		let mut vec_a = Vec::new();
		let mut vec_b = Vec::new();
		for (user, &rating_a) in ratings_a {
			if let Some(&rating_b) = ratings_b.get(user) {
				vec_a.push(rating_a);
				vec_b.push(rating_b);
			}
		}

		if vec_a.is_empty() {
			return Err(CollaborativeFilteringError::InsufficientData);
		}

		self.similarity.compute(&vec_a, &vec_b)
	}
}

impl Recommender for ItemBasedRecommender {
	fn predict(
		&self,
		matrix: &SparseRatingMatrix,
		user_id: UserId,
		item_id: ItemId,
		config: &CollaborativeFilteringConfig,
	) -> Result<f64, CollaborativeFilteringError> {
		let user_ratings = matrix
			.get_user_ratings(user_id)
			.ok_or(CollaborativeFilteringError::UserNotFound(user_id))?;

		// Ensure the target item exists in the matrix
		if matrix.get_item_ratings(item_id).is_none() {
			return Err(CollaborativeFilteringError::ItemNotFound(item_id));
		}

		// Find similar items that the user has rated
		let mut neighbors: Vec<(f64, f64)> = Vec::new();
		for (&rated_item, &rating) in user_ratings {
			if rated_item == item_id {
				continue;
			}
			if let Ok(sim) = self.item_similarity(matrix, item_id, rated_item)
				&& sim > config.min_similarity
			{
				neighbors.push((sim, rating));
			}
		}

		// Sort by similarity descending and take top-k
		neighbors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
		neighbors.truncate(config.k_neighbors);

		if neighbors.is_empty() {
			return Err(CollaborativeFilteringError::InsufficientData);
		}

		// Weighted average
		let (weighted_sum, weight_total) = neighbors
			.iter()
			.fold((0.0, 0.0), |(ws, wt), &(sim, rating)| {
				(ws + sim * rating, wt + sim.abs())
			});

		if weight_total == 0.0 {
			return Err(CollaborativeFilteringError::InsufficientData);
		}

		Ok(weighted_sum / weight_total)
	}

	fn recommend(
		&self,
		matrix: &SparseRatingMatrix,
		user_id: UserId,
		n: usize,
		config: &CollaborativeFilteringConfig,
	) -> Result<Vec<Recommendation>, CollaborativeFilteringError> {
		let user_ratings = matrix
			.get_user_ratings(user_id)
			.ok_or(CollaborativeFilteringError::UserNotFound(user_id))?;
		let rated_items: std::collections::HashSet<_> = user_ratings.keys().collect();

		let mut predictions: Vec<Recommendation> = Vec::new();
		for item_id in matrix.items() {
			if rated_items.contains(item_id) {
				continue;
			}
			if let Ok(score) = self.predict(matrix, user_id, *item_id, config) {
				predictions.push(Recommendation {
					item_id: *item_id,
					score,
				});
			}
		}

		// Sort by score descending
		predictions.sort_by(|a, b| {
			b.score
				.partial_cmp(&a.score)
				.unwrap_or(std::cmp::Ordering::Equal)
		});
		predictions.truncate(n);

		Ok(predictions)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::similarity::CosineSimilarity;
	use crate::types::Rating;
	use rstest::{fixture, rstest};

	#[fixture]
	fn small_dataset() -> SparseRatingMatrix {
		// Multiple users rating overlapping items
		let ratings = vec![
			Rating {
				user_id: UserId(1),
				item_id: ItemId(1),
				value: 5.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(2),
				value: 4.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(3),
				value: 3.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(1),
				value: 4.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(2),
				value: 5.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(4),
				value: 4.0,
			},
			Rating {
				user_id: UserId(3),
				item_id: ItemId(1),
				value: 3.0,
			},
			Rating {
				user_id: UserId(3),
				item_id: ItemId(3),
				value: 2.0,
			},
			Rating {
				user_id: UserId(3),
				item_id: ItemId(4),
				value: 3.0,
			},
			Rating {
				user_id: UserId(3),
				item_id: ItemId(5),
				value: 5.0,
			},
		];
		SparseRatingMatrix::from_ratings(&ratings)
	}

	#[rstest]
	fn predict_returns_reasonable_score(small_dataset: SparseRatingMatrix) {
		// Arrange
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();

		// Act: predict user 1's rating for item 4
		let score = recommender
			.predict(&small_dataset, UserId(1), ItemId(4), &config)
			.unwrap();

		// Assert: score should be between 1.0 and 5.0
		assert!((1.0..=5.0).contains(&score));
	}

	#[rstest]
	fn predict_unknown_user_returns_error(small_dataset: SparseRatingMatrix) {
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();
		let result = recommender.predict(&small_dataset, UserId(999), ItemId(1), &config);
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::UserNotFound(_))
		));
	}

	#[rstest]
	fn predict_unknown_item_returns_error(small_dataset: SparseRatingMatrix) {
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();
		let result = recommender.predict(&small_dataset, UserId(1), ItemId(999), &config);
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::ItemNotFound(_))
		));
	}

	#[rstest]
	fn recommend_returns_unrated_items(small_dataset: SparseRatingMatrix) {
		// Arrange
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();

		// Act: recommend items for user 1 (has rated items 1,2,3)
		let recs = recommender
			.recommend(&small_dataset, UserId(1), 5, &config)
			.unwrap();

		// Assert: should only contain unrated items (4 and/or 5)
		for rec in &recs {
			assert!(rec.item_id == ItemId(4) || rec.item_id == ItemId(5));
		}
	}

	#[rstest]
	fn recommend_respects_n_limit(small_dataset: SparseRatingMatrix) {
		// Arrange
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();

		// Act
		let recs = recommender
			.recommend(&small_dataset, UserId(1), 1, &config)
			.unwrap();

		// Assert
		assert!(recs.len() <= 1);
	}

	#[rstest]
	fn high_min_similarity_filters_neighbors(small_dataset: SparseRatingMatrix) {
		// Arrange: set min_similarity to 1.0 so no item can qualify
		// (cosine similarity is strictly < 1.0 for non-identical co-rated vectors)
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig {
			k_neighbors: 20,
			min_similarity: 1.0,
		};

		// Act
		let result = recommender.predict(&small_dataset, UserId(1), ItemId(4), &config);

		// Assert: should fail due to no qualifying similar items
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::InsufficientData)
		));
	}

	#[rstest]
	fn k_neighbors_limits_neighbor_count(small_dataset: SparseRatingMatrix) {
		// Arrange: k=1 means only the single most similar item is used
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config_k1 = CollaborativeFilteringConfig {
			k_neighbors: 1,
			min_similarity: 0.0,
		};
		let config_all = CollaborativeFilteringConfig {
			k_neighbors: 20,
			min_similarity: 0.0,
		};

		// Act
		let score_k1 = recommender
			.predict(&small_dataset, UserId(1), ItemId(4), &config_k1)
			.unwrap();
		let score_all = recommender
			.predict(&small_dataset, UserId(1), ItemId(4), &config_all)
			.unwrap();

		// Assert: both produce valid scores
		assert!((1.0..=5.0).contains(&score_k1));
		assert!((1.0..=5.0).contains(&score_all));
	}

	#[rstest]
	fn user_rated_only_one_item() {
		// Arrange: user 4 has rated only item 1
		let ratings = vec![
			Rating {
				user_id: UserId(1),
				item_id: ItemId(1),
				value: 5.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(2),
				value: 4.0,
			},
			Rating {
				user_id: UserId(4),
				item_id: ItemId(1),
				value: 3.0,
			},
		];
		let matrix = SparseRatingMatrix::from_ratings(&ratings);
		let recommender = ItemBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();

		// Act: predict user 4's rating for item 2
		// item 2's co-rating users with item 1 is user 1 only → single user similarity
		let result = recommender.predict(&matrix, UserId(4), ItemId(2), &config);

		// Assert: should succeed since items 1 and 2 share user 1
		let score = result.unwrap();
		assert!((1.0..=5.0).contains(&score));
	}
}
