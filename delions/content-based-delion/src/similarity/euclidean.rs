use crate::error::ContentBasedError;
use crate::similarity::SimilarityMetric;
use crate::types::FeatureVector;

/// Computes similarity from Euclidean distance using the transformation
/// `1.0 / (1.0 + distance)`.
///
/// This maps distances in `[0, inf)` to similarities in `(0, 1]`.
#[derive(Debug)]
pub struct EuclideanDistance;

impl SimilarityMetric for EuclideanDistance {
	fn compute(&self, a: &FeatureVector, b: &FeatureVector) -> Result<f64, ContentBasedError> {
		if a.values.is_empty() || b.values.is_empty() {
			return Err(ContentBasedError::EmptyFeatureVector);
		}
		if a.values.len() != b.values.len() {
			return Err(ContentBasedError::DimensionMismatch {
				expected: a.values.len(),
				actual: b.values.len(),
			});
		}
		let distance: f64 = a
			.values
			.iter()
			.zip(b.values.iter())
			.map(|(x, y)| (x - y).powi(2))
			.sum::<f64>()
			.sqrt();

		Ok(1.0 / (1.0 + distance))
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	fn euclidean_identical_vectors() {
		let metric = EuclideanDistance;
		let v = FeatureVector::new(vec![1.0, 2.0, 3.0]);
		let result = metric.compute(&v, &v).unwrap();
		assert_eq!(result, 1.0);
	}

	#[rstest]
	fn euclidean_known_distance() {
		// Arrange
		let metric = EuclideanDistance;
		let a = FeatureVector::new(vec![0.0, 0.0]);
		let b = FeatureVector::new(vec![3.0, 4.0]);

		// Act
		let result = metric.compute(&a, &b).unwrap();

		// Assert — distance = 5.0, similarity = 1/(1+5) = 1/6
		let expected = 1.0 / 6.0;
		assert!((result - expected).abs() < 1e-10);
	}

	#[rstest]
	fn euclidean_empty_vector_returns_error() {
		let metric = EuclideanDistance;
		let a = FeatureVector::new(vec![]);
		let b = FeatureVector::new(vec![1.0]);
		let result = metric.compute(&a, &b);
		assert!(matches!(result, Err(ContentBasedError::EmptyFeatureVector)));
	}

	#[rstest]
	fn euclidean_dimension_mismatch() {
		let metric = EuclideanDistance;
		let a = FeatureVector::new(vec![1.0]);
		let b = FeatureVector::new(vec![1.0, 2.0]);
		let result = metric.compute(&a, &b);
		assert!(matches!(
			result,
			Err(ContentBasedError::DimensionMismatch { .. })
		));
	}

	#[rstest]
	fn euclidean_identical_vectors_returns_one() {
		// Arrange
		let metric = EuclideanDistance;
		let a = FeatureVector::new(vec![3.0, 7.0, -2.0]);

		// Act
		let result = metric.compute(&a, &a).unwrap();

		// Assert — distance=0, similarity = 1/(1+0) = 1.0
		assert_eq!(result, 1.0);
	}

	#[rstest]
	fn euclidean_similarity_decreases_with_distance() {
		// Arrange
		let metric = EuclideanDistance;
		let origin = FeatureVector::new(vec![0.0, 0.0]);
		let near = FeatureVector::new(vec![1.0, 0.0]);
		let far = FeatureVector::new(vec![10.0, 0.0]);

		// Act
		let sim_near = metric.compute(&origin, &near).unwrap();
		let sim_far = metric.compute(&origin, &far).unwrap();

		// Assert
		assert!(sim_near > sim_far);
	}
}
