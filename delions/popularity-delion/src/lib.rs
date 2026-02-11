//! # popularity-delion
//!
//! Time-decay popularity-based recommendation plugin for the
//! [reinhardt](https://github.com/kent8192/reinhardt) framework.
//!
//! This crate provides popularity scoring and recommendation using view counts,
//! rating counts, and trending scores with configurable time-decay functions.
//!
//! ## Features
//!
//! - **View count scoring** -- rank items by total views in a time window
//! - **Rating count scoring** -- rank items by total ratings in a time window
//! - **Trending scoring** -- rank items with time-decay weighting (exponential,
//!   linear, or no decay)
//! - **Category-based recommendations** -- filter results by item category
//!
//! ## Planned Features
//!
//! - Weighted interaction types (configurable weights per `InteractionKind`)
//! - Real-time streaming event ingestion

pub mod decay;
pub mod error;
pub mod plugin;
pub mod recommender;
pub mod scorer;
pub mod types;

pub use decay::{DecayFunction, ExponentialDecay, LinearDecay, NoDecay};
pub use error::PopularityError;
pub use plugin::PopularityPlugin;
pub use recommender::PopularityRecommender;
pub use scorer::{
	PopularityScorer, RatingCountScorer, ScorerKind, TrendingScorer, ViewCountScorer,
};
pub use types::{
	Category, InteractionEvent, InteractionKind, ItemId, ItemMetadata, PopularityScore,
	Recommendation, TimeWindow,
};
