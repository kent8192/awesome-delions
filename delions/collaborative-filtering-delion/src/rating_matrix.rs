use std::collections::HashMap;

use crate::types::{ItemId, Rating, UserId};

/// Sparse rating matrix that stores user-item ratings in dual hash maps
/// for efficient lookup by both user and item.
#[derive(Debug, Default)]
pub struct SparseRatingMatrix {
	user_ratings: HashMap<UserId, HashMap<ItemId, f64>>,
	item_ratings: HashMap<ItemId, HashMap<UserId, f64>>,
}

impl SparseRatingMatrix {
	/// Creates a new empty rating matrix.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds or updates a rating in the matrix.
	pub fn add_rating(&mut self, user_id: UserId, item_id: ItemId, value: f64) {
		self.user_ratings
			.entry(user_id)
			.or_default()
			.insert(item_id, value);
		self.item_ratings
			.entry(item_id)
			.or_default()
			.insert(user_id, value);
	}

	/// Returns all ratings by a given user.
	#[must_use]
	pub fn get_user_ratings(&self, user_id: UserId) -> Option<&HashMap<ItemId, f64>> {
		self.user_ratings.get(&user_id)
	}

	/// Returns all ratings for a given item.
	#[must_use]
	pub fn get_item_ratings(&self, item_id: ItemId) -> Option<&HashMap<UserId, f64>> {
		self.item_ratings.get(&item_id)
	}

	/// Returns an iterator over all user IDs in the matrix.
	pub fn users(&self) -> impl Iterator<Item = &UserId> {
		self.user_ratings.keys()
	}

	/// Returns an iterator over all item IDs in the matrix.
	pub fn items(&self) -> impl Iterator<Item = &ItemId> {
		self.item_ratings.keys()
	}

	/// Creates a rating matrix from a slice of ratings.
	#[must_use]
	pub fn from_ratings(ratings: &[Rating]) -> Self {
		let mut matrix = Self::new();
		for r in ratings {
			matrix.add_rating(r.user_id, r.item_id, r.value);
		}
		matrix
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::{fixture, rstest};

	#[fixture]
	fn sample_matrix() -> SparseRatingMatrix {
		let ratings = vec![
			Rating {
				user_id: UserId(1),
				item_id: ItemId(10),
				value: 4.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(20),
				value: 3.0,
			},
			Rating {
				user_id: UserId(2),
				item_id: ItemId(10),
				value: 5.0,
			},
		];
		SparseRatingMatrix::from_ratings(&ratings)
	}

	#[rstest]
	fn add_rating_stores_in_both_maps() {
		// Arrange
		let mut matrix = SparseRatingMatrix::new();

		// Act
		matrix.add_rating(UserId(1), ItemId(10), 4.5);

		// Assert
		assert_eq!(
			matrix.get_user_ratings(UserId(1)).unwrap()[&ItemId(10)],
			4.5
		);
		assert_eq!(
			matrix.get_item_ratings(ItemId(10)).unwrap()[&UserId(1)],
			4.5
		);
	}

	#[rstest]
	fn from_ratings_builds_correct_matrix(sample_matrix: SparseRatingMatrix) {
		// Assert
		let user1 = sample_matrix.get_user_ratings(UserId(1)).unwrap();
		assert_eq!(user1.len(), 2);
		assert_eq!(user1[&ItemId(10)], 4.0);
		assert_eq!(user1[&ItemId(20)], 3.0);

		let item10 = sample_matrix.get_item_ratings(ItemId(10)).unwrap();
		assert_eq!(item10.len(), 2);
		assert_eq!(item10[&UserId(1)], 4.0);
		assert_eq!(item10[&UserId(2)], 5.0);
	}

	#[rstest]
	fn get_user_ratings_returns_none_for_unknown_user(sample_matrix: SparseRatingMatrix) {
		assert!(sample_matrix.get_user_ratings(UserId(999)).is_none());
	}

	#[rstest]
	fn get_item_ratings_returns_none_for_unknown_item(sample_matrix: SparseRatingMatrix) {
		assert!(sample_matrix.get_item_ratings(ItemId(999)).is_none());
	}

	#[rstest]
	fn users_returns_all_user_ids(sample_matrix: SparseRatingMatrix) {
		let mut users: Vec<_> = sample_matrix.users().map(|u| u.0).collect();
		users.sort();
		assert_eq!(users, vec![1, 2]);
	}

	#[rstest]
	fn items_returns_all_item_ids(sample_matrix: SparseRatingMatrix) {
		let mut items: Vec<_> = sample_matrix.items().map(|i| i.0).collect();
		items.sort();
		assert_eq!(items, vec![10, 20]);
	}
}
