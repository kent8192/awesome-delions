use std::collections::HashSet;

use crate::error::ContentBasedError;
use crate::profile::item::ItemProfile;
use crate::profile::user::UserProfile;
use crate::similarity::SimilarityMetric;
use crate::types::{ItemId, Recommendation};

/// Content-based recommender that ranks items by feature similarity to a
/// user profile.
pub struct ContentBasedRecommender {
	similarity: Box<dyn SimilarityMetric>,
}

impl ContentBasedRecommender {
	/// Creates a new recommender with the given similarity metric.
	#[must_use]
	pub fn new(similarity: Box<dyn SimilarityMetric>) -> Self {
		Self { similarity }
	}

	/// Generates top-N recommendations for a user.
	///
	/// Computes similarity between the user profile and each item profile,
	/// excluding items in `exclude_items`, and returns the top `n` items
	/// sorted by descending score.
	///
	/// # Errors
	///
	/// Returns an error if the similarity metric fails for any item pair.
	pub fn recommend(
		&self,
		user_profile: &UserProfile,
		item_profiles: &[ItemProfile],
		n: usize,
		exclude_items: &HashSet<ItemId>,
	) -> Result<Vec<Recommendation>, ContentBasedError> {
		let mut scored: Vec<Recommendation> = item_profiles
			.iter()
			.filter(|ip| !exclude_items.contains(&ip.id))
			.map(|ip| {
				let score = self
					.similarity
					.compute(&user_profile.features, &ip.features)?;
				Ok(Recommendation {
					item_id: ip.id,
					score,
				})
			})
			.collect::<Result<Vec<_>, ContentBasedError>>()?;

		scored.sort_by(|a, b| {
			b.score
				.partial_cmp(&a.score)
				.unwrap_or(std::cmp::Ordering::Equal)
		});
		scored.truncate(n);

		Ok(scored)
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use crate::similarity::CosineSimilarity;
	use crate::types::FeatureVector;

	use super::*;

	fn make_item_profile(id: u64, values: Vec<f64>) -> ItemProfile {
		ItemProfile {
			id: ItemId(id),
			features: FeatureVector::new(values),
		}
	}

	fn make_user_profile(id: u64, values: Vec<f64>) -> UserProfile {
		UserProfile {
			id: crate::types::UserId(id),
			features: FeatureVector::new(values),
		}
	}

	#[rstest]
	fn recommend_returns_top_n() {
		// Arrange
		let recommender = ContentBasedRecommender::new(Box::new(CosineSimilarity));
		let user = make_user_profile(1, vec![1.0, 0.0]);
		let items = vec![
			make_item_profile(1, vec![1.0, 0.0]), // identical to user
			make_item_profile(2, vec![0.0, 1.0]), // orthogonal
			make_item_profile(3, vec![0.7, 0.3]), // similar
		];

		// Act
		let recs = recommender
			.recommend(&user, &items, 2, &HashSet::new())
			.unwrap();

		// Assert
		assert_eq!(recs.len(), 2);
		assert_eq!(recs[0].item_id, ItemId(1));
		assert_eq!(recs[1].item_id, ItemId(3));
	}

	#[rstest]
	fn recommend_excludes_items() {
		// Arrange
		let recommender = ContentBasedRecommender::new(Box::new(CosineSimilarity));
		let user = make_user_profile(1, vec![1.0, 0.0]);
		let items = vec![
			make_item_profile(1, vec![1.0, 0.0]),
			make_item_profile(2, vec![0.5, 0.5]),
		];
		let exclude: HashSet<ItemId> = [ItemId(1)].into_iter().collect();

		// Act
		let recs = recommender.recommend(&user, &items, 10, &exclude).unwrap();

		// Assert
		assert_eq!(recs.len(), 1);
		assert_eq!(recs[0].item_id, ItemId(2));
	}

	#[rstest]
	fn recommend_empty_items_returns_empty() {
		let recommender = ContentBasedRecommender::new(Box::new(CosineSimilarity));
		let user = make_user_profile(1, vec![1.0, 0.0]);
		let recs = recommender
			.recommend(&user, &[], 5, &HashSet::new())
			.unwrap();
		assert!(recs.is_empty());
	}

	#[rstest]
	fn recommend_scores_sorted_descending() {
		// Arrange
		let recommender = ContentBasedRecommender::new(Box::new(CosineSimilarity));
		let user = make_user_profile(1, vec![1.0, 0.0]);
		let items = vec![
			make_item_profile(1, vec![0.0, 1.0]), // cos = 0
			make_item_profile(2, vec![1.0, 1.0]), // cos ~ 0.707
			make_item_profile(3, vec![1.0, 0.0]), // cos = 1
		];

		// Act
		let recs = recommender
			.recommend(&user, &items, 3, &HashSet::new())
			.unwrap();

		// Assert
		assert_eq!(recs[0].item_id, ItemId(3));
		assert_eq!(recs[1].item_id, ItemId(2));
		assert_eq!(recs[2].item_id, ItemId(1));
		assert!(recs[0].score >= recs[1].score);
		assert!(recs[1].score >= recs[2].score);
	}

	#[rstest]
	fn recommend_full_pipeline() {
		// Arrange — end-to-end test with TF-IDF
		// Uses 5 documents so that shared terms retain non-zero IDF values.
		use crate::profile::{ItemProfileBuilder, UserProfileBuilder};
		use crate::tfidf::TfIdfConfig;
		use crate::types::{Document, UserId};

		let docs = vec![
			Document {
				id: ItemId(1),
				tokens: vec![
					"rust".to_string(),
					"systems".to_string(),
					"programming".to_string(),
				],
			},
			Document {
				id: ItemId(2),
				tokens: vec![
					"python".to_string(),
					"data".to_string(),
					"science".to_string(),
				],
			},
			Document {
				id: ItemId(3),
				tokens: vec![
					"rust".to_string(),
					"web".to_string(),
					"programming".to_string(),
				],
			},
			Document {
				id: ItemId(4),
				tokens: vec![
					"java".to_string(),
					"enterprise".to_string(),
					"backend".to_string(),
				],
			},
			Document {
				id: ItemId(5),
				tokens: vec![
					"go".to_string(),
					"cloud".to_string(),
					"microservices".to_string(),
				],
			},
		];

		let mut builder = ItemProfileBuilder::new(TfIdfConfig::default());
		let item_profiles = builder.build_profiles(&docs).unwrap();

		// User rated item 1 (rust/systems/programming) highly
		let rated_items = vec![(ItemId(1), 5.0)];
		let user_profile =
			UserProfileBuilder::build_profile(UserId(1), &rated_items, &item_profiles).unwrap();

		let recommender = ContentBasedRecommender::new(Box::new(CosineSimilarity));
		let exclude: HashSet<ItemId> = [ItemId(1)].into_iter().collect();

		// Act
		let recs = recommender
			.recommend(&user_profile, &item_profiles, 4, &exclude)
			.unwrap();

		// Assert — item 3 shares "rust" and "programming" with item 1,
		// so it should rank highest among the remaining items.
		assert_eq!(recs.len(), 4);
		assert_eq!(recs[0].item_id, ItemId(3));
		// Items 2, 4, 5 share no terms with item 1 and should all have score 0
		assert!(recs[0].score > 0.0);
	}
}
