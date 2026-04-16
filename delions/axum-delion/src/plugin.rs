//! Dentdelion plugin implementation for axum integration.

use reinhardt::dentdelion::capability::{Capability, PluginCapability};
use reinhardt::dentdelion::context::PluginContext;
use reinhardt::dentdelion::error::PluginError;
use reinhardt::dentdelion::metadata::PluginMetadata;
use reinhardt::dentdelion::plugin::{Plugin, PluginLifecycle};

/// Axum integration plugin for the Reinhardt dentdelion plugin system.
pub struct AxumDelionPlugin {
	metadata: PluginMetadata,
	capabilities: Vec<Capability>,
}

impl AxumDelionPlugin {
	/// Creates a new instance of the axum integration plugin.
	///
	/// # Panics
	///
	/// Panics if plugin metadata construction fails (should never happen with valid constants).
	#[must_use]
	pub fn new() -> Self {
		let metadata = PluginMetadata::builder("axum-delion", "0.1.0")
			.description("Axum web framework integration for Reinhardt")
			.license("MIT OR Apache-2.0")
			.provides(PluginCapability::Services)
			.provides(PluginCapability::Middleware)
			.build()
			.expect("valid plugin metadata");

		Self {
			metadata,
			capabilities: vec![
				Capability::Core(PluginCapability::Services),
				Capability::Core(PluginCapability::Middleware),
			],
		}
	}
}

impl Default for AxumDelionPlugin {
	fn default() -> Self {
		Self::new()
	}
}

impl Plugin for AxumDelionPlugin {
	fn metadata(&self) -> &PluginMetadata {
		&self.metadata
	}

	fn capabilities(&self) -> &[Capability] {
		&self.capabilities
	}
}

#[async_trait::async_trait]
impl PluginLifecycle for AxumDelionPlugin {
	async fn on_enable(&self, _ctx: &PluginContext) -> Result<(), PluginError> {
		Ok(())
	}
}
