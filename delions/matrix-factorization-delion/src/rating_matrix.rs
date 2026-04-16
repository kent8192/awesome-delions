use std::collections::HashMap;

use ndarray::Array2;

use crate::error::MatrixFactorizationError;
use crate::types::{ItemId, Rating, UserId};

/// Dense rating matrix with observed entry tracking and index mappings.
pub struct RatingMatrix {
	/// Rating values (0.0 for unobserved entries).
	pub values: Array2<f64>,
	/// Mask indicating which entries have been observed.
	pub observed: Array2<bool>,
	/// Maps `UserId` to matrix row index.
	user_index: HashMap<UserId, usize>,
	/// Maps `ItemId` to matrix column index.
	item_index: HashMap<ItemId, usize>,
	/// Maps row index back to `UserId`.
	index_user: Vec<UserId>,
	/// Maps column index back to `ItemId`.
	index_item: Vec<ItemId>,
	/// Mean of all observed ratings.
	pub global_mean: f64,
}

impl RatingMatrix {
	/// Build a rating matrix from a slice of ratings.
	///
	/// # Errors
	///
	/// Returns `EmptyRatings` if the input slice is empty.
	#[allow(clippy::cast_precision_loss)] // Rating count will not exceed f64 precision
	pub fn from_ratings(ratings: &[Rating]) -> Result<Self, MatrixFactorizationError> {
		if ratings.is_empty() {
			return Err(MatrixFactorizationError::EmptyRatings);
		}

		let mut user_index = HashMap::new();
		let mut item_index = HashMap::new();
		let mut index_user = Vec::new();
		let mut index_item = Vec::new();

		for rating in ratings {
			if let std::collections::hash_map::Entry::Vacant(e) = user_index.entry(rating.user_id) {
				let idx = index_user.len();
				e.insert(idx);
				index_user.push(rating.user_id);
			}
			if let std::collections::hash_map::Entry::Vacant(e) = item_index.entry(rating.item_id) {
				let idx = index_item.len();
				e.insert(idx);
				index_item.push(rating.item_id);
			}
		}

		let n_users = index_user.len();
		let n_items = index_item.len();
		let mut values = Array2::zeros((n_users, n_items));
		let mut observed = Array2::from_elem((n_users, n_items), false);
		let mut sum = 0.0;
		let mut count = 0usize;

		for rating in ratings {
			let u = user_index[&rating.user_id];
			let i = item_index[&rating.item_id];
			values[[u, i]] = rating.value;
			observed[[u, i]] = true;
			sum += rating.value;
			count += 1;
		}

		let global_mean = sum / count as f64;

		Ok(Self {
			values,
			observed,
			user_index,
			item_index,
			index_user,
			index_item,
			global_mean,
		})
	}

	/// Returns the number of users.
	#[must_use]
	pub fn n_users(&self) -> usize {
		self.index_user.len()
	}

	/// Returns the number of items.
	#[must_use]
	pub fn n_items(&self) -> usize {
		self.index_item.len()
	}

	/// Returns the rating value at the given indices.
	#[must_use]
	pub fn get(&self, user: usize, item: usize) -> f64 {
		self.values[[user, item]]
	}

	/// Returns whether the entry at the given indices has been observed.
	#[must_use]
	pub fn is_observed(&self, user: usize, item: usize) -> bool {
		self.observed[[user, item]]
	}

	/// Maps a `UserId` to its matrix row index.
	#[must_use]
	pub fn user_to_index(&self, user_id: UserId) -> Option<usize> {
		self.user_index.get(&user_id).copied()
	}

	/// Maps an `ItemId` to its matrix column index.
	#[must_use]
	pub fn item_to_index(&self, item_id: ItemId) -> Option<usize> {
		self.item_index.get(&item_id).copied()
	}

	/// Maps a row index back to its `UserId`.
	#[must_use]
	pub fn index_to_user(&self, index: usize) -> Option<UserId> {
		self.index_user.get(index).copied()
	}

	/// Maps a column index back to its `ItemId`.
	#[must_use]
	pub fn index_to_item(&self, index: usize) -> Option<ItemId> {
		self.index_item.get(index).copied()
	}
}

#[cfg(test)]
mod tests {
	use approx::assert_abs_diff_eq;
	use rstest::{fixture, rstest};

