use std::collections::HashSet;

use crate::error::PopularityError;
use crate::scorer::PopularityScorer;
use crate::types::{Category, InteractionEvent, ItemMetadata, Recommendation, TimeWindow};

/// Produces recommendations using a popularity scorer.
pub struct PopularityRecommender {
	scorer: Box<dyn PopularityScorer>,
}

impl PopularityRecommender {
	/// Creates a new recommender with the given scorer.
	#[must_use]
	pub fn new(scorer: Box<dyn PopularityScorer>) -> Self {
		Self { scorer }
	}

	/// Returns the top `n` recommendations based on popularity scores.
	///
	/// # Errors
	///
	/// Returns [`PopularityError`] if the underlying scorer fails.
	pub fn recommend(
		&self,
		events: &[InteractionEvent],
		window: &TimeWindow,
		n: usize,
	) -> Result<Vec<Recommendation>, PopularityError> {
		let scores = self.scorer.score(events, window)?;
		let recommendations = scores
			.into_iter()
			.take(n)
			.map(|s| Recommendation {
				item_id: s.item_id,
				score: s.score,
			})
			.collect();
		Ok(recommendations)
	}

	/// Returns the top `n` recommendations filtered to items in the given
	/// category.
	///
	/// # Errors
	///
	/// Returns [`PopularityError::CategoryNotFound`] if no items match the
	/// category, or propagates errors from the underlying scorer.
	pub fn recommend_by_category(
		&self,
		events: &[InteractionEvent],
		metadata: &[ItemMetadata],
		window: &TimeWindow,
		category: &Category,
		n: usize,
	) -> Result<Vec<Recommendation>, PopularityError> {
		let category_items: HashSet<_> = metadata
			.iter()
			.filter(|m| m.category == *category)
			.map(|m| m.id)
			.collect();

		if category_items.is_empty() {
			return Err(PopularityError::CategoryNotFound(category.clone()));
		}

		let filtered_events: Vec<_> = events
			.iter()
			.filter(|e| category_items.contains(&e.item_id))
			.cloned()
			.collect();

		let scores = self.scorer.score(&filtered_events, window)?;
		let recommendations = scores
			.into_iter()
			.take(n)
			.map(|s| Recommendation {
				item_id: s.item_id,
				score: s.score,
			})
			.collect();
		Ok(recommendations)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::time::{Duration, SystemTime};

	use crate::scorer::ViewCountScorer;
	use crate::types::{InteractionKind, ItemId};

	use rstest::{fixture, rstest};

	#[fixture]
	fn base_time() -> SystemTime {
		SystemTime::UNIX_EPOCH
	}

	#[fixture]
	fn window(base_time: SystemTime) -> TimeWindow {
		TimeWindow::new(base_time, base_time + Duration::from_secs(3600)).unwrap()
	}

	#[fixture]
	fn sample_events(base_time: SystemTime) -> Vec<InteractionEvent> {
		vec![
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base_time + Duration::from_secs(100),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(1),
				timestamp: base_time + Duration::from_secs(200),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(2),
				timestamp: base_time + Duration::from_secs(300),
				kind: InteractionKind::View,
			},
			InteractionEvent {
				item_id: ItemId(3),
				timestamp: base_time + Duration::from_secs(400),
				kind: InteractionKind::View,
			},
		]
	}

