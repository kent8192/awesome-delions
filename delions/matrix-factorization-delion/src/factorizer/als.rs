use ndarray::{Array1, Array2};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::error::MatrixFactorizationError;
use crate::factorizer::Factorizer;
use crate::model::LatentFactorModel;
use crate::rating_matrix::RatingMatrix;
use crate::types::ModelConfig;

/// ALS factorizer using alternating least squares.
pub struct AlsFactorizer;

impl Factorizer for AlsFactorizer {
	#[allow(clippy::cast_precision_loss)] // Observed count will not exceed f64 precision
	fn factorize(
		&self,
		matrix: &RatingMatrix,
		config: &ModelConfig,
	) -> Result<LatentFactorModel, MatrixFactorizationError> {
		let n_users = matrix.n_users();
		let n_items = matrix.n_items();
		let n_factors = config.n_factors;
		let lambda = config.regularization;

		let mut rng = StdRng::seed_from_u64(42);

		let mut user_factors =
			Array2::from_shape_fn((n_users, n_factors), |_| 0.01 * rng.random::<f64>());
		let mut item_factors =
			Array2::from_shape_fn((n_items, n_factors), |_| 0.01 * rng.random::<f64>());

		let mut prev_rmse = f64::MAX;

		for _iter in 0..config.max_iterations {
			// Fix item_factors, solve for each user
			solve_factor_block(
				matrix,
				&item_factors,
				&mut user_factors,
				n_factors,
				lambda,
				true,
			);

			// Fix user_factors, solve for each item
			solve_factor_block(
				matrix,
				&user_factors,
				&mut item_factors,
				n_factors,
				lambda,
				false,
			);

			// Compute RMSE on observed entries
			let mut sse = 0.0;
			let mut count = 0usize;
			for i in 0..n_users {
				for j in 0..n_items {
					if matrix.is_observed(i, j) {
						let pred =
							user_factors.row(i).dot(&item_factors.row(j)) + matrix.global_mean;
						let err = matrix.get(i, j) - pred;
						sse += err * err;
						count += 1;
					}
				}
			}
			let rmse = (sse / count as f64).sqrt();

			if (prev_rmse - rmse).abs() < config.tolerance {
				break;
			}
			prev_rmse = rmse;
		}

		Ok(LatentFactorModel {
			user_factors,
			item_factors,
			global_mean: matrix.global_mean,
		})
	}
}

/// Solve one side of the ALS alternation (users or items).
///
/// When `solve_users` is true, solves for user factors using fixed
/// item factors. When false, solves for item factors using fixed
/// user factors.
fn solve_factor_block(
	matrix: &RatingMatrix,
	fixed: &Array2<f64>,
	target: &mut Array2<f64>,
	n_factors: usize,
	lambda: f64,
	solve_users: bool,
) {
	let n_target = target.nrows();
	let n_fixed = fixed.nrows();

	for t in 0..n_target {
		// Collect indices and ratings for entities associated with target t
		let mut indices = Vec::new();
		let mut ratings = Vec::new();

		for f in 0..n_fixed {
			let is_obs = if solve_users {
				matrix.is_observed(t, f)
			} else {
				matrix.is_observed(f, t)
			};
			if is_obs {
				indices.push(f);
				let val = if solve_users {
					matrix.get(t, f)
				} else {
					matrix.get(f, t)
				};
				ratings.push(val - matrix.global_mean);
			}
		}

		if indices.is_empty() {
			continue;
		}

		let n_obs = indices.len();
		let mut sub = Array2::zeros((n_obs, n_factors));
		let mut r = Array1::zeros(n_obs);

		for (idx, &fi) in indices.iter().enumerate() {
			sub.row_mut(idx).assign(&fixed.row(fi));
			r[idx] = ratings[idx];
		}

		// Solve (sub^T * sub + lambda * I) * x = sub^T * r
		let mut ata = sub.t().dot(&sub);
		for k in 0..n_factors {
			ata[[k, k]] += lambda;
		}
		let atb = sub.t().dot(&r);

		if let Ok(x) = solve_linear_system(ata, atb) {
			target.row_mut(t).assign(&x);
		}
	}
}

