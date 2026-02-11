use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::ContentBasedError;

/// Unique identifier for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl fmt::Display for UserId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Unique identifier for an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

impl fmt::Display for ItemId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// A vector of floating-point feature values.
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureVector {
	pub values: Vec<f64>,
}

impl FeatureVector {
	/// Creates a new feature vector from the given values.
	#[must_use]
	pub fn new(values: Vec<f64>) -> Self {
		Self { values }
	}

	/// Computes the dot product of this vector with another.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::DimensionMismatch` if the vectors have
	/// different dimensions.
	pub fn dot(&self, other: &Self) -> Result<f64, ContentBasedError> {
		if self.values.len() != other.values.len() {
			return Err(ContentBasedError::DimensionMismatch {
				expected: self.values.len(),
				actual: other.values.len(),
			});
		}
		Ok(self
			.values
			.iter()
			.zip(other.values.iter())
			.map(|(a, b)| a * b)
			.sum())
	}

	/// Computes the L2 norm (Euclidean length) of this vector.
	#[must_use]
	pub fn norm(&self) -> f64 {
		self.values.iter().map(|v| v * v).sum::<f64>().sqrt()
	}

	/// Returns a normalized (unit length) copy of this vector.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::EmptyFeatureVector` if the vector is empty.
	pub fn normalize(&self) -> Result<Self, ContentBasedError> {
		if self.values.is_empty() {
			return Err(ContentBasedError::EmptyFeatureVector);
		}
		let n = self.norm();
		if n == 0.0 {
			return Ok(self.clone());
		}
		Ok(Self {
			values: self.values.iter().map(|v| v / n).collect(),
		})
	}

	/// Returns the dimensionality of this vector.
	#[must_use]
	pub fn dimension(&self) -> usize {
		self.values.len()
	}

	/// Returns a new vector scaled by the given factor.
	#[must_use]
	pub fn scale(&self, factor: f64) -> Self {
		Self {
			values: self.values.iter().map(|v| v * factor).collect(),
		}
	}
}

/// A tokenized document associated with an item.
#[derive(Debug, Clone)]
pub struct Document {
	pub id: ItemId,
	pub tokens: Vec<String>,
}

/// A recommendation result containing the item and its relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
	pub item_id: ItemId,
	pub score: f64,
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	fn feature_vector_dot_product() {
		// Arrange
		let a = FeatureVector::new(vec![1.0, 2.0, 3.0]);
		let b = FeatureVector::new(vec![4.0, 5.0, 6.0]);

		// Act
		let result = a.dot(&b).unwrap();

		// Assert
		assert_eq!(result, 32.0);
	}

	#[rstest]
	fn feature_vector_dot_dimension_mismatch() {
		// Arrange
		let a = FeatureVector::new(vec![1.0, 2.0]);
		let b = FeatureVector::new(vec![1.0, 2.0, 3.0]);

		// Act
		let result = a.dot(&b);

		// Assert
		assert!(matches!(
			result,
			Err(ContentBasedError::DimensionMismatch {
				expected: 2,
				actual: 3
			})
		));
	}

	#[rstest]
	fn feature_vector_norm() {
		let v = FeatureVector::new(vec![3.0, 4.0]);
		assert_eq!(v.norm(), 5.0);
	}

	#[rstest]
	fn feature_vector_normalize() {
		// Arrange
		let v = FeatureVector::new(vec![3.0, 4.0]);

		// Act
		let normalized = v.normalize().unwrap();

		// Assert
		let expected = FeatureVector::new(vec![0.6, 0.8]);
		assert_eq!(normalized.values.len(), expected.values.len());
		for (a, b) in normalized.values.iter().zip(expected.values.iter()) {
			assert!((a - b).abs() < 1e-10);
		}
	}

	#[rstest]
	fn feature_vector_normalize_empty() {
		let v = FeatureVector::new(vec![]);
		assert!(matches!(
			v.normalize(),
			Err(ContentBasedError::EmptyFeatureVector)
		));
	}

	#[rstest]
	fn feature_vector_normalize_zero() {
		let v = FeatureVector::new(vec![0.0, 0.0]);
		let normalized = v.normalize().unwrap();
		assert_eq!(normalized.values, vec![0.0, 0.0]);
	}

	#[rstest]
	fn feature_vector_dimension() {
		let v = FeatureVector::new(vec![1.0, 2.0, 3.0]);
		assert_eq!(v.dimension(), 3);
	}

	#[rstest]
	fn feature_vector_scale() {
		let v = FeatureVector::new(vec![1.0, 2.0, 3.0]);
		let scaled = v.scale(2.0);
		assert_eq!(scaled.values, vec![2.0, 4.0, 6.0]);
	}

	#[rstest]
	fn user_id_display() {
		assert_eq!(UserId(42).to_string(), "42");
	}

	#[rstest]
	fn item_id_display() {
		assert_eq!(ItemId(99).to_string(), "99");
	}
}
