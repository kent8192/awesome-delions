use crate::error::ContentBasedError;
use crate::similarity::SimilarityMetric;
use crate::types::FeatureVector;

/// Computes cosine similarity between two feature vectors.
///
/// `cos(a, b) = dot(a, b) / (||a|| * ||b||)`
///
/// Returns 0.0 when either vector has zero norm.
#[derive(Debug)]
pub struct CosineSimilarity;

impl SimilarityMetric for CosineSimilarity {
	fn compute(&self, a: &FeatureVector, b: &FeatureVector) -> Result<f64, ContentBasedError> {
		if a.values.is_empty() || b.values.is_empty() {
			return Err(ContentBasedError::EmptyFeatureVector);
		}
		let dot = a.dot(b)?;
		let norm_a = a.norm();
		let norm_b = b.norm();
		if norm_a == 0.0 || norm_b == 0.0 {
			return Ok(0.0);
		}
		Ok(dot / (norm_a * norm_b))
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	fn cosine_identical_vectors() {
		let sim = CosineSimilarity;
		let v = FeatureVector::new(vec![1.0, 2.0, 3.0]);
		let result = sim.compute(&v, &v).unwrap();
		assert!((result - 1.0).abs() < 1e-10);
	}

	#[rstest]
	fn cosine_orthogonal_vectors() {
		let sim = CosineSimilarity;
		let a = FeatureVector::new(vec![1.0, 0.0]);
		let b = FeatureVector::new(vec![0.0, 1.0]);
		let result = sim.compute(&a, &b).unwrap();
		assert!((result - 0.0).abs() < 1e-10);
	}

	#[rstest]
	fn cosine_opposite_vectors() {
		let sim = CosineSimilarity;
		let a = FeatureVector::new(vec![1.0, 0.0]);
		let b = FeatureVector::new(vec![-1.0, 0.0]);
		let result = sim.compute(&a, &b).unwrap();
		assert!((result - (-1.0)).abs() < 1e-10);
	}

	#[rstest]
	fn cosine_zero_vector_returns_zero() {
		let sim = CosineSimilarity;
		let a = FeatureVector::new(vec![0.0, 0.0]);
		let b = FeatureVector::new(vec![1.0, 2.0]);
		let result = sim.compute(&a, &b).unwrap();
		assert_eq!(result, 0.0);
	}

	#[rstest]
	fn cosine_empty_vector_returns_error() {
		let sim = CosineSimilarity;
		let a = FeatureVector::new(vec![]);
		let b = FeatureVector::new(vec![1.0]);
		let result = sim.compute(&a, &b);
		assert!(matches!(result, Err(ContentBasedError::EmptyFeatureVector)));
	}

	#[rstest]
	fn cosine_dimension_mismatch() {
		let sim = CosineSimilarity;
		let a = FeatureVector::new(vec![1.0, 2.0]);
		let b = FeatureVector::new(vec![1.0, 2.0, 3.0]);
		let result = sim.compute(&a, &b);
		assert!(matches!(
			result,
			Err(ContentBasedError::DimensionMismatch { .. })
		));
	}

	#[rstest]
	fn cosine_both_zero_vectors_returns_zero() {
		let sim = CosineSimilarity;
		let a = FeatureVector::new(vec![0.0, 0.0, 0.0]);
		let b = FeatureVector::new(vec![0.0, 0.0, 0.0]);
		let result = sim.compute(&a, &b).unwrap();
		assert_eq!(result, 0.0);
	}

	#[rstest]
	fn cosine_known_value() {
		// Arrange
		let sim = CosineSimilarity;
		let a = FeatureVector::new(vec![1.0, 2.0]);
		let b = FeatureVector::new(vec![3.0, 4.0]);

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert — (1*3 + 2*4) / (sqrt(5) * sqrt(25)) = 11 / (sqrt(5)*5)
		let expected = 11.0 / (5.0_f64.sqrt() * 5.0);
		assert!((result - expected).abs() < 1e-10);
	}
}
