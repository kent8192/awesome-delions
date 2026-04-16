pub mod rating_count;
pub mod trending;
pub mod view_count;

use crate::decay::DecayFunction;
use crate::error::PopularityError;
use crate::types::{InteractionEvent, PopularityScore, TimeWindow};

pub use rating_count::RatingCountScorer;
pub use trending::TrendingScorer;
pub use view_count::ViewCountScorer;

/// Computes popularity scores from interaction events within a time window.
pub trait PopularityScorer: Send + Sync {
	/// Scores items based on the given events within the time window.
	///
	/// Returns scores sorted in descending order by score.
	///
	/// # Errors
	///
	/// Returns [`PopularityError`] if scoring fails.
	fn score(
		&self,
		events: &[InteractionEvent],
		window: &TimeWindow,
	) -> Result<Vec<PopularityScore>, PopularityError>;
}

/// Describes which scoring strategy to use.
pub enum ScorerKind {
	ViewCount,
	RatingCount,
	Trending(Box<dyn DecayFunction>),
}

impl ScorerKind {
	/// Converts this kind into a boxed scorer implementation.
	#[must_use]
	pub fn into_scorer(self) -> Box<dyn PopularityScorer> {
		match self {
			Self::ViewCount => Box::new(ViewCountScorer),
			Self::RatingCount => Box::new(RatingCountScorer),
			Self::Trending(decay) => Box::new(TrendingScorer::new(decay)),
		}
	}
}
