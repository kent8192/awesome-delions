pub mod als;
pub mod svd;

use crate::error::MatrixFactorizationError;
use crate::model::LatentFactorModel;
use crate::rating_matrix::RatingMatrix;
use crate::types::ModelConfig;

pub use als::AlsFactorizer;
pub use svd::SvdFactorizer;

/// Trait for matrix factorization algorithms.
pub trait Factorizer {
	/// Factorize a rating matrix into a latent factor model.
	///
	/// # Errors
	///
	/// Returns an error if factorization fails due to numerical issues
	/// or invalid configuration.
	fn factorize(
		&self,
		matrix: &RatingMatrix,
		config: &ModelConfig,
	) -> Result<LatentFactorModel, MatrixFactorizationError>;
}

/// Available factorization algorithms.
pub enum FactorizerKind {
	/// Truncated SVD via power iteration.
	Svd,
	/// Alternating Least Squares.
	Als,
}

impl FactorizerKind {
	/// Convert into a boxed `Factorizer` implementation.
	#[must_use]
	pub fn into_factorizer(self) -> Box<dyn Factorizer> {
		match self {
			Self::Svd => Box::new(SvdFactorizer),
			Self::Als => Box::new(AlsFactorizer),
		}
	}
}
