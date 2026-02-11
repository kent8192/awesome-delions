use crate::error::CollaborativeFilteringError;
use crate::similarity::Similarity;

/// Pearson correlation coefficient.
///
/// Computes `cov(a,b) / (std(a) * std(b))` where values are centered by mean.
pub struct PearsonCorrelation;

impl Similarity for PearsonCorrelation {
	fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, CollaborativeFilteringError> {
		if a.is_empty() || b.is_empty() {
			return Err(CollaborativeFilteringError::EmptyVectors);
		}

		// Precision loss is acceptable: vector lengths are small in practice
		#[allow(clippy::cast_precision_loss)]
		let n = a.len() as f64;
		let mean_a: f64 = a.iter().sum::<f64>() / n;
		let mean_b: f64 = b.iter().sum::<f64>() / n;

		let mut cov = 0.0;
		let mut var_a = 0.0;
		let mut var_b = 0.0;

		for (x, y) in a.iter().zip(b.iter()) {
			let da = x - mean_a;
			let db = y - mean_b;
			cov += da * db;
			var_a += da * da;
			var_b += db * db;
		}

		let std_a = var_a.sqrt();
		let std_b = var_b.sqrt();

		if std_a == 0.0 || std_b == 0.0 {
			return Err(CollaborativeFilteringError::ZeroNorm);
		}

		Ok(cov / (std_a * std_b))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;

	#[rstest]
	fn perfect_positive_correlation() {
		// Arrange
		let sim = PearsonCorrelation;
		let a = [1.0, 2.0, 3.0, 4.0, 5.0];
		let b = [2.0, 4.0, 6.0, 8.0, 10.0];

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - 1.0).abs() < 1e-10);
	}

	#[rstest]
	fn perfect_negative_correlation() {
		// Arrange
		let sim = PearsonCorrelation;
		let a = [1.0, 2.0, 3.0, 4.0, 5.0];
		let b = [10.0, 8.0, 6.0, 4.0, 2.0];

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - (-1.0)).abs() < 1e-10);
	}

	#[rstest]
	fn known_pearson_value() {
		// Arrange
		let sim = PearsonCorrelation;
		let a = [1.0, 2.0, 3.0];
		let b = [1.0, 3.0, 2.0];
		// mean_a=2, mean_b=2
		// cov = (-1)(-1) + (0)(1) + (1)(0) = 1
		// std_a = sqrt(2), std_b = sqrt(2)
		// r = 1/2 = 0.5
		let expected = 0.5;

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - expected).abs() < 1e-10);
	}

	#[rstest]
	fn empty_vectors_return_error() {
		let sim = PearsonCorrelation;
		let result = sim.compute(&[], &[1.0]);
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::EmptyVectors)
		));
	}

	#[rstest]
	fn constant_vector_returns_zero_norm_error() {
		// Arrange: constant vector has zero standard deviation
		let sim = PearsonCorrelation;
		let a = [3.0, 3.0, 3.0];
		let b = [1.0, 2.0, 3.0];

		// Act
		let result = sim.compute(&a, &b);

		// Assert
		assert!(matches!(result, Err(CollaborativeFilteringError::ZeroNorm)));
	}

	#[rstest]
	fn different_length_vectors() {
		// Arrange: mean uses a.len() for n, sum uses full vector,
		// but zip in the loop processes only min(len) pairs.
		let sim = PearsonCorrelation;
		let a = [1.0, 2.0, 3.0];
		let b = [2.0, 4.0, 6.0, 100.0, 200.0];
		// n=3, mean_a=2.0, mean_b=312/3=104.0
		// cov = (-1)(-102) + (0)(-100) + (1)(-98) = 4
		// var_a = 2, var_b = 30008
		// r = 4 / sqrt(2*30008) = 4 / sqrt(60016)
		let expected = 4.0 / 60016.0_f64.sqrt();

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - expected).abs() < 1e-10);
	}

	#[rstest]
	fn two_element_vectors() {
		// Arrange: minimum valid length for meaningful Pearson
		let sim = PearsonCorrelation;
		let a = [1.0, 3.0];
		let b = [2.0, 6.0];
		// mean_a=2, mean_b=4, cov=(-1)(-2)+(1)(2)=4, std_a=sqrt(2), std_b=sqrt(8)
		// r = 4/sqrt(16) = 4/4 = 1.0

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - 1.0).abs() < 1e-10);
	}
}
