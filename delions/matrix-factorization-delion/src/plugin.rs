use std::sync::Arc;

use reinhardt_dentdelion::prelude::*;

struct MatrixFactorizationPlugin {
	metadata: PluginMetadata,
	capabilities: Vec<Capability>,
}

impl MatrixFactorizationPlugin {
	fn new() -> Self {
		Self {
			metadata: PluginMetadata::builder("matrix-factorization-delion", "0.1.0")
				.description("SVD and ALS matrix factorization recommendation plugin")
				.build()
				.unwrap(),
			capabilities: vec![
				Capability::Custom("recommendation".into()),
				Capability::Custom("matrix-factorization".into()),
				Capability::Custom("svd".into()),
				Capability::Custom("als".into()),
			],
		}
	}
}

impl Plugin for MatrixFactorizationPlugin {
	fn metadata(&self) -> &PluginMetadata {
		&self.metadata
	}

	fn capabilities(&self) -> &[Capability] {
		&self.capabilities
	}
}

#[async_trait]
impl PluginLifecycle for MatrixFactorizationPlugin {}

register_plugin!(|| Arc::new(MatrixFactorizationPlugin::new()));