	use super::*;

	#[fixture]
	fn sample_ratings() -> Vec<Rating> {
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
				user_id: UserId(1),
				item_id: ItemId(0),
				value: 4.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(2),
				value: 2.0,
			},
		]
	}

	#[rstest]
	fn from_ratings_builds_correct_matrix(sample_ratings: Vec<Rating>) {
		// Act
		let matrix = RatingMatrix::from_ratings(&sample_ratings).unwrap();

		// Assert
		assert_eq!(matrix.n_users(), 2);
		assert_eq!(matrix.n_items(), 3);
		assert_eq!(matrix.get(0, 0), 5.0);
		assert_eq!(matrix.get(0, 1), 3.0);
		assert_eq!(matrix.get(1, 0), 4.0);
		assert_eq!(matrix.get(1, 2), 2.0);
		assert!(matrix.is_observed(0, 0));
		assert!(matrix.is_observed(1, 2));
		assert!(!matrix.is_observed(0, 2));
		assert!(!matrix.is_observed(1, 1));
	}

	#[rstest]
	fn from_ratings_empty_returns_error() {
		let result = RatingMatrix::from_ratings(&[]);
		assert!(matches!(
			result,
			Err(MatrixFactorizationError::EmptyRatings)
		));
	}

	#[rstest]
	fn index_lookups_roundtrip(sample_ratings: Vec<Rating>) {
		// Arrange
		let matrix = RatingMatrix::from_ratings(&sample_ratings).unwrap();

		// Act & Assert
		let user_idx = matrix.user_to_index(UserId(0)).unwrap();
		assert_eq!(matrix.index_to_user(user_idx), Some(UserId(0)));

		let item_idx = matrix.item_to_index(ItemId(2)).unwrap();
		assert_eq!(matrix.index_to_item(item_idx), Some(ItemId(2)));

		assert_eq!(matrix.user_to_index(UserId(99)), None);
		assert_eq!(matrix.item_to_index(ItemId(99)), None);
		assert_eq!(matrix.index_to_user(99), None);
		assert_eq!(matrix.index_to_item(99), None);
	}

	#[rstest]
	fn global_mean_calculation(sample_ratings: Vec<Rating>) {
		// Act
		let matrix = RatingMatrix::from_ratings(&sample_ratings).unwrap();

		// Assert: mean of [5.0, 3.0, 4.0, 2.0] = 14.0 / 4 = 3.5
		assert_abs_diff_eq!(matrix.global_mean, 3.5);
	}

	#[rstest]
	fn single_user_single_item_matrix() {
		// Arrange
		let ratings = vec![Rating {
			user_id: UserId(7),
			item_id: ItemId(3),
			value: 4.0,
		}];

		// Act
		let matrix = RatingMatrix::from_ratings(&ratings).unwrap();

		// Assert
		assert_eq!(matrix.n_users(), 1);
		assert_eq!(matrix.n_items(), 1);
		assert_eq!(matrix.get(0, 0), 4.0);
		assert!(matrix.is_observed(0, 0));
		assert_abs_diff_eq!(matrix.global_mean, 4.0);
		assert_eq!(matrix.user_to_index(UserId(7)), Some(0));
		assert_eq!(matrix.item_to_index(ItemId(3)), Some(0));
		assert_eq!(matrix.index_to_user(0), Some(UserId(7)));
		assert_eq!(matrix.index_to_item(0), Some(ItemId(3)));
	}

	#[rstest]
	fn index_to_user_out_of_range_returns_none(sample_ratings: Vec<Rating>) {
		// Arrange
		let matrix = RatingMatrix::from_ratings(&sample_ratings).unwrap();

		// Act & Assert
		assert_eq!(matrix.index_to_user(matrix.n_users()), None);
		assert_eq!(matrix.index_to_user(100), None);
	}

	#[rstest]
	fn index_to_item_out_of_range_returns_none(sample_ratings: Vec<Rating>) {
		// Arrange
		let matrix = RatingMatrix::from_ratings(&sample_ratings).unwrap();

		// Act & Assert
		assert_eq!(matrix.index_to_item(matrix.n_items()), None);
		assert_eq!(matrix.index_to_item(100), None);
	}
}
