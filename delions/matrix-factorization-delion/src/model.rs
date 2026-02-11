use ndarray::Array2;

/// Latent factor model consisting of user and item factor matrices.
pub struct LatentFactorModel {
	/// User factor matrix with shape `(n_users, n_factors)`.
	pub user_factors: Array2<f64>,
	/// Item factor matrix with shape `(n_items, n_factors)`.
	pub item_factors: Array2<f64>,
	/// Global mean of observed ratings.
	pub global_mean: f64,
}

impl LatentFactorModel {
	/// Predict the rating for a given user-item pair.
	///
	/// Computes the dot product of the user and item factor vectors
	/// plus the global mean.
	#[must_use]
	pub fn predict(&self, user_idx: usize, item_idx: usize) -> f64 {
		let user_row = self.user_factors.row(user_idx);
		let item_row = self.item_factors.row(item_idx);
		user_row.dot(&item_row) + self.global_mean
	}

	/// Returns the number of latent factors.
	#[must_use]
	pub fn n_factors(&self) -> usize {
		self.user_factors.ncols()
	}
}

#[cfg(test)]
mod tests {
	use approx::assert_abs_diff_eq;
	use ndarray::array;
	use rstest::rstest;

	use super::*;

	#[rstest]
	fn predict_with_known_factors() {
		// Arrange
		let model = LatentFactorModel {
			user_factors: array![[1.0, 2.0], [0.5, -1.0]],
			item_factors: array![[3.0, 1.0], [-1.0, 2.0]],
			global_mean: 3.0,
		};

		// Act & Assert
		// user 0 x item 0: (1*3 + 2*1) + 3 = 8.0
		assert_abs_diff_eq!(model.predict(0, 0), 8.0);
		// user 0 x item 1: (1*-1 + 2*2) + 3 = 6.0
		assert_abs_diff_eq!(model.predict(0, 1), 6.0);
		// user 1 x item 0: (0.5*3 + -1*1) + 3 = 3.5
		assert_abs_diff_eq!(model.predict(1, 0), 3.5);
		// user 1 x item 1: (0.5*-1 + -1*2) + 3 = 0.5
		assert_abs_diff_eq!(model.predict(1, 1), 0.5);
	}

	#[rstest]
	fn n_factors_returns_correct_count() {
		let model = LatentFactorModel {
			user_factors: array![[1.0, 2.0, 3.0]],
			item_factors: array![[4.0, 5.0, 6.0]],
			global_mean: 0.0,
		};
		assert_eq!(model.n_factors(), 3);
	}

	#[rstest]
	fn predict_adds_global_mean() {
		// Arrange: zero factors so dot product is 0; global_mean drives prediction
		let model = LatentFactorModel {
			user_factors: array![[0.0, 0.0]],
			item_factors: array![[0.0, 0.0]],
			global_mean: 3.5,
		};

		// Act
		let pred = model.predict(0, 0);

		// Assert: 0.0 + 3.5 = 3.5
		assert_abs_diff_eq!(pred, 3.5);
	}

	#[rstest]
	fn predict_with_large_factor_count() {
		// Arrange: 10 latent factors
		let user_factors = Array2::from_elem((2, 10), 0.1);
		let item_factors = Array2::from_elem((3, 10), 0.2);
		let model = LatentFactorModel {
			user_factors,
			item_factors,
			global_mean: 1.0,
		};

		// Act: dot product of [0.1]*10 . [0.2]*10 = 10 * 0.02 = 0.2; + 1.0 = 1.2
		let pred = model.predict(0, 0);

		// Assert
		assert_abs_diff_eq!(pred, 1.2, epsilon = 1e-10);
		assert_eq!(model.n_factors(), 10);
	}
}
