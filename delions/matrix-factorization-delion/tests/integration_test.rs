use approx::assert_abs_diff_eq;
use rstest::{fixture, rstest};

use matrix_factorization_delion::{
	AlsFactorizer, Factorizer, FactorizerKind, ItemId, MatrixFactorizationRecommender, ModelConfig,
	Rating, RatingMatrix, SvdFactorizer, UserId,
};

#[fixture]
fn standard_ratings() -> Vec<Rating> {
	vec![
		Rating {
			user_id: UserId(0),
			item_id: ItemId(0),
			value: 5.0,
		},
		Rating {
			user_id: UserId(0),
			item_id: ItemId(1),
			value: 3.0,
		},
		Rating {
			user_id: UserId(0),
			item_id: ItemId(2),
			value: 1.0,
		},
		Rating {
			user_id: UserId(1),
			item_id: ItemId(0),
			value: 4.0,
		},
		Rating {
			user_id: UserId(1),
			item_id: ItemId(2),
			value: 2.0,
		},
		Rating {
			user_id: UserId(1),
			item_id: ItemId(3),
			value: 5.0,
		},
		Rating {
			user_id: UserId(2),
			item_id: ItemId(1),
			value: 1.0,
		},
		Rating {
			user_id: UserId(2),
			item_id: ItemId(2),
			value: 4.0,
		},
		Rating {
			user_id: UserId(2),
			item_id: ItemId(3),
			value: 3.0,
		},
	]
}

#[fixture]
fn default_config() -> ModelConfig {
	ModelConfig {
		n_factors: 3,
		regularization: 0.01,
		max_iterations: 200,
		tolerance: 1e-6,
		..ModelConfig::default()
	}
}

#[rstest]
fn full_pipeline_svd(standard_ratings: Vec<Rating>, default_config: ModelConfig) {
	// Arrange
	let recommender = MatrixFactorizationRecommender;
	let factorizer = SvdFactorizer;

	// Act: train
	let (model, matrix) = recommender
		.train(&standard_ratings, &default_config, &factorizer)
		.unwrap();

	// Act: predict an observed entry
	let pred = recommender
		.predict(&model, &matrix, UserId(0), ItemId(0))
		.unwrap();

	// Assert: prediction should be near the observed value 5.0
	assert_abs_diff_eq!(pred, 5.0, epsilon = 1.5);

	// Act: recommend for user 0 (has not rated item 3)
	let recs = recommender
		.recommend(&model, &matrix, UserId(0), 5)
		.unwrap();

	// Assert: only unrated items should be recommended
	assert_eq!(recs.len(), 1);
	assert_eq!(recs[0].item_id, ItemId(3));
	assert!(recs[0].score.is_finite());
}

#[rstest]
fn svd_and_als_predictions_in_reasonable_range(
	standard_ratings: Vec<Rating>,
	default_config: ModelConfig,
) {
	// Arrange
	let matrix = RatingMatrix::from_ratings(&standard_ratings).unwrap();

	// Act: factorize with both algorithms
	let svd_model = SvdFactorizer.factorize(&matrix, &default_config).unwrap();
	let als_model = AlsFactorizer.factorize(&matrix, &default_config).unwrap();

	// Assert: both models produce predictions in a reasonable range for all entries
	for u in 0..matrix.n_users() {
		for i in 0..matrix.n_items() {
			let svd_pred = svd_model.predict(u, i);
			let als_pred = als_model.predict(u, i);

			// NOTE: exact values are non-deterministic across algorithms,
			// but all predictions should be in a plausible rating range
			assert!(
				(-3.0..=10.0).contains(&svd_pred),
				"SVD prediction {svd_pred} out of range for ({u}, {i})"
			);
			assert!(
				(-3.0..=10.0).contains(&als_pred),
				"ALS prediction {als_pred} out of range for ({u}, {i})"
			);
		}
	}
}

