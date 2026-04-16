use collaborative_filtering_delion::{
	CollaborativeFilteringConfig, CosineSimilarity, ItemBasedRecommender, ItemId,
	JaccardSimilarity, PearsonCorrelation, Rating, Recommender, SparseRatingMatrix,
	UserBasedRecommender, UserId,
};
use rstest::{fixture, rstest};

#[fixture]
fn movie_ratings() -> SparseRatingMatrix {
	// A small movie recommendation dataset
	// User 1: action fan (high on action items 1,2; low on romance 3)
	// User 2: similar to user 1 (action fan)
	// User 3: romance fan (high on romance 3,4; low on action 1)
	// User 4: mixed tastes
	let ratings = vec![
		Rating {
			user_id: UserId(1),
			item_id: ItemId(1),
			value: 5.0,
		},
		Rating {
			user_id: UserId(1),
			item_id: ItemId(2),
			value: 4.5,
		},
		Rating {
			user_id: UserId(1),
			item_id: ItemId(3),
			value: 1.0,
		},
		Rating {
			user_id: UserId(2),
			item_id: ItemId(1),
			value: 4.0,
		},
		Rating {
			user_id: UserId(2),
			item_id: ItemId(2),
			value: 5.0,
		},
		Rating {
			user_id: UserId(2),
			item_id: ItemId(4),
			value: 2.0,
		},
		Rating {
			user_id: UserId(3),
			item_id: ItemId(1),
			value: 1.5,
		},
		Rating {
			user_id: UserId(3),
			item_id: ItemId(3),
			value: 5.0,
		},
		Rating {
			user_id: UserId(3),
			item_id: ItemId(4),
			value: 4.5,
		},
		Rating {
			user_id: UserId(4),
			item_id: ItemId(1),
			value: 3.0,
		},
		Rating {
			user_id: UserId(4),
			item_id: ItemId(2),
			value: 3.5,
		},
		Rating {
			user_id: UserId(4),
			item_id: ItemId(3),
			value: 3.0,
		},
		Rating {
			user_id: UserId(4),
			item_id: ItemId(4),
			value: 3.5,
		},
	];
	SparseRatingMatrix::from_ratings(&ratings)
}

#[rstest]
fn end_to_end_user_based_pipeline(movie_ratings: SparseRatingMatrix) {
	// Arrange
	let recommender = UserBasedRecommender::new(Box::new(CosineSimilarity));
	let config = CollaborativeFilteringConfig {
		k_neighbors: 10,
		min_similarity: 0.0,
	};

	// Act: recommend top-2 items for user 1 (has rated items 1,2,3)
	let recs = recommender
		.recommend(&movie_ratings, UserId(1), 2, &config)
		.unwrap();

	// Assert: recommendations should be for unrated item 4 only
	assert!(!recs.is_empty());
	assert!(recs.len() <= 2);
	for rec in &recs {
		// User 1 has rated items 1,2,3 so recommendations should only be item 4
		assert_eq!(rec.item_id, ItemId(4));
		assert!(rec.score.is_finite());
	}
}

#[rstest]
fn all_similarity_metrics_produce_valid_results(movie_ratings: SparseRatingMatrix) {
	// Arrange
	let config = CollaborativeFilteringConfig::default();
	let cosine = UserBasedRecommender::new(Box::new(CosineSimilarity));
	let pearson = UserBasedRecommender::new(Box::new(PearsonCorrelation));
	let jaccard = UserBasedRecommender::new(Box::new(JaccardSimilarity));

	// Act
	let recs_cosine = cosine
		.recommend(&movie_ratings, UserId(1), 5, &config)
		.unwrap();
	let recs_pearson = pearson
		.recommend(&movie_ratings, UserId(1), 5, &config)
		.unwrap();
	let recs_jaccard = jaccard
		.recommend(&movie_ratings, UserId(1), 5, &config)
		.unwrap();

	// Assert: all metrics produce non-empty results with finite scores
	assert!(!recs_cosine.is_empty());
	assert!(!recs_pearson.is_empty());
	assert!(!recs_jaccard.is_empty());

	for rec in recs_cosine
		.iter()
		.chain(recs_pearson.iter())
		.chain(recs_jaccard.iter())
	{
		assert!(rec.score.is_finite());
	}
}

