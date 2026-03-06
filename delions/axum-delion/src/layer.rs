//! Tower Layer for injecting `ReinhardtState` into axum requests.

use std::task::{Context, Poll};

use axum::body::Body;
use http::Request;
use tower::Layer;
use tower::Service;

use crate::ReinhardtState;

/// A Tower Layer that injects [`ReinhardtState`] into request extensions.
///
/// Use [`ReinhardtLayer::builder()`] to construct this layer.
#[derive(Clone)]
pub struct ReinhardtLayer {
	state: ReinhardtState,
}

impl ReinhardtLayer {
	/// Creates a new builder for configuring the layer.
	#[must_use]
	pub fn builder() -> ReinhardtLayerBuilder {
		ReinhardtLayerBuilder::default()
	}
}

impl<S> Layer<S> for ReinhardtLayer {
	type Service = ReinhardtService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		ReinhardtService {
			inner,
			state: self.state.clone(),
		}
	}
}

/// Builder for [`ReinhardtLayer`].
#[derive(Default)]
pub struct ReinhardtLayerBuilder {
	injection_context: Option<reinhardt::InjectionContext>,
}

impl ReinhardtLayerBuilder {
	/// Sets a custom injection context.
	pub fn injection_context(mut self, ctx: reinhardt::InjectionContext) -> Self {
		self.injection_context = Some(ctx);
		self
	}

	/// Builds the [`ReinhardtLayer`].
	#[must_use]
	pub fn build(self) -> ReinhardtLayer {
		let injection_context = self.injection_context.unwrap_or_else(|| {
			reinhardt::InjectionContext::builder(reinhardt::SingletonScope::new()).build()
		});

		let state = ReinhardtState::new(injection_context);
		ReinhardtLayer { state }
	}
}

/// Tower Service that injects `ReinhardtState` into each request's extensions.
#[derive(Clone)]
pub struct ReinhardtService<S> {
	inner: S,
	state: ReinhardtState,
}

impl<S> Service<Request<Body>> for ReinhardtService<S>
where
	S: Service<Request<Body>> + Clone + Send + 'static,
	S::Future: Send,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future = S::Future;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, mut req: Request<Body>) -> Self::Future {
		req.extensions_mut().insert(self.state.clone());
		self.inner.call(req)
	}
}
