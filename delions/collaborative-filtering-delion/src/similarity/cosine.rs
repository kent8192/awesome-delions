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
}
