use crate::error::CollaborativeFilteringError;
use crate::similarity::Similarity;

/// Cosine similarity metric.
///
/// Computes `dot(a,b) / (||a|| * ||b||)`.
pub struct CosineSimilarity;

impl Similarity for CosineSimilarity {
	fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, CollaborativeFilteringError> {
		if a.is_empty() || b.is_empty() {
			return Err(CollaborativeFilteringError::EmptyVectors);
		}

		let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
		let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
		let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

		if norm_a == 0.0 || norm_b == 0.0 {
			return Err(CollaborativeFilteringError::ZeroNorm);
		}

		Ok(dot / (norm_a * norm_b))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;

	#[rstest]
	fn identical_vectors_have_similarity_one() {
		// Arrange
		let sim = CosineSimilarity;
		let a = [1.0, 2.0, 3.0];

		// Act
		let result = sim.compute(&a, &a).unwrap();

		// Assert
		assert!((result - 1.0).abs() < 1e-10);
	}

	#[rstest]
	fn orthogonal_vectors_have_similarity_zero() {
		// Arrange
		let sim = CosineSimilarity;
		let a = [1.0, 0.0];
		let b = [0.0, 1.0];

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!(result.abs() < 1e-10);
	}

	#[rstest]
	fn known_cosine_value() {
		// Arrange
		let sim = CosineSimilarity;
		let a = [1.0, 2.0, 3.0];
		let b = [4.0, 5.0, 6.0];
		// dot = 32, ||a|| = sqrt(14), ||b|| = sqrt(77)
		let expected = 32.0 / (14.0_f64.sqrt() * 77.0_f64.sqrt());

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - expected).abs() < 1e-10);
	}

	#[rstest]
	fn empty_vectors_return_error() {
		let sim = CosineSimilarity;
		let result = sim.compute(&[], &[1.0]);
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::EmptyVectors)
		));
	}

	#[rstest]
	fn zero_norm_returns_error() {
		let sim = CosineSimilarity;
		let result = sim.compute(&[0.0, 0.0], &[1.0, 2.0]);
		assert!(matches!(result, Err(CollaborativeFilteringError::ZeroNorm)));
	}

	#[rstest]
	fn different_length_vectors() {
		// Arrange: dot uses zip (min length), but norms use full vectors
		let sim = CosineSimilarity;
		let a = [1.0, 0.0];
		let b = [1.0, 0.0, 99.0];
		// dot = 1*1 + 0*0 = 1 (zip: 2 elements)
		// norm_a = sqrt(1+0) = 1 (all of a)
		// norm_b = sqrt(1+0+9801) = sqrt(9802) (all of b)
		let expected = 1.0 / 9802.0_f64.sqrt();

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - expected).abs() < 1e-10);
	}

	#[rstest]
	fn single_element_vectors() {
		// Arrange
		let sim = CosineSimilarity;
		let a = [3.0];
		let b = [5.0];
		// dot=15, ||a||=3, ||b||=5 → 15/15 = 1.0

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - 1.0).abs() < 1e-10);
	}

	#[rstest]
	fn negative_values_in_vectors() {
		// Arrange
		let sim = CosineSimilarity;
		let a = [1.0, -1.0];
		let b = [-1.0, 1.0];
		// dot = -1 + -1 = -2, ||a|| = sqrt(2), ||b|| = sqrt(2) → -2/2 = -1.0

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - (-1.0)).abs() < 1e-10);
	}
}
