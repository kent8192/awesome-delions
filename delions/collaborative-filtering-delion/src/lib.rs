//! # collaborative-filtering-delion
//!
//! User-based and item-based collaborative filtering recommendation plugin.
//!
//! ## Planned Features
//!
//! - Implicit feedback support
//! - Incremental matrix updates

mod plugin;

pub mod error;
pub mod rating_matrix;
pub mod recommender;
pub mod similarity;
pub mod types;

pub use error::CollaborativeFilteringError;
pub use rating_matrix::SparseRatingMatrix;
pub use recommender::{ItemBasedRecommender, Recommender, UserBasedRecommender};
pub use similarity::{
	CosineSimilarity, JaccardSimilarity, PearsonCorrelation, Similarity, SimilarityKind,
};
pub use types::{CollaborativeFilteringConfig, ItemId, Rating, Recommendation, UserId};
