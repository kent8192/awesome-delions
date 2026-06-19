use reinhardt::di::{InjectionContext, SingletonScope};
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::RequestDiContext;

#[derive(Clone)]
pub struct DiLayer {
	root_context: Arc<InjectionContext>,
}

impl DiLayer {
	#[must_use]
	pub fn new(root_context: InjectionContext) -> Self {
		Self::from_arc(Arc::new(root_context))
	}

	#[must_use]
	pub fn from_arc(root_context: Arc<InjectionContext>) -> Self {
		Self { root_context }
	}

	#[must_use]
	pub fn from_singleton_scope(singleton_scope: impl Into<Arc<SingletonScope>>) -> Self {
		Self::new(InjectionContext::builder(singleton_scope).build())
	}

	#[must_use]
	pub fn root_context(&self) -> &InjectionContext {
		self.root_context.as_ref()
	}
}

#[derive(Clone)]
pub struct DiService<S> {
	inner: S,
	root_context: Arc<InjectionContext>,
}

impl<S> DiService<S> {
	#[must_use]
	pub fn new(inner: S, root_context: Arc<InjectionContext>) -> Self {
		Self {
			inner,
			root_context,
		}
	}

	#[must_use]
	pub fn inner(&self) -> &S {
		&self.inner
	}

	#[must_use]
	pub fn root_context(&self) -> &InjectionContext {
		self.root_context.as_ref()
	}
}

impl<S> Layer<S> for DiLayer {
	type Service = DiService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		DiService::new(inner, Arc::clone(&self.root_context))
	}
}

impl<S, B> Service<axum::http::Request<B>> for DiService<S>
where
	S: Service<axum::http::Request<B>>,
{
	type Error = S::Error;
	type Future = S::Future;
	type Response = S::Response;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, mut request: axum::http::Request<B>) -> Self::Future {
		let request_context = RequestDiContext::new(self.root_context.fork());
		request.extensions_mut().insert(request_context);
		self.inner.call(request)
	}
}
