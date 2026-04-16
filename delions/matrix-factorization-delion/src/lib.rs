//! # Matrix Factorization Delion
//!
//! SVD and ALS matrix factorization recommendation plugin for the
//! reinhardt plugin framework.
//!
//! This crate provides latent factor models for collaborative filtering,
//! supporting both Singular Value Decomposition (SVD) via power iteration
//! and Alternating Least Squares (ALS) as factorization algorithms.
//!
//! ## Planned Features
//!
//! - SGD optimizer for online learning
//! - Bias terms (user and item biases)
//! - Implicit feedback support

pub mod error;
pub mod factorizer;
pub mod model;
pub mod plugin;
pub mod rating_matrix;
pub mod recommender;
pub mod types;

pub use error::MatrixFactorizationError;
pub use factorizer::{AlsFactorizer, Factorizer, FactorizerKind, SvdFactorizer};
pub use model::LatentFactorModel;
pub use rating_matrix::RatingMatrix;
pub use recommender::MatrixFactorizationRecommender;
pub use types::{ItemId, ModelConfig, Rating, Recommendation, UserId};
