use crate::error::ContentBasedError;
use crate::profile::item::ItemProfile;
use crate::types::{FeatureVector, ItemId, UserId};

/// A user's preference profile built from rated item features.
#[derive(Debug, Clone)]
pub struct UserProfile {
	pub id: UserId,
	pub features: FeatureVector,
}

/// Builds user profiles as weighted averages of rated item feature vectors.
pub struct UserProfileBuilder;

impl UserProfileBuilder {
	/// Builds a user profile from their rated items.
	///
	/// The profile is the weighted average of rated item feature vectors,
	/// where the weight is the rating value. The result is normalized by
	/// the sum of weights.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::UserNotFound` if `rated_items` is empty,
	/// or `ContentBasedError::ItemNotFound` if a rated item is not in
	/// `item_profiles`.
	pub fn build_profile(
		user_id: UserId,
		rated_items: &[(ItemId, f64)],
		item_profiles: &[ItemProfile],
	) -> Result<UserProfile, ContentBasedError> {
		if rated_items.is_empty() {
			return Err(ContentBasedError::UserNotFound(user_id));
		}

		let dimension = item_profiles.first().map_or(0, |p| p.features.dimension());

		let mut weighted_sum = vec![0.0_f64; dimension];
		let mut weight_total = 0.0_f64;

		for &(item_id, rating) in rated_items {
			let profile = item_profiles
				.iter()
				.find(|p| p.id == item_id)
				.ok_or(ContentBasedError::ItemNotFound(item_id))?;

			for (i, &val) in profile.features.values.iter().enumerate() {
				weighted_sum[i] += val * rating;
			}
			weight_total += rating;
		}

		if weight_total > 0.0 {
			for val in &mut weighted_sum {
				*val /= weight_total;
			}
		}

		Ok(UserProfile {
			id: user_id,
			features: FeatureVector::new(weighted_sum),
		})
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	fn make_item_profile(id: u64, values: Vec<f64>) -> ItemProfile {
		ItemProfile {
			id: ItemId(id),
			features: FeatureVector::new(values),
		}
	}

	#[rstest]
	fn build_profile_weighted_average() {
		// Arrange
		let profiles = vec![
			make_item_profile(1, vec![1.0, 0.0]),
			make_item_profile(2, vec![0.0, 1.0]),
		];
		let rated_items = vec![(ItemId(1), 3.0), (ItemId(2), 1.0)];

		// Act
		let user_profile =
			UserProfileBuilder::build_profile(UserId(10), &rated_items, &profiles).unwrap();

		// Assert — weighted avg: (3*[1,0] + 1*[0,1]) / 4 = [0.75, 0.25]
		assert_eq!(user_profile.id, UserId(10));
		assert!((user_profile.features.values[0] - 0.75).abs() < 1e-10);
		assert!((user_profile.features.values[1] - 0.25).abs() < 1e-10);
	}

	#[rstest]
	fn build_profile_single_item() {
		// Arrange
		let profiles = vec![make_item_profile(1, vec![2.0, 4.0])];
		let rated_items = vec![(ItemId(1), 5.0)];

		// Act
		let user_profile =
			UserProfileBuilder::build_profile(UserId(1), &rated_items, &profiles).unwrap();

		// Assert — single item: [2,4] * 5 / 5 = [2, 4]
		assert!((user_profile.features.values[0] - 2.0).abs() < 1e-10);
		assert!((user_profile.features.values[1] - 4.0).abs() < 1e-10);
	}

	#[rstest]
	fn build_profile_empty_ratings_returns_error() {
		let profiles = vec![make_item_profile(1, vec![1.0])];
		let result = UserProfileBuilder::build_profile(UserId(1), &[], &profiles);
		assert!(matches!(result, Err(ContentBasedError::UserNotFound(_))));
	}

	#[rstest]
	fn build_profile_single_item_weighted_average_equals_original() {
		// Arrange
		let profiles = vec![make_item_profile(1, vec![3.0, 7.0, 1.0])];
		let rated_items = vec![(ItemId(1), 4.0)];

		// Act
		let user_profile =
			UserProfileBuilder::build_profile(UserId(1), &rated_items, &profiles).unwrap();

		// Assert — single item: [3,7,1] * 4 / 4 = [3, 7, 1]
		assert!((user_profile.features.values[0] - 3.0).abs() < 1e-10);
		assert!((user_profile.features.values[1] - 7.0).abs() < 1e-10);
		assert!((user_profile.features.values[2] - 1.0).abs() < 1e-10);
	}

	#[rstest]
	fn build_profile_high_vs_low_rating_weight_difference() {
		// Arrange
		let profiles = vec![
			make_item_profile(1, vec![1.0, 0.0]),
			make_item_profile(2, vec![0.0, 1.0]),
		];
		let rated_items_high_first = vec![(ItemId(1), 9.0), (ItemId(2), 1.0)];
		let rated_items_high_second = vec![(ItemId(1), 1.0), (ItemId(2), 9.0)];

		// Act
		let profile_high_first =
			UserProfileBuilder::build_profile(UserId(1), &rated_items_high_first, &profiles)
				.unwrap();
		let profile_high_second =
			UserProfileBuilder::build_profile(UserId(2), &rated_items_high_second, &profiles)
				.unwrap();

		// Assert — high rating on item 1 should pull features toward [1,0]
		assert!(profile_high_first.features.values[0] > profile_high_first.features.values[1]);
		// High rating on item 2 should pull features toward [0,1]
		assert!(profile_high_second.features.values[1] > profile_high_second.features.values[0]);
		// Exact values: high_first = [0.9, 0.1], high_second = [0.1, 0.9]
		assert!((profile_high_first.features.values[0] - 0.9).abs() < 1e-10);
		assert!((profile_high_second.features.values[1] - 0.9).abs() < 1e-10);
	}

	#[rstest]
	fn build_profile_missing_item_returns_error() {
		// Arrange
		let profiles = vec![make_item_profile(1, vec![1.0])];
		let rated_items = vec![(ItemId(99), 5.0)];

		// Act
		let result = UserProfileBuilder::build_profile(UserId(1), &rated_items, &profiles);

		// Assert
		assert!(matches!(
			result,
			Err(ContentBasedError::ItemNotFound(ItemId(99)))
		));
	}
}
