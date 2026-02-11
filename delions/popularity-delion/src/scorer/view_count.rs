use std::collections::HashMap;

use crate::error::PopularityError;
use crate::types::{InteractionEvent, InteractionKind, ItemId, PopularityScore, TimeWindow};

use super::PopularityScorer;

/// Scores items by counting view events within a time window.
pub struct ViewCountScorer;

impl PopularityScorer for ViewCountScorer {
	fn score(
		&self,
		events: &[InteractionEvent],
		window: &TimeWindow,
	) -> Result<Vec<PopularityScore>, PopularityError> {
		let mut counts: HashMap<ItemId, u64> = HashMap::new();

		for event in events {
			if event.kind == InteractionKind::View && window.contains(event.timestamp) {
				*counts.entry(event.item_id).or_default() += 1;
			}
		}

		let mut scores: Vec<PopularityScore> = counts
			.into_iter()
			.map(|(item_id, count)| PopularityScore {
				item_id,
				// Precision loss acceptable: view counts won't exceed 2^52
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
	fn counts_views_within_window() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(200),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base + Duration::from_secs(300),
				kind: InteractionKind::View,
			},
		];

		// Act
		let scores = ViewCountScorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 2);
		assert_eq!(scores[0].item_id, ItemId(1));
		assert_eq!(scores[0].score, 2.0);
		assert_eq!(scores[1].item_id, ItemId(2));
		assert_eq!(scores[1].score, 1.0);
	}

	#[rstest]
	fn ignores_non_view_events() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(200),
				kind: InteractionKind::Rating,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(300),
				kind: InteractionKind::Purchase,
			},
		];

		// Act
		let scores = ViewCountScorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 1);
		assert_eq!(scores[0].score, 1.0);
	}

	#[rstest]
	fn ignores_events_outside_window() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base + Duration::from_secs(5000),
				kind: InteractionKind::View,
			},
		];

		// Act
		let scores = ViewCountScorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 1);
		assert_eq!(scores[0].item_id, ItemId(1));
	}

	#[rstest]
	fn empty_events_returns_empty() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();

		// Act
		let scores = ViewCountScorer.score(&[], &window).unwrap();

		// Assert
		assert!(scores.is_empty());
	}

	#[rstest]
	fn aggregates_multiple_views_for_same_item() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(5),
				timestamp: base + Duration::from_secs(100),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(5),
				timestamp: base + Duration::from_secs(200),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(5),
				timestamp: base + Duration::from_secs(300),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(5),
				timestamp: base + Duration::from_secs(400),
				kind: InteractionKind::View,
			},
		];

		// Act
		let scores = ViewCountScorer.score(&events, &window).unwrap();

		// Assert
		assert_eq!(scores.len(), 1);
		assert_eq!(scores[0].item_id, ItemId(5));
		assert_eq!(scores[0].score, 4.0);
	}

	#[rstest]
	fn ignores_purchase_and_click_events() {
		// Arrange
		let base = SystemTime::UNIX_EPOCH;
		let window = TimeWindow::new(base, base + Duration::from_secs(3600)).unwrap();
		let events = vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base + Duration::from_secs(100),
				kind: InteractionKind::Purchase,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base + Duration::from_secs(200),
				kind: InteractionKind::Click,
			},
			InteractionEvent {
				item_id: ItemId(3),
				timestamp: base + Duration::from_secs(300),
				kind: InteractionKind::Purchase,
			},
			InteractionEvent {
				item_id: ItemId(4),
				timestamp: base + Duration::from_secs(400),
				kind: InteractionKind::Click,
			},
		];

		// Act
		let scores = ViewCountScorer.score(&events, &window).unwrap();

		// Assert
		assert!(scores.is_empty());
	}
}
