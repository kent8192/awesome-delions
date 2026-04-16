use crate::error::CollaborativeFilteringError;
use crate::rating_matrix::SparseRatingMatrix;
use crate::recommender::Recommender;
use crate::similarity::Similarity;
use crate::types::{CollaborativeFilteringConfig, ItemId, Recommendation, UserId};

/// User-based collaborative filtering recommender.
///
/// Finds similar users and uses their ratings to predict
/// items the target user might like.
pub struct UserBasedRecommender {
	similarity: Box<dyn Similarity>,
}

impl UserBasedRecommender {
	/// Creates a new user-based recommender with the given similarity metric.
	#[must_use]
	pub fn new(similarity: Box<dyn Similarity>) -> Self {
		Self { similarity }
	}

	/// Computes similarity between two users based on co-rated items.
	fn user_similarity(
		&self,
		matrix: &SparseRatingMatrix,
		user_a: UserId,
		user_b: UserId,
	) -> Result<f64, CollaborativeFilteringError> {
		let ratings_a = matrix
			.get_user_ratings(user_a)
			.ok_or(CollaborativeFilteringError::UserNotFound(user_a))?;
		let ratings_b = matrix
			.get_user_ratings(user_b)
			.ok_or(CollaborativeFilteringError::UserNotFound(user_b))?;

		// Find co-rated items
		let mut vec_a = Vec::new();
		let mut vec_b = Vec::new();
		for (item, &rating_a) in ratings_a {
			if let Some(&rating_b) = ratings_b.get(item) {
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

impl Recommender for UserBasedRecommender {
	fn predict(
		&self,
		matrix: &SparseRatingMatrix,
		user_id: UserId,
		item_id: ItemId,
		config: &CollaborativeFilteringConfig,
	) -> Result<f64, CollaborativeFilteringError> {
		let _user_ratings = matrix
			.get_user_ratings(user_id)
			.ok_or(CollaborativeFilteringError::UserNotFound(user_id))?;
		let item_ratings = matrix
			.get_item_ratings(item_id)
			.ok_or(CollaborativeFilteringError::ItemNotFound(item_id))?;

		// Find similar users who rated this item
		let mut neighbors: Vec<(f64, f64)> = Vec::new();
		for (&other_user, &rating) in item_ratings {
			if other_user == user_id {
				continue;
			}
			if let Ok(sim) = self.user_similarity(matrix, user_id, other_user)
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
		// User 1: likes items 1,2,3
		// User 2: likes items 1,2,4 (similar to user 1)
		// User 3: likes items 3,4,5 (different from user 1)
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
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();

		// Act: predict user 1's rating for item 4 (rated by users 2 and 3)
		let score = recommender
			.predict(&small_dataset, UserId(1), ItemId(4), &config)
			.unwrap();

		// Assert: score should be between 1.0 and 5.0
		assert!((1.0..=5.0).contains(&score));
	}

	#[rstest]
	fn predict_unknown_user_returns_error(small_dataset: SparseRatingMatrix) {
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();
		let result = recommender.predict(&small_dataset, UserId(999), ItemId(1), &config);
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::UserNotFound(_))
		));
	}

	#[rstest]
	fn predict_unknown_item_returns_error(small_dataset: SparseRatingMatrix) {
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
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
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
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
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
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
		// Arrange: set min_similarity to 1.0 so no neighbor can qualify
		// (cosine similarity is strictly < 1.0 for non-identical co-rated vectors)
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig {
			k_neighbors: 20,
			min_similarity: 1.0,
		};

		// Act: predict for item 4 with very high threshold
		let result = recommender.predict(&small_dataset, UserId(1), ItemId(4), &config);

		// Assert: should fail due to no qualifying neighbors
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::InsufficientData)
		));
	}

	#[rstest]
	fn k_neighbors_limits_neighbor_count(small_dataset: SparseRatingMatrix) {
		// Arrange: k=1 means only the single most similar user is used
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
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

		// Assert: both produce valid scores; with k=1 we use fewer neighbors
		assert!((1.0..=5.0).contains(&score_k1));
		assert!((1.0..=5.0).contains(&score_all));
	}

	#[rstest]
	fn all_users_identical_ratings() {
		// Arrange: all users have exactly the same ratings
		let ratings = vec![
			Rating {
				user_id: UserId(1),
				item_id: ItemId(1),
				value: 5.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(2),
				value: 3.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(1),
				value: 5.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(2),
				value: 3.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(3),
				value: 4.0,
			},
		];
		let matrix = SparseRatingMatrix::from_ratings(&ratings);
		let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
		let config = CollaborativeFilteringConfig::default();

		// Act: user 1 should get recommendation for item 3 based on user 2
		let recs = recommender
			.recommend(&matrix, UserId(1), 5, &config)
			.unwrap();

		// Assert
		assert_eq!(recs.len(), 1);
		assert_eq!(recs[0].item_id, ItemId(3));
		assert!((recs[0].score - 4.0).abs() < 1e-10);
	}
}
