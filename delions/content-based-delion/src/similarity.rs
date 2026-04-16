mod cosine;
mod euclidean;

pub use cosine::CosineSimilarity;
pub use euclidean::EuclideanDistance;

use crate::error::ContentBasedError;
use crate::types::FeatureVector;

/// Trait for computing similarity between two feature vectors.
pub trait SimilarityMetric {
	/// Computes the similarity score between vectors `a` and `b`.
	///
	/// Higher values indicate greater similarity.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::DimensionMismatch` if vectors differ in
	/// dimension, or `ContentBasedError::EmptyFeatureVector` if either is empty.
	fn compute(&self, a: &FeatureVector, b: &FeatureVector) -> Result<f64, ContentBasedError>;
}