#[rstest]
fn different_model_configs(standard_ratings: Vec<Rating>) {
	// Arrange
	let matrix = RatingMatrix::from_ratings(&standard_ratings).unwrap();
	let configs = vec![
		ModelConfig {
			n_factors: 1,
			regularization: 0.01,
			max_iterations: 100,
			tolerance: 1e-4,
			..ModelConfig::default()
		},
		ModelConfig {
			n_factors: 5,
			regularization: 0.5,
			max_iterations: 100,
			tolerance: 1e-4,
			..ModelConfig::default()
		},
		ModelConfig {
			n_factors: 3,
			regularization: 0.001,
			max_iterations: 50,
			tolerance: 1e-3,
			..ModelConfig::default()
		},
	];

	for config in &configs {
		// Act: both factorizers should succeed with each config
		let svd_model = SvdFactorizer.factorize(&matrix, config).unwrap();
		let als_model = AlsFactorizer.factorize(&matrix, config).unwrap();

		// Assert: models have correct factor count
		assert_eq!(svd_model.n_factors(), config.n_factors);
		assert_eq!(als_model.n_factors(), config.n_factors);

		// Assert: all predictions should be finite
		for u in 0..matrix.n_users() {
			for i in 0..matrix.n_items() {
				assert!(svd_model.predict(u, i).is_finite());
				assert!(als_model.predict(u, i).is_finite());
			}
		}
	}
}

#[rstest]
fn svd_reproducibility(standard_ratings: Vec<Rating>, default_config: ModelConfig) {
	// Arrange
	let matrix = RatingMatrix::from_ratings(&standard_ratings).unwrap();

	// Act: factorize twice with same seed (SvdFactorizer uses seed 42 internally)
	let model1 = SvdFactorizer.factorize(&matrix, &default_config).unwrap();
	let model2 = SvdFactorizer.factorize(&matrix, &default_config).unwrap();

	// Assert: predictions should be identical across both runs
	for u in 0..matrix.n_users() {
		for i in 0..matrix.n_items() {
			assert_abs_diff_eq!(model1.predict(u, i), model2.predict(u, i), epsilon = 1e-12);
		}
	}
}

#[rstest]
fn als_reproducibility(standard_ratings: Vec<Rating>, default_config: ModelConfig) {
	// Arrange
	let matrix = RatingMatrix::from_ratings(&standard_ratings).unwrap();

	// Act: factorize twice with same seed (AlsFactorizer uses seed 42 internally)
	let model1 = AlsFactorizer.factorize(&matrix, &default_config).unwrap();
	let model2 = AlsFactorizer.factorize(&matrix, &default_config).unwrap();

	// Assert: predictions should be identical across both runs
	for u in 0..matrix.n_users() {
		for i in 0..matrix.n_items() {
			assert_abs_diff_eq!(model1.predict(u, i), model2.predict(u, i), epsilon = 1e-12);
		}
	}
}

#[rstest]
fn plugin_registration_via_inventory() {
	// Act: iterate all registered plugins from this crate
	let plugins: Vec<_> = reinhardt_dentdelion::plugin::registered_plugins().collect();

	// Assert: at least one plugin registered
	assert!(
		!plugins.is_empty(),
		"expected at least one registered plugin"
	);

	// Find the matrix factorization plugin by name
	let mf_plugin = plugins
		.iter()
		.find(|p| p.metadata().name == "matrix-factorization-delion");
	assert!(
		mf_plugin.is_some(),
		"matrix-factorization-delion plugin not found in registry"
	);

	let plugin = mf_plugin.unwrap();
	assert_eq!(plugin.metadata().version.to_string(), "0.1.0");
	assert_eq!(
		plugin.metadata().description,
		"SVD and ALS matrix factorization recommendation plugin"
	);
	assert!(!plugin.capabilities().is_empty());
}

#[rstest]
fn factorizer_kind_into_factorizer(standard_ratings: Vec<Rating>) {
	// Arrange
	let matrix = RatingMatrix::from_ratings(&standard_ratings).unwrap();
	let config = ModelConfig {
		n_factors: 2,
		max_iterations: 50,
		tolerance: 1e-4,
		..ModelConfig::default()
	};

	// Act: use FactorizerKind to create factorizers dynamically
	let svd: Box<dyn Factorizer> = FactorizerKind::Svd.into_factorizer();
	let als: Box<dyn Factorizer> = FactorizerKind::Als.into_factorizer();

	let svd_model = svd.factorize(&matrix, &config).unwrap();
	let als_model = als.factorize(&matrix, &config).unwrap();

	// Assert: both produce valid models
	assert_eq!(svd_model.n_factors(), 2);
	assert_eq!(als_model.n_factors(), 2);
}
