use std::collections::HashMap;

use crate::decay::DecayFunction;
use crate::error::PopularityError;
use crate::types::{InteractionEvent, ItemId, PopularityScore, TimeWindow};

use super::PopularityScorer;

/// Scores items using time-decay weighting for trending detection.
///
/// For each event within the time window, the scorer computes a weight based
/// on the event's age relative to the window end, then sums weights per item.
pub struct TrendingScorer {
	decay: Box<dyn DecayFunction>,
}

impl TrendingScorer {
	/// Creates a new `TrendingScorer` with the given decay function.
	#[must_use]
	pub fn new(decay: Box<dyn DecayFunction>) -> Self {
		Self { decay }
	}
}

impl PopularityScorer for TrendingScorer {
	fn score(
		&self,
		events: &[InteractionEvent],
		window: &TimeWindow,
	) -> Result<Vec<PopularityScore>, PopularityError> {
		let mut weights: HashMap<ItemId, f64> = HashMap::new();

		for event in events {
			if !window.contains(event.timestamp) {
				continue;
			}
			let age = window
				.end
				.duration_since(event.timestamp)
				// Events within the window should never be after window.end
				.unwrap_or_default();
			let w = self.decay.weight(age);
			*weights.entry(event.item_id).or_default() += w;
		}

		let mut scores: Vec<PopularityScore> = weights
			.into_iter()
			.map(|(item_id, score)| PopularityScore { item_id, score })
			.collect();

		scores.sort_by(|a, b| {
			b.score
				.partial_cmp(&a.score)
				.unwrap_or(std::cmp::Ordering::Equal)
		});

		Ok(scores)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::time::{Duration, SystemTime};

	use crate::decay::{ExponentialDecay, NoDecay};

	use rstest::rstest;

	#[rstest]
	fn trending_with_no_decay_acts_as_counter() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: crate::types::InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(200),
				kind: crate::types::InteractionKind::Rating,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base + Duration::from_secs(300),
				kind: crate::types::InteractionKind::View,
			},
		];
		let scorer = TrendingScorer::new(Box::new(NoDecay));

		// Act
		let scores = scorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 2);
		assert_eq!(scores[0].item_id, ItemId(1));
		assert_eq!(scores[0].score, 2.0);
		assert_eq!(scores[1].item_id, ItemId(2));
		assert_eq!(scores[1].score, 1.0);
	}

	#[rstest]
	fn trending_with_decay_favors_recent_events() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(7200)).unwrap();
		let decay = ExponentialDecay::new(Duration::from_secs(3600)).unwrap();
		let scorer = TrendingScorer::new(Box::new(decay));

		// Item 1: one old event (age = 7100s, ~2 half-lives)
		// Item 2: one recent event (age = 100s, ~0 half-lives)
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: crate::types::InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base + Duration::from_secs(7100),
				kind: crate::types::InteractionKind::View,
			},
		];

		// Act
		let scores = scorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 2);
		// Recent event (item 2) should rank higher
		assert_eq!(scores[0].item_id, ItemId(2));
		assert!(scores[0].score > scores[1].score);
	}

	#[rstest]
	fn trending_ignores_events_outside_window() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let scorer = TrendingScorer::new(Box::new(NoDecay));
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: crate::types::InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base + Duration::from_secs(5000),
				kind: crate::types::InteractionKind::View,
			},
		];

		// Act
		let scores = scorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 1);
		assert_eq!(scores[0].item_id, ItemId(1));
	}

	#[rstest]
	fn trending_empty_events() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let scorer = TrendingScorer::new(Box::new(NoDecay));

		// Act
		let scores = scorer.score(&[], &window).unwrap();

		// Assert
		assert!(scores.is_empty());
	}
}
