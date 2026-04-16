use ndarray::{Array1, Array2};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::error::MatrixFactorizationError;
use crate::factorizer::Factorizer;
use crate::model::LatentFactorModel;
use crate::rating_matrix::RatingMatrix;
use crate::types::ModelConfig;

/// SVD factorizer using truncated SVD via power iteration.
pub struct SvdFactorizer;

impl Factorizer for SvdFactorizer {
	fn factorize(
		&self,
		matrix: &RatingMatrix,
		config: &ModelConfig,
	) -> Result<LatentFactorModel, MatrixFactorizationError> {
		let n_users = matrix.n_users();
		let n_items = matrix.n_items();
		let n_factors = config.n_factors;

		// Mean-center observed ratings
		let mut a = matrix.values.clone();
		for i in 0..n_users {
			for j in 0..n_items {
				if matrix.is_observed(i, j) {
					a[[i, j]] -= matrix.global_mean;
				}
			}
		}

		let mut rng = StdRng::seed_from_u64(42);
		let mut u_vecs = Array2::zeros((n_users, n_factors));
		let mut v_vecs = Array2::zeros((n_items, n_factors));
		let mut sigmas = Vec::with_capacity(n_factors);

		for k in 0..n_factors {
			let (sigma, u, v) = power_iteration(&a, &matrix.observed, &mut rng, config);

			u_vecs.column_mut(k).assign(&u);
			v_vecs.column_mut(k).assign(&v);
			sigmas.push(sigma);

			// Deflate: A = A - sigma * u * v^T (observed entries only)
			for i in 0..n_users {
				for j in 0..n_items {
					if matrix.is_observed(i, j) {
						a[[i, j]] -= sigma * u[i] * v[j];
					}
				}
			}
		}

		// Build model: user_factors = U * sqrt(Sigma),
		// item_factors = V * sqrt(Sigma)
		let mut user_factors = Array2::zeros((n_users, n_factors));
		let mut item_factors = Array2::zeros((n_items, n_factors));

		for k in 0..n_factors {
			let sqrt_sigma = sigmas[k].max(0.0).sqrt();
			for i in 0..n_users {
				user_factors[[i, k]] = u_vecs[[i, k]] * sqrt_sigma;
			}
			for j in 0..n_items {
				item_factors[[j, k]] = v_vecs[[j, k]] * sqrt_sigma;
			}
		}

		Ok(LatentFactorModel {
			user_factors,
			item_factors,
			global_mean: matrix.global_mean,
		})
	}
}

