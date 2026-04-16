mod cosine;
mod jaccard;
mod pearson;

pub use cosine::CosineSimilarity;
pub use jaccard::JaccardSimilarity;
pub use pearson::PearsonCorrelation;

use crate::error::CollaborativeFilteringError;

/// Trait for computing similarity between two vectors.
pub trait Similarity: Send + Sync {
	/// Computes the similarity between two vectors.
	///
	/// # Errors
	///
	/// Returns `CollaborativeFilteringError` if the vectors are empty or have zero norm.
	fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, CollaborativeFilteringError>;
}

/// Available similarity metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityKind {
	Cosine,
	Pearson,
	Jaccard,
}

impl SimilarityKind {
	/// Creates a boxed similarity implementation for this kind.
	#[must_use]
	pub fn into_similarity(self) -> Box<dyn Similarity> {
		match self {
			Self::Cosine => Box::new(CosineSimilarity),
			Self::Pearson => Box::new(PearsonCorrelation),
			Self::Jaccard => Box::new(JaccardSimilarity),
		}
	}
}