#[rstest]
fn user_based_vs_item_based_both_valid(movie_ratings: SparseRatingMatrix) {
	// Arrange
	let config = CollaborativeFilteringConfig::default();
	let user_based = UserBasedRecommender::new(Box::new(CosineSimilarity));
	let item_based = ItemBasedRecommender::new(Box::new(CosineSimilarity));

	// Act
	let recs_user = user_based
		.recommend(&movie_ratings, UserId(1), 5, &config)
		.unwrap();
	let recs_item = item_based
		.recommend(&movie_ratings, UserId(1), 5, &config)
		.unwrap();

	// Assert: both produce valid recommendations for unrated items
	assert!(!recs_user.is_empty());
	assert!(!recs_item.is_empty());

	let rated_items = [ItemId(1), ItemId(2), ItemId(3)];
	for rec in recs_user.iter().chain(recs_item.iter()) {
		assert!(!rated_items.contains(&rec.item_id));
		assert!(rec.score.is_finite());
	}
}

#[rstest]
fn large_dataset_no_panics() {
	// Arrange: 100 users x 50 items synthetic data
	let ratings: Vec<Rating> = (0..100_u64)
		.flat_map(|u| {
			// Each user rates about half the items (even-indexed + some based on user)
			(0..50_u64).filter_map(move |i| {
				if (u + i) % 3 != 0 {
					Some(Rating {
						user_id: UserId(u),
						item_id: ItemId(i),
						// Precision loss is acceptable for test data generation
						#[allow(clippy::cast_precision_loss)]
						value: ((u * 7 + i * 3) % 5 + 1) as f64,
					})
				} else {
					None
				}
			})
		})
		.collect();

	let matrix = SparseRatingMatrix::from_ratings(&ratings);

	// Act
	let config = CollaborativeFilteringConfig {
		k_neighbors: 10,
		min_similarity: 0.0,
	};
	let user_rec = UserBasedRecommender::new(Box::new(CosineSimilarity));
	let item_rec = ItemBasedRecommender::new(Box::new(CosineSimilarity));

	let recs_user = user_rec.recommend(&matrix, UserId(0), 10, &config);
	let recs_item = item_rec.recommend(&matrix, UserId(0), 10, &config);

	// Assert: no panics and results are valid
	assert!(recs_user.is_ok());
	assert!(recs_item.is_ok());

	let recs_user = recs_user.unwrap();
	let recs_item = recs_item.unwrap();
	assert!(recs_user.len() <= 10);
	assert!(recs_item.len() <= 10);

	for rec in recs_user.iter().chain(recs_item.iter()) {
		assert!(rec.score.is_finite());
	}
}

#[rstest]
fn plugin_registration_metadata() {
	use reinhardt_dentdelion::prelude::{Capability, PluginRegistration};

	// Arrange: collect registered plugins via inventory
	let plugins: Vec<_> = inventory::iter::<PluginRegistration>().collect();

	// Assert: the collaborative-filtering plugin should be registered
	let cf_plugin = plugins
		.iter()
		.find(|reg| {
			let plugin = (reg.factory)();
			plugin.name() == "collaborative-filtering-delion"
		})
		.expect("collaborative-filtering-delion plugin should be registered");

	let plugin = (cf_plugin.factory)();
	assert_eq!(plugin.name(), "collaborative-filtering-delion");
	assert_eq!(plugin.version().to_string(), "0.1.0");

	let capabilities = plugin.capabilities();
	assert_eq!(capabilities.len(), 3);
	assert!(matches!(
		&capabilities[0],
		Capability::Custom(s) if s == "recommendation"
	));
	assert!(matches!(
		&capabilities[1],
		Capability::Custom(s) if s == "user-based"
	));
	assert!(matches!(
		&capabilities[2],
		Capability::Custom(s) if s == "item-based"
	));
}
