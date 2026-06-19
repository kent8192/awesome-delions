use reinhardt::di::{InjectionContext, SingletonScope};
use std::sync::Arc;

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