/// Solve a linear system `Ax = b` using Gaussian elimination
/// with partial pivoting.
fn solve_linear_system(
	mut a: Array2<f64>,
	mut b: Array1<f64>,
) -> Result<Array1<f64>, MatrixFactorizationError> {
	let n = a.nrows();

	// Forward elimination with partial pivoting
	for k in 0..n {
		let mut max_val = a[[k, k]].abs();
		let mut max_row = k;
		for i in (k + 1)..n {
			if a[[i, k]].abs() > max_val {
				max_val = a[[i, k]].abs();
				max_row = i;
			}
		}

		if max_val < 1e-12 {
			return Err(MatrixFactorizationError::SingularMatrix);
		}

		// Swap rows
		if max_row != k {
			for j in 0..n {
				let tmp = a[[k, j]];
				a[[k, j]] = a[[max_row, j]];
				a[[max_row, j]] = tmp;
			}
			let tmp = b[k];
			b[k] = b[max_row];
			b[max_row] = tmp;
		}

		// Eliminate below
		for i in (k + 1)..n {
			let factor = a[[i, k]] / a[[k, k]];
			for j in (k + 1)..n {
				a[[i, j]] -= factor * a[[k, j]];
			}
			b[i] -= factor * b[k];
			a[[i, k]] = 0.0;
		}
	}

	// Back substitution
	let mut x = Array1::zeros(n);
	for k in (0..n).rev() {
		let mut sum = b[k];
		for j in (k + 1)..n {
			sum -= a[[k, j]] * x[j];
		}
		x[k] = sum / a[[k, k]];
	}

	Ok(x)
}

#[cfg(test)]
mod tests {
	use approx::assert_abs_diff_eq;
	use rstest::{fixture, rstest};

	use super::*;
	use crate::rating_matrix::RatingMatrix;
	use crate::types::{ItemId, Rating, UserId};

	#[fixture]
	fn small_ratings() -> Vec<Rating> {
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
	fn als_approximates_observed_ratings(small_ratings: Vec<Rating>) {
		// Arrange
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 3,
			regularization: 0.01,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};

		// Act
		let model = AlsFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: predictions for observed entries should be close
		for rating in &small_ratings {
			let u = matrix.user_to_index(rating.user_id).unwrap();
			let i = matrix.item_to_index(rating.item_id).unwrap();
			let pred = model.predict(u, i);
			assert_abs_diff_eq!(pred, rating.value, epsilon = 1.0);
		}
	}

	#[rstest]
	fn als_predictions_in_reasonable_range(small_ratings: Vec<Rating>) {
		// Arrange
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 2,
			regularization: 0.1,
			max_iterations: 100,
			tolerance: 1e-4,
			..ModelConfig::default()
		};

		// Act
		let model = AlsFactorizer.factorize(&matrix, &config).unwrap();

		// Assert
		let n_users = matrix.n_users();
		let n_items = matrix.n_items();
		for u in 0..n_users {
			for i in 0..n_items {
				let pred = model.predict(u, i);
				assert!(
					(-2.0..=8.0).contains(&pred),
					"prediction {pred} out of reasonable range"
				);
			}
		}
	}

	#[rstest]
	fn solve_linear_system_identity() {
		// Arrange: I * x = b => x = b
		let a = Array2::from_diag(&Array1::from_vec(vec![1.0, 1.0, 1.0]));
		let b = Array1::from_vec(vec![3.0, 5.0, 7.0]);

		// Act
		let x = solve_linear_system(a, b).unwrap();

		// Assert
		assert_abs_diff_eq!(x[0], 3.0, epsilon = 1e-10);
		assert_abs_diff_eq!(x[1], 5.0, epsilon = 1e-10);
		assert_abs_diff_eq!(x[2], 7.0, epsilon = 1e-10);
	}

	#[rstest]
	fn solve_linear_system_2x2() {
		// Arrange: [[2, 1], [1, 3]] * x = [5, 10] => x = [1, 3]
		let a = Array2::from_shape_vec((2, 2), vec![2.0, 1.0, 1.0, 3.0]).unwrap();
		let b = Array1::from_vec(vec![5.0, 10.0]);

		// Act
		let x = solve_linear_system(a, b).unwrap();

		// Assert
		assert_abs_diff_eq!(x[0], 1.0, epsilon = 1e-10);
		assert_abs_diff_eq!(x[1], 3.0, epsilon = 1e-10);
	}

	#[rstest]
	fn als_zero_regularization(small_ratings: Vec<Rating>) {
		// Arrange: lambda = 0.0 means no regularization
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 3,
			regularization: 0.0,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};

		// Act
		let model = AlsFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: should still produce finite predictions
		for u in 0..matrix.n_users() {
			for i in 0..matrix.n_items() {
				let pred = model.predict(u, i);
				assert!(
					pred.is_finite(),
					"prediction should be finite with zero regularization"
				);
			}
		}
	}

	#[rstest]
	fn als_high_regularization(small_ratings: Vec<Rating>) {
		// Arrange: very large lambda pushes factors toward zero
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 3,
			regularization: 1000.0,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};

		// Act
		let model = AlsFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: with extreme regularization, predictions should be close to global_mean
		// because factors are driven near zero
		for u in 0..matrix.n_users() {
			for i in 0..matrix.n_items() {
				let pred = model.predict(u, i);
				assert_abs_diff_eq!(pred, matrix.global_mean, epsilon = 1.0);
			}
		}
	}
}
