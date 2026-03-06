//! Shared state holding Reinhardt services for axum integration.

use std::sync::Arc;

use reinhardt::InjectionContext;

/// Shared state that holds Reinhardt services accessible from axum handlers.
///
/// This state is injected into axum request extensions by [`ReinhardtLayer`](crate::ReinhardtLayer).
#[derive(Clone)]
pub struct ReinhardtState {
	injection_context: Arc<InjectionContext>,
}

impl ReinhardtState {
	/// Creates a new `ReinhardtState` with the given injection context.
	#[must_use]
	pub fn new(injection_context: InjectionContext) -> Self {
		Self {
			injection_context: Arc::new(injection_context),
		}
	}

	/// Returns a reference to the injection context.
	#[must_use]
	pub fn injection_context(&self) -> &InjectionContext {
		&self.injection_context
	}
}
