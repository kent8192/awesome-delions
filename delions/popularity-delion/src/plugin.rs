use std::sync::Arc;

use reinhardt_dentdelion::prelude::*;

/// Popularity-based recommendation plugin for the reinhardt framework.
pub struct PopularityPlugin {
	metadata: PluginMetadata,
	capabilities: Vec<Capability>,
}

impl PopularityPlugin {
	/// Creates a new `PopularityPlugin` instance.
	///
	/// # Panics
	///
	/// Panics if the plugin metadata builder fails (should never happen with
	/// valid hardcoded values).
	#[must_use]
	pub fn new() -> Self {
		Self {
			metadata: PluginMetadata::builder("popularity-delion", "0.1.0")
				.description("Time-decay popularity-based recommendation plugin")
				.build()
				.expect("valid plugin metadata"),
			capabilities: vec![
				Capability::Custom("recommendation".to_string()),
				Capability::Custom("popularity".to_string()),
				Capability::Custom("trending".to_string()),
			],
		}
	}
}

impl Default for PopularityPlugin {
	fn default() -> Self {
		Self::new()
	}
}

impl Plugin for PopularityPlugin {
	fn metadata(&self) -> &PluginMetadata {
		&self.metadata
	}

	fn capabilities(&self) -> &[Capability] {
		&self.capabilities
	}
}

#[async_trait]
impl PluginLifecycle for PopularityPlugin {}

register_plugin!(|| Arc::new(PopularityPlugin::new()));