	#[rstest]
	fn recommend_returns_top_n(window: TimeWindow, sample_events: Vec<InteractionEvent>) {
		// Arrange
		let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));

		// Act
		let recs = recommender.recommend(&sample_events, &window, 2).unwrap();

		// Assert
		assert_eq!(recs.len(), 2);
		assert_eq!(recs[0].item_id, ItemId(1));
		assert_eq!(recs[0].score, 2.0);
	}

	#[rstest]
	fn recommend_returns_all_when_n_exceeds_items(
		window: TimeWindow,
		sample_events: Vec<InteractionEvent>,
	) {
		// Arrange
		let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));

		// Act
		let recs = recommender.recommend(&sample_events, &window, 100).unwrap();

		// Assert
		assert_eq!(recs.len(), 3);
	}

	#[rstest]
	fn recommend_by_category_filters_items(
		window: TimeWindow,
		sample_events: Vec<InteractionEvent>,
	) {
		// Arrange
		let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));
		let metadata = vec![
			ItemMetadata {
				id: ItemId(1),
				category: Category("electronics".to_string()),
			},
			ItemMetadata {
				id: ItemId(2),
				category: Category("books".to_string()),
			},
			ItemMetadata {
				id: ItemId(3),
				category: Category("electronics".to_string()),
			},
		];
		let category = Category("electronics".to_string());

		// Act
		let recs = recommender
			.recommend_by_category(&sample_events, &metadata, &window, &category, 10)
			.unwrap();

		// Assert
		assert_eq!(recs.len(), 2);
		let ids: Vec<_> = recs.iter().map(|r| r.item_id).collect();
		assert!(ids.contains(&ItemId(1)));
		assert!(ids.contains(&ItemId(3)));
	}

	#[rstest]
	fn recommend_by_category_unknown_category(
		window: TimeWindow,
		sample_events: Vec<InteractionEvent>,
	) {
		// Arrange
		let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));
		let metadata = vec![ItemMetadata {
			id: ItemId(1),
			category: Category("electronics".to_string()),
		}];
		let category = Category("nonexistent".to_string());

		// Act
		let result =
			recommender.recommend_by_category(&sample_events, &metadata, &window, &category, 10);

		// Assert
		assert!(matches!(result, Err(PopularityError::CategoryNotFound(_))));
	}

	#[rstest]
	fn recommend_with_zero_n_returns_empty(
		window: TimeWindow,
		sample_events: Vec<InteractionEvent>,
	) {
		// Arrange
		let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));

		// Act
		let recs = recommender.recommend(&sample_events, &window, 0).unwrap();

		// Assert
		assert!(recs.is_empty());
	}

	#[rstest]
	fn recommend_by_category_with_empty_events(window: TimeWindow) {
		// Arrange
		let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));
		let metadata = vec![ItemMetadata {
			id: ItemId(1),
			category: Category("electronics".to_string()),
		}];
		let category = Category("electronics".to_string());

		// Act
		let recs = recommender
			.recommend_by_category(&[], &metadata, &window, &category, 10)
			.unwrap();

		// Assert
		assert!(recs.is_empty());
	}

	#[rstest]
	fn recommend_by_category_multiple_categories(
		window: TimeWindow,
		sample_events: Vec<InteractionEvent>,
	) {
		// Arrange
		let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));
		let metadata = vec![
			ItemMetadata {
				id: ItemId(1),
				category: Category("electronics".to_string()),
			},
			ItemMetadata {
				id: ItemId(2),
				category: Category("books".to_string()),
			},
			ItemMetadata {
				id: ItemId(3),
				category: Category("clothing".to_string()),
			},
		];

		// Act
		let electronics = recommender
			.recommend_by_category(
				&sample_events,
				&metadata,
				&window,
				&Category("electronics".to_string()),
				10,
			)
			.unwrap();
		let books = recommender
			.recommend_by_category(
				&sample_events,
				&metadata,
				&window,
				&Category("books".to_string()),
				10,
			)
			.unwrap();
		let clothing = recommender
			.recommend_by_category(
				&sample_events,
				&metadata,
				&window,
				&Category("clothing".to_string()),
				10,
			)
			.unwrap();

		// Assert
		assert_eq!(electronics.len(), 1);
		assert_eq!(electronics[0].item_id, ItemId(1));
		assert_eq!(books.len(), 1);
		assert_eq!(books[0].item_id, ItemId(2));
		assert_eq!(clothing.len(), 1);
		assert_eq!(clothing[0].item_id, ItemId(3));
	}
}
