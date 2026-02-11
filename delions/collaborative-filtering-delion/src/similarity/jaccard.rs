use crate::error::CollaborativeFilteringError;
use crate::similarity::Similarity;

/// Jaccard similarity metric.
///
/// Treats vectors as sets of non-zero elements and computes
/// `|intersection| / |union|`.
pub struct JaccardSimilarity;

impl Similarity for JaccardSimilarity {
	fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, CollaborativeFilteringError> {
		if a.is_empty() || b.is_empty() {
			return Err(CollaborativeFilteringError::EmptyVectors);
		}

		let set_a: Vec<bool> = a.iter().map(|x| *x != 0.0).collect();
		let set_b: Vec<bool> = b.iter().map(|x| *x != 0.0).collect();

		let intersection: usize = set_a
			.iter()
			.zip(set_b.iter())
			.filter(|(a, b)| **a && **b)
			.count();
		let union: usize = set_a
			.iter()
			.zip(set_b.iter())
			.filter(|(a, b)| **a || **b)
			.count();

		if union == 0 {
			// Both vectors are all zeros
			return Ok(1.0);
		}

		// Precision loss is acceptable: intersection/union counts are small in practice
		#[allow(clippy::cast_precision_loss)]
		Ok(intersection as f64 / union as f64)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;

	#[rstest]
	fn identical_nonzero_vectors() {
		// Arrange
		let sim = JaccardSimilarity;
		let a = [1.0, 2.0, 3.0];

		// Act
		let result = sim.compute(&a, &a).unwrap();

		// Assert
		assert!((result - 1.0).abs() < 1e-10);
	}

	#[rstest]
	fn completely_disjoint_sets() {
		// Arrange
		let sim = JaccardSimilarity;
		let a = [1.0, 0.0, 1.0, 0.0];
		let b = [0.0, 1.0, 0.0, 1.0];

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - 0.0).abs() < 1e-10);
	}

	#[rstest]
	fn partial_overlap() {
		// Arrange
		let sim = JaccardSimilarity;
		let a = [1.0, 1.0, 0.0, 0.0];
		let b = [1.0, 0.0, 1.0, 0.0];
		// intersection = 1 (index 0), union = 3 (indices 0,1,2)
		let expected = 1.0 / 3.0;

		// Act
		let result = sim.compute(&a, &b).unwrap();

		// Assert
		assert!((result - expected).abs() < 1e-10);
	}

	#[rstest]
	fn both_all_zeros_returns_one() {
		let sim = JaccardSimilarity;
		let a = [0.0, 0.0, 0.0];
		let result = sim.compute(&a, &a).unwrap();
		assert!((result - 1.0).abs() < 1e-10);
	}

	#[rstest]
	fn empty_vectors_return_error() {
		let sim = JaccardSimilarity;
		let result = sim.compute(&[], &[1.0]);
		assert!(matches!(
			result,
			Err(CollaborativeFilteringError::EmptyVectors)
		));
	}
}
