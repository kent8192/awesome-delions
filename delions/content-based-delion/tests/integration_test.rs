use std::collections::HashSet;

use content_based_delion::profile::{ItemProfileBuilder, UserProfileBuilder};
use content_based_delion::recommender::ContentBasedRecommender;
use content_based_delion::similarity::{CosineSimilarity, EuclideanDistance};
use content_based_delion::tfidf::TfIdfConfig;
use content_based_delion::types::{Document, ItemId, Recommendation, UserId};
use rstest::rstest;

fn make_doc(id: u64, tokens: &[&str]) -> Document {
	Document {
		id: ItemId(id),
		tokens: tokens.iter().map(|s| s.to_string()).collect(),
	}
}

fn verify_valid_recommendations(recs: &[Recommendation]) {
	for window in recs.windows(2) {
		assert!(
			window[0].score >= window[1].score,
			"recommendations not sorted descending: {} < {}",
			window[0].score,
			window[1].score
		);
	}
}

#[rstest]
fn full_pipeline_tfidf_to_recommendation() {
	// Arrange
	let docs = vec![
		make_doc(1, &["rust", "systems", "programming", "fast"]),
		make_doc(2, &["python", "data", "science", "ml"]),
		make_doc(3, &["rust", "web", "programming", "async"]),
		make_doc(4, &["java", "enterprise", "backend", "spring"]),
		make_doc(5, &["python", "ml", "deep", "learning"]),
	];

	let mut builder = ItemProfileBuilder::new(TfIdfConfig::default());
	let item_profiles = builder.build_profiles(&docs).unwrap();

	// User likes Rust items
	let rated_items = vec![(ItemId(1), 5.0), (ItemId(3), 4.0)];
	let user_profile =
		UserProfileBuilder::build_profile(UserId(1), &rated_items, &item_profiles).unwrap();

	let recommender = ContentBasedRecommender::new(Box::new(CosineSimilarity));
	let exclude: HashSet<ItemId> = [ItemId(1), ItemId(3)].into_iter().collect();

	// Act
	let recs = recommender
		.recommend(&user_profile, &item_profiles, 3, &exclude)
		.unwrap();

	// Assert
	assert_eq!(recs.len(), 3);
	verify_valid_recommendations(&recs);
	// Items 2, 4, 5 share no terms with items 1 and 3
	// All should have score 0.0 since cosine similarity with disjoint terms is 0
	for rec in &recs {
		assert!(!exclude.contains(&rec.item_id));
	}
}

#[rstest]
fn cosine_vs_euclidean_both_produce_valid_results() {
	// Arrange
	let docs = vec![
		make_doc(1, &["alpha", "beta", "gamma"]),
		make_doc(2, &["alpha", "delta", "epsilon"]),
		make_doc(3, &["beta", "gamma", "zeta"]),
		make_doc(4, &["delta", "epsilon", "eta"]),
		make_doc(5, &["theta", "iota", "kappa"]),
	];

	let mut builder = ItemProfileBuilder::new(TfIdfConfig::default());
	let item_profiles = builder.build_profiles(&docs).unwrap();

	let rated_items = vec![(ItemId(1), 5.0)];
	let user_profile =
		UserProfileBuilder::build_profile(UserId(1), &rated_items, &item_profiles).unwrap();
	let exclude: HashSet<ItemId> = [ItemId(1)].into_iter().collect();

	// Act
	let cosine_recommender = ContentBasedRecommender::new(Box::new(CosineSimilarity));
	let euclidean_recommender = ContentBasedRecommender::new(Box::new(EuclideanDistance));

	let cosine_recs = cosine_recommender
		.recommend(&user_profile, &item_profiles, 4, &exclude)
		.unwrap();
	let euclidean_recs = euclidean_recommender
		.recommend(&user_profile, &item_profiles, 4, &exclude)
		.unwrap();

	// Assert — both produce valid sorted results of the same length
	assert_eq!(cosine_recs.len(), 4);
	assert_eq!(euclidean_recs.len(), 4);
	verify_valid_recommendations(&cosine_recs);
	verify_valid_recommendations(&euclidean_recs);

	// Both should rank item 3 (shares "beta", "gamma") or item 2 (shares "alpha") highly
	// The top item for both metrics should be among items sharing terms with item 1
	let top_cosine = cosine_recs[0].item_id;
	let top_euclidean = euclidean_recs[0].item_id;
	let sharing_items: HashSet<ItemId> = [ItemId(2), ItemId(3)].into_iter().collect();
	assert!(sharing_items.contains(&top_cosine));
	assert!(sharing_items.contains(&top_euclidean));
}

#[rstest]
fn tfidf_config_sublinear_min_df_max_df() {
	// Arrange
	let config = TfIdfConfig {
		sublinear_tf: true,
		min_df: 2,
		max_df_ratio: 0.5,
	};
	// 4 docs, max_df = ceil(0.5 * 4) = 2
	let docs = vec![
		make_doc(1, &["common", "rare_a", "everywhere"]),
		make_doc(2, &["common", "rare_b", "everywhere"]),
		make_doc(3, &["unique", "everywhere"]),
		make_doc(4, &["solo", "everywhere"]),
	];

	let mut builder = ItemProfileBuilder::new(config);

	// Act
	let profiles = builder.build_profiles(&docs).unwrap();

	// Assert
	// "everywhere" appears in 4/4 docs (ratio=1.0 > 0.5) — filtered out
	// "common" appears in 2/4 docs (ratio=0.5, max_df=ceil(2)=2) — included
	// "rare_a", "rare_b", "unique", "solo" each appear in 1 doc (< min_df=2) — filtered out
	// Only "common" should survive
	assert_eq!(profiles.len(), 4);
	assert_eq!(profiles[0].features.dimension(), 1);
}

#[rstest]
fn plugin_registration_metadata_and_capabilities() {
	// Act
	use reinhardt_dentdelion::plugin::registered_plugins;
	use reinhardt_dentdelion::prelude::Capability;
	let plugins: Vec<_> = registered_plugins().collect();

	// Assert
	let plugin = plugins
		.iter()
		.find(|p| p.metadata().name == "content-based-delion");
	assert!(plugin.is_some(), "content-based-delion plugin not found");

	let plugin = plugin.unwrap();
	assert_eq!(plugin.metadata().version.to_string(), "0.1.0");

	let caps = plugin.capabilities();
	let cap_names: Vec<String> = caps
		.iter()
		.filter_map(|c| match c {
			Capability::Custom(name) => Some(name.clone()),
			_ => None,
		})
		.collect();
	assert!(cap_names.contains(&"recommendation".to_string()));
	assert!(cap_names.contains(&"content-analysis".to_string()));
	assert!(cap_names.contains(&"tfidf".to_string()));
}
