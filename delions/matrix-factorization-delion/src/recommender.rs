use std::cmp::Ordering;

use crate::error::MatrixFactorizationError;
use crate::factorizer::Factorizer;
use crate::model::LatentFactorModel;
use crate::rating_matrix::RatingMatrix;
use crate::types::{ItemId, ModelConfig, Rating, Recommendation, UserId};

/// Recommendation engine using matrix factorization.
pub struct MatrixFactorizationRecommender;

// Methods do not use self state but follow service-object pattern
// for future extensibility (e.g., caching, configuration)
#[allow(clippy::unused_self)]
impl MatrixFactorizationRecommender {
	/// Train a latent factor model from ratings.
	///
	/// # Errors
	///
	/// Returns an error if the rating matrix cannot be built or
	/// factorization fails.
	pub fn train(
		&self,
		ratings: &[Rating],
		config: &ModelConfig,
		factorizer: &dyn Factorizer,
	) -> Result<(LatentFactorModel, RatingMatrix), MatrixFactorizationError> {
		let matrix = RatingMatrix::from_ratings(ratings)?;
		let model = factorizer.factorize(&matrix, config)?;
		Ok((model, matrix))
	}

	/// Predict the rating for a specific user-item pair.
	///
	/// # Errors
	///
	/// Returns `UserNotFound` or `ItemNotFound` if the given IDs
	/// were not present in the training data.
	pub fn predict(
		&self,
		model: &LatentFactorModel,
		matrix: &RatingMatrix,
		user_id: UserId,
		item_id: ItemId,
	) -> Result<f64, MatrixFactorizationError> {
		let user_idx = matrix
			.user_to_index(user_id)
			.ok_or(MatrixFactorizationError::UserNotFound(user_id))?;
		let item_idx = matrix
			.item_to_index(item_id)
			.ok_or(MatrixFactorizationError::ItemNotFound(item_id))?;
		Ok(model.predict(user_idx, item_idx))
	}

	/// Recommend top-n unrated items for a user.
	///
	/// Returns items sorted by predicted score in descending order,
	/// excluding items already rated by the user.
	///
	/// # Errors
	///
	/// Returns `UserNotFound` if the user ID was not present in the
	/// training data.
	pub fn recommend(
		&self,
		model: &LatentFactorModel,
		matrix: &RatingMatrix,
		user_id: UserId,
		n: usize,
	) -> Result<Vec<Recommendation>, MatrixFactorizationError> {
		let user_idx = matrix
			.user_to_index(user_id)
			.ok_or(MatrixFactorizationError::UserNotFound(user_id))?;

		let mut recommendations: Vec<Recommendation> = (0..matrix.n_items())
			.filter(|&item_idx| !matrix.is_observed(user_idx, item_idx))
			.filter_map(|item_idx| {
				let item_id = matrix.index_to_item(item_idx)?;
				let score = model.predict(user_idx, item_idx);
				Some(Recommendation { item_id, score })
			})
			.collect();

		recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
		recommendations.truncate(n);

		Ok(recommendations)
	}
}

#[cfg(test)]
mod tests {
	use approx::assert_abs_diff_eq;
	use rstest::{fixture, rstest};

	use super::*;
	use crate::factorizer::AlsFactorizer;
	use crate::types::Rating;

	#[fixture]
	fn pipeline_ratings() -> Vec<Rating> {
		vec![
			Rating {
				user_id: UserId(0),
				item_id: ItemId(0),
				value: 5.0,
			},
			Rating {
				user_id: UserId(0),
				item_id: ItemId(1),
				value: 3.0,
			},
			Rating {
				user_id: UserId(0),
				item_id: ItemId(2),
				value: 1.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(0),
				value: 4.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(2),
				value: 2.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(3),
				value: 5.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(1),
				value: 1.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(2),
				value: 4.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(3),
				value: 3.0,
			},
		]
	}

	#[rstest]
	fn train_predict_pipeline(pipeline_ratings: Vec<Rating>) {
		// Arrange
		let recommender = MatrixFactorizationRecommender;
		let config = ModelConfig {
			n_factors: 3,
			regularization: 0.01,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};
		let factorizer = AlsFactorizer;

		// Act
		let (model, matrix) = recommender
			.train(&pipeline_ratings, &config, &factorizer)
			.unwrap();

		// Assert: predict an observed rating
		let pred = recommender
			.predict(&model, &matrix, UserId(0), ItemId(0))
			.unwrap();
		assert_abs_diff_eq!(pred, 5.0, epsilon = 1.0);
	}

