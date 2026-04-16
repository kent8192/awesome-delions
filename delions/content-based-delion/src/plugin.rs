use std::sync::Arc;

use reinhardt_dentdelion::prelude::*;

/// Content-based recommendation plugin for the reinhardt ecosystem.
struct ContentBasedPlugin {
	metadata: PluginMetadata,
	capabilities: Vec<Capability>,
}

impl ContentBasedPlugin {
	fn new() -> Self {
		Self {
			metadata: PluginMetadata::builder("content-based-delion", "0.1.0")
				.description("TF-IDF and feature similarity content-based recommendation plugin")
				.build()
				.unwrap(),
			capabilities: vec![
				Capability::Custom("recommendation".to_string()),
				Capability::Custom("content-analysis".to_string()),
				Capability::Custom("tfidf".to_string()),
			],
		}
	}
}

impl Plugin for ContentBasedPlugin {
	fn metadata(&self) -> &PluginMetadata {
		&self.metadata
	}

	fn capabilities(&self) -> &[Capability] {
		&self.capabilities
	}
}

#[async_trait]
impl PluginLifecycle for ContentBasedPlugin {}

register_plugin!(|| Arc::new(ContentBasedPlugin::new()));
