use reinhardt_dentdelion::prelude::*;
use std::sync::Arc;

/// Collaborative filtering recommendation plugin.
struct CollaborativeFilteringPlugin {
	metadata: PluginMetadata,
	capabilities: Vec<Capability>,
}

impl CollaborativeFilteringPlugin {
	fn new() -> Self {
		Self {
			metadata: PluginMetadata::builder("collaborative-filtering-delion", "0.1.0")
				.description(
					"User-based and item-based collaborative filtering recommendation plugin",
				)
				.build()
				.unwrap(),
			capabilities: vec![
				Capability::Custom("recommendation".to_string()),
				Capability::Custom("user-based".to_string()),
				Capability::Custom("item-based".to_string()),
			],
		}
	}
}

impl Plugin for CollaborativeFilteringPlugin {
	fn metadata(&self) -> &PluginMetadata {
		&self.metadata
	}

	fn capabilities(&self) -> &[Capability] {
		&self.capabilities
	}
}

#[async_trait]
impl PluginLifecycle for CollaborativeFilteringPlugin {}

register_plugin!(|| Arc::new(CollaborativeFilteringPlugin::new()));