	#[rstest]
	fn recommend_returns_unrated_items(pipeline_ratings: Vec<Rating>) {
		// Arrange
		let recommender = MatrixFactorizationRecommender;
		let config = ModelConfig {
			n_factors: 3,
			regularization: 0.01,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};
		let factorizer = AlsFactorizer;
		let (model, matrix) = recommender
			.train(&pipeline_ratings, &config, &factorizer)
			.unwrap();

		// Act: user 0 has not rated item 3
		let recs = recommender
			.recommend(&model, &matrix, UserId(0), 5)
			.unwrap();

		// Assert: should only recommend item 3 (the only unrated)
		assert_eq!(recs.len(), 1);
		assert_eq!(recs[0].item_id, ItemId(3));
	}

	#[rstest]
	fn predict_unknown_user_returns_error() {
		// Arrange
		let recommender = MatrixFactorizationRecommender;
		let ratings = vec![Rating {
			user_id: UserId(0),
			item_id: ItemId(0),
			value: 3.0,
		}];
		let config = ModelConfig::default();
		let factorizer = AlsFactorizer;
		let (model, matrix) = recommender.train(&ratings, &config, &factorizer).unwrap();

		// Act
		let result = recommender.predict(&model, &matrix, UserId(99), ItemId(0));

		// Assert
		assert!(matches!(
			result,
			Err(MatrixFactorizationError::UserNotFound(UserId(99)))
		));
	}

	#[rstest]
	fn predict_unknown_item_returns_error() {
		// Arrange
		let recommender = MatrixFactorizationRecommender;
		let ratings = vec![Rating {
			user_id: UserId(0),
			item_id: ItemId(0),
			value: 3.0,
		}];
		let config = ModelConfig::default();
		let factorizer = AlsFactorizer;
		let (model, matrix) = recommender.train(&ratings, &config, &factorizer).unwrap();

		// Act
		let result = recommender.predict(&model, &matrix, UserId(0), ItemId(99));

		// Assert
		assert!(matches!(
			result,
			Err(MatrixFactorizationError::ItemNotFound(ItemId(99)))
		));
	}

	#[rstest]
	fn predict_existing_user_item(pipeline_ratings: Vec<Rating>) {
		// Arrange
		let recommender = MatrixFactorizationRecommender;
		let config = ModelConfig {
			n_factors: 3,
			regularization: 0.01,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};
		let factorizer = AlsFactorizer;
		let (model, matrix) = recommender
			.train(&pipeline_ratings, &config, &factorizer)
			.unwrap();

		// Act: predict for multiple observed user-item pairs
		let pred_u0_i1 = recommender
			.predict(&model, &matrix, UserId(0), ItemId(1))
			.unwrap();
		let pred_u2_i3 = recommender
			.predict(&model, &matrix, UserId(2), ItemId(3))
			.unwrap();

		// Assert: predictions should be near observed values
		assert_abs_diff_eq!(pred_u0_i1, 3.0, epsilon = 1.0);
		assert_abs_diff_eq!(pred_u2_i3, 3.0, epsilon = 1.0);
	}

	#[rstest]
	fn recommend_n_exceeds_unrated_items(pipeline_ratings: Vec<Rating>) {
		// Arrange
		let recommender = MatrixFactorizationRecommender;
		let config = ModelConfig {
			n_factors: 3,
			regularization: 0.01,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};
		let factorizer = AlsFactorizer;
		let (model, matrix) = recommender
			.train(&pipeline_ratings, &config, &factorizer)
			.unwrap();

		// Act: user 0 has 3 rated items out of 4, so only 1 unrated;
		// requesting 100 should return only the available unrated items
		let recs = recommender
			.recommend(&model, &matrix, UserId(0), 100)
			.unwrap();

		// Assert: only 1 unrated item (item 3) for user 0
		assert_eq!(recs.len(), 1);
		assert_eq!(recs[0].item_id, ItemId(3));
	}
}
