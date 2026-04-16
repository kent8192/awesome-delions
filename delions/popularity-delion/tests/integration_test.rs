use std::time::{Duration, SystemTime};

use popularity_delion::{
	Category, ExponentialDecay, InteractionEvent, InteractionKind, ItemId, ItemMetadata, NoDecay,
	PopularityPlugin, PopularityRecommender, PopularityScorer, RatingCountScorer, TimeWindow,
	TrendingScorer, ViewCountScorer,
};
use reinhardt_dentdelion::prelude::*;
use rstest::{fixture, rstest};

#[fixture]
fn base_time() -> SystemTime {
	SystemTime::UNIX_EPOCH
}

#[fixture]
fn window(base_time: SystemTime) -> TimeWindow {
	TimeWindow::new(base_time, base_time + Duration::from_secs(7200)).unwrap()
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
			item_id: ItemId(1),
			timestamp: base_time + Duration::from_secs(300),
			kind: InteractionKind::Rating,
		},
		InteractionEvent {
			item_id: ItemId(2),
			timestamp: base_time + Duration::from_secs(400),
			kind: InteractionKind::View,
		},
		InteractionEvent {
			item_id: ItemId(2),
			timestamp: base_time + Duration::from_secs(500),
			kind: InteractionKind::Rating,
		},
		InteractionEvent {
			item_id: ItemId(2),
			timestamp: base_time + Duration::from_secs(600),
			kind: InteractionKind::Rating,
		},
		InteractionEvent {
			item_id: ItemId(3),
			timestamp: base_time + Duration::from_secs(700),
			kind: InteractionKind::View,
		},
	]
}

#[fixture]
fn sample_metadata() -> Vec<ItemMetadata> {
	vec![
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
	]
}

// --- 1. Full pipeline test ---

#[rstest]
fn full_pipeline_view_count_recommender(window: TimeWindow, sample_events: Vec<InteractionEvent>) {
	// Arrange
	let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));

	// Act
	let recs = recommender.recommend(&sample_events, &window, 2).unwrap();

	// Assert
	assert_eq!(recs.len(), 2);
	// Item 1 has 2 views, item 2 has 1 view, item 3 has 1 view
	assert_eq!(recs[0].item_id, ItemId(1));
	assert_eq!(recs[0].score, 2.0);
}

// --- 2. All scorer comparison ---

#[rstest]
fn all_scorers_on_same_data(window: TimeWindow, sample_events: Vec<InteractionEvent>) {
	// Arrange
	let view_scorer = ViewCountScorer;
	let rating_scorer = RatingCountScorer;
	let trending_scorer = TrendingScorer::new(Box::new(NoDecay));

	// Act
	let view_scores = view_scorer.score(&sample_events, &window).unwrap();
	let rating_scores = rating_scorer.score(&sample_events, &window).unwrap();
	let trending_scores = trending_scorer.score(&sample_events, &window).unwrap();

	// Assert
	// ViewCount: item1=2, item2=1, item3=1
	assert_eq!(view_scores.len(), 3);
	assert_eq!(view_scores[0].item_id, ItemId(1));
	assert_eq!(view_scores[0].score, 2.0);

	// RatingCount: item2=2, item1=1
	assert_eq!(rating_scores.len(), 2);
	assert_eq!(rating_scores[0].item_id, ItemId(2));
	assert_eq!(rating_scores[0].score, 2.0);
	assert_eq!(rating_scores[1].item_id, ItemId(1));
	assert_eq!(rating_scores[1].score, 1.0);

	// TrendingScorer with NoDecay counts all event kinds: item1=3, item2=3, item3=1
	assert_eq!(trending_scores.len(), 3);
	// Items 1 and 2 both have score 3.0, item 3 has 1.0
	let item3_score = trending_scores
		.iter()
		.find(|s| s.item_id == ItemId(3))
		.unwrap();
	assert_eq!(item3_score.score, 1.0);
	assert_eq!(trending_scores[0].score, 3.0);
}

// --- 3. Category filtering ---

#[rstest]
fn category_filtering_integration(
	window: TimeWindow,
	sample_events: Vec<InteractionEvent>,
	sample_metadata: Vec<ItemMetadata>,
) {
	// Arrange
	let recommender = PopularityRecommender::new(Box::new(ViewCountScorer));
	let electronics = Category("electronics".to_string());
	let books = Category("books".to_string());

	// Act
	let elec_recs = recommender
		.recommend_by_category(&sample_events, &sample_metadata, &window, &electronics, 10)
		.unwrap();
	let book_recs = recommender
		.recommend_by_category(&sample_events, &sample_metadata, &window, &books, 10)
		.unwrap();

	// Assert
	// Electronics: item1 (2 views), item3 (1 view)
	assert_eq!(elec_recs.len(), 2);
	assert_eq!(elec_recs[0].item_id, ItemId(1));
	assert_eq!(elec_recs[0].score, 2.0);
	assert_eq!(elec_recs[1].item_id, ItemId(3));
	assert_eq!(elec_recs[1].score, 1.0);

	// Books: item2 (1 view)
	assert_eq!(book_recs.len(), 1);
	assert_eq!(book_recs[0].item_id, ItemId(2));
	assert_eq!(book_recs[0].score, 1.0);
}

// --- 4. Time decay effect ---

#[rstest]
fn time_decay_effect_old_vs_new(base_time: SystemTime) {
	// Arrange
	let window = TimeWindow::new(base_time, base_time + Duration::from_secs(7200)).unwrap();
	let decay = ExponentialDecay::new(Duration::from_secs(3600)).unwrap();
	let scorer = TrendingScorer::new(Box::new(decay));

	// Item 1: old event near window start
	// Item 2: new event near window end
	let events = vec![
		InteractionEvent {
			item_id: ItemId(1),
			timestamp: base_time + Duration::from_secs(10),
			kind: InteractionKind::View,
		},
		InteractionEvent {
			item_id: ItemId(2),
			timestamp: base_time + Duration::from_secs(7190),
			kind: InteractionKind::View,
		},
	];

	// Act
	let scores = scorer.score(&events, &window).unwrap();

	// Assert
	assert_eq!(scores.len(), 2);
	// New event (item 2) should have a much higher score
	assert_eq!(scores[0].item_id, ItemId(2));
	assert!(scores[0].score > scores[1].score);
	// Item 2 is ~10s old, item 1 is ~7190s old (about 2 half-lives)
	// Item 2 weight ≈ 1.0, Item 1 weight ≈ 0.25
	assert!(scores[0].score > 0.9);
	assert!(scores[1].score < 0.3);
}

// --- 5. Plugin registration ---

#[rstest]
fn plugin_metadata_and_capabilities() {
	// Arrange
	let plugin = PopularityPlugin::new();

	// Act
	let metadata = plugin.metadata();
	let capabilities = plugin.capabilities();

	// Assert
	assert_eq!(metadata.name, "popularity-delion");
	assert_eq!(metadata.version.to_string(), "0.1.0");
	assert_eq!(capabilities.len(), 3);
	assert!(matches!(
		&capabilities[0],
		Capability::Custom(s) if s == "recommendation"
	));
	assert!(matches!(
		&capabilities[1],
		Capability::Custom(s) if s == "popularity"
	));
	assert!(matches!(
		&capabilities[2],
		Capability::Custom(s) if s == "trending"
	));
}
