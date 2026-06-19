use reinhardt::dentdelion::prelude::{
	ArcPlugin, Capability, Plugin, PluginLifecycle, PluginMetadata, register_plugin,
};
use std::sync::Arc;

pub struct AxumDiPlugin {
	metadata: PluginMetadata,
	capabilities: Vec<Capability>,
}

impl AxumDiPlugin {
	/// Creates plugin metadata from crate metadata.
	///
	/// # Panics
	///
	/// Panics if the crate version is not valid semantic version metadata.
	#[must_use]
	pub fn new() -> Self {
		let capabilities = vec![
			Capability::custom("axum"),
			Capability::custom("di"),
			Capability::custom("integration"),
		];
		let metadata = PluginMetadata::builder(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
			.description(env!("CARGO_PKG_DESCRIPTION"))
			.license(env!("CARGO_PKG_LICENSE"))
			.repository(env!("CARGO_PKG_REPOSITORY"))
			.provides(Capability::custom("axum"))
			.provides(Capability::custom("di"))
			.provides(Capability::custom("integration"))
			.build()
			.expect("crate metadata must describe a valid axum-di-delion plugin");

		Self {
			metadata,
			capabilities,
		}
	}
}

impl Default for AxumDiPlugin {
	fn default() -> Self {
		Self::new()
	}
}

impl Plugin for AxumDiPlugin {
	fn metadata(&self) -> &PluginMetadata {
		&self.metadata
	}

	fn capabilities(&self) -> &[Capability] {
		self.capabilities.as_slice()
	}
}

impl PluginLifecycle for AxumDiPlugin {}

fn create_plugin() -> ArcPlugin {
	Arc::new(AxumDiPlugin::new())
}

register_plugin!(create_plugin);
