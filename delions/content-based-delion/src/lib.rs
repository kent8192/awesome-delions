//! # content-based-delion
//!
//! TF-IDF and feature similarity content-based recommendation plugin for the
//! reinhardt ecosystem.
//!
//! This crate provides a complete content-based recommendation pipeline:
//!
//! 1. **Feature Extraction** -- Transform tokenized documents into TF-IDF
//!    feature vectors
//! 2. **Profile Building** -- Construct item and user profiles from feature
//!    vectors
//! 3. **Similarity Computation** -- Compare profiles using cosine similarity
//!    or Euclidean distance
//! 4. **Recommendation** -- Generate ranked item recommendations for users
//!
//! ## Planned Features
//!
//! - Implicit feedback support (click/view signals)
//! - Online learning with incremental vocabulary updates

pub mod error;
mod plugin;
pub mod profile;
pub mod recommender;
pub mod similarity;
pub mod tfidf;
pub mod types;

pub use error::ContentBasedError;
pub use profile::{ItemProfile, ItemProfileBuilder, UserProfile, UserProfileBuilder};
pub use recommender::ContentBasedRecommender;
pub use similarity::{CosineSimilarity, EuclideanDistance, SimilarityMetric};
pub use tfidf::{TfIdf, TfIdfConfig};
pub use types::{Document, FeatureVector, ItemId, Recommendation, UserId};
