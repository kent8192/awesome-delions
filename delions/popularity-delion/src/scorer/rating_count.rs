use std::collections::HashMap;

use crate::error::PopularityError;
use crate::types::{InteractionEvent, InteractionKind, ItemId, PopularityScore, TimeWindow};

use super::PopularityScorer;

/// Scores items by counting rating events within a time window.
pub struct RatingCountScorer;

impl PopularityScorer for RatingCountScorer {
	fn score(
		&self,
		events: &[InteractionEvent],
		window: &TimeWindow,
	) -> Result<Vec<PopularityScore>, PopularityError> {
		let mut counts: HashMap<ItemId, u64> = HashMap::new();

		for event in events {
			if event.kind == InteractionKind::Rating && window.contains(event.timestamp) {
				*counts.entry(event.item_id).or_default() += 1;
			}
		}

		let mut scores: Vec<PopularityScore> = counts
			.into_iter()
			.map(|(item_id, count)| PopularityScore {
				item_id,
				// Precision loss acceptable: rating counts won't exceed 2^52
				#[allow(clippy::cast_precision_loss)]
				score: count as f64,
			})
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

	use rstest::rstest;

	#[rstest]
	fn counts_ratings_within_window() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: InteractionKind::Rating,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(200),
				kind: InteractionKind::Rating,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base + Duration::from_secs(300),
				kind: InteractionKind::Rating,
			},
		];

		// Act
		let scores = RatingCountScorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 2);
		assert_eq!(scores[0].item_id, ItemId(1));
		assert_eq!(scores[0].score, 2.0);
		assert_eq!(scores[1].item_id, ItemId(2));
		assert_eq!(scores[1].score, 1.0);
	}

	#[rstest]
	fn ignores_non_rating_events() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: InteractionKind::Rating,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(200),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(300),
				kind: InteractionKind::Click,
			},
		];

		// Act
		let scores = RatingCountScorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 1);
		assert_eq!(scores[0].score, 1.0);
	}

	#[rstest]
	fn empty_events_returns_empty() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();

		// Act
		let scores = RatingCountScorer.score(&[], &window).unwrap();

		// Assert
		assert!(scores.is_empty());
	}
}