/// Extract one singular triplet (sigma, u, v) via power iteration.
fn power_iteration(
	a: &Array2<f64>,
	observed: &Array2<bool>,
	rng: &mut StdRng,
	config: &ModelConfig,
) -> (f64, Array1<f64>, Array1<f64>) {
	let n_users = a.nrows();
	let n_items = a.ncols();

	// Initialize random v vector
	let mut v = Array1::from_shape_fn(n_items, |_| rng.random::<f64>());
	let v_norm = v.dot(&v).sqrt();
	if v_norm > 0.0 {
		v /= v_norm;
	}

	let mut sigma: f64 = 0.0;

	for _iter in 0..config.max_iterations {
		// u = A * v (observed entries only)
		let mut u = Array1::<f64>::zeros(n_users);
		for i in 0..n_users {
			for j in 0..n_items {
				if observed[[i, j]] {
					u[i] += a[[i, j]] * v[j];
				}
			}
		}

		let u_norm = u.dot(&u).sqrt();
		if u_norm < 1e-12 {
			break;
		}
		u /= u_norm;

		// v_new = A^T * u (observed entries only)
		let mut v_new = Array1::<f64>::zeros(n_items);
		for j in 0..n_items {
			for i in 0..n_users {
				if observed[[i, j]] {
					v_new[j] += a[[i, j]] * u[i];
				}
			}
		}

		let new_sigma = v_new.dot(&v_new).sqrt();
		if new_sigma < 1e-12 {
			sigma = new_sigma;
			v = v_new;
			break;
		}
		v_new /= new_sigma;

		let converged = (new_sigma - sigma).abs() < config.tolerance * sigma.abs().max(1.0);
		sigma = new_sigma;
		v = v_new;

		if converged {
			break;
		}
	}

	// Compute final u for this singular value
	let mut u = Array1::<f64>::zeros(n_users);
	for i in 0..n_users {
		for j in 0..n_items {
			if observed[[i, j]] {
				u[i] += a[[i, j]] * v[j];
			}
		}
	}
	let u_norm = u.dot(&u).sqrt();
	if u_norm > 1e-12 {
		u /= u_norm;
	}

	(sigma, u, v)
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
	fn svd_approximates_observed_ratings(small_ratings: Vec<Rating>) {
		// Arrange
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 3,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};

		// Act
		let model = SvdFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: predictions for observed entries should be close
		for rating in &small_ratings {
			let u = matrix.user_to_index(rating.user_id).unwrap();
			let i = matrix.item_to_index(rating.item_id).unwrap();
			let pred = model.predict(u, i);
			assert_abs_diff_eq!(pred, rating.value, epsilon = 1.5);
		}
	}

	#[rstest]
	fn svd_predictions_in_reasonable_range(small_ratings: Vec<Rating>) {
		// Arrange
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 2,
			max_iterations: 100,
			tolerance: 1e-4,
			..ModelConfig::default()
		};

		// Act
		let model = SvdFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: unrated predictions should be in a plausible range
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
	fn svd_near_identity_input() {
		// Arrange: uniform ratings where all values are the same
		let ratings = vec![
			Rating {
				user_id: UserId(0),
				item_id: ItemId(0),
				value: 3.0,
			},
			Rating {
				user_id: UserId(0),
				item_id: ItemId(1),
				value: 3.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(0),
				value: 3.0,
			},
			Rating {
				user_id: UserId(1),
				item_id: ItemId(1),
				value: 3.0,
			},
		];
		let matrix = RatingMatrix::from_ratings(&ratings).unwrap();
		let config = ModelConfig {
			n_factors: 2,
			max_iterations: 200,
			tolerance: 1e-6,
			..ModelConfig::default()
		};

		// Act
		let model = SvdFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: predictions should all be close to 3.0
		for u in 0..2 {
			for i in 0..2 {
				assert_abs_diff_eq!(model.predict(u, i), 3.0, epsilon = 0.5);
			}
		}
	}

	#[rstest]
	fn svd_n_factors_larger_than_dimensions(small_ratings: Vec<Rating>) {
		// Arrange: n_factors (10) > min(n_users=3, n_items=4)
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 10,
			max_iterations: 100,
			tolerance: 1e-4,
			..ModelConfig::default()
		};

		// Act: should not panic even with oversized factors
		let model = SvdFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: model dimensions are correct
		assert_eq!(model.n_factors(), 10);
		assert_eq!(model.user_factors.nrows(), matrix.n_users());
		assert_eq!(model.item_factors.nrows(), matrix.n_items());
	}

	#[rstest]
	fn svd_few_iterations_convergence(small_ratings: Vec<Rating>) {
		// Arrange: very few iterations (1)
		let matrix = RatingMatrix::from_ratings(&small_ratings).unwrap();
		let config = ModelConfig {
			n_factors: 2,
			max_iterations: 1,
			tolerance: 1e-4,
			..ModelConfig::default()
		};

		// Act: should still produce a model without errors
		let model = SvdFactorizer.factorize(&matrix, &config).unwrap();

		// Assert: model is valid even with minimal iterations
		// NOTE: predictions may not be accurate with 1 iteration, but should be finite
		for u in 0..matrix.n_users() {
			for i in 0..matrix.n_items() {
				let pred = model.predict(u, i);
				assert!(pred.is_finite(), "prediction should be finite");
			}
		}
	}
}
