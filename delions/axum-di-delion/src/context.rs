use reinhardt::di::InjectionContext;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestDiContext {
	injection_context: Arc<InjectionContext>,
}

impl RequestDiContext {
	#[must_use]
	pub fn new(injection_context: InjectionContext) -> Self {
		Self::from_arc(Arc::new(injection_context))
	}

	#[must_use]
	pub fn from_arc(injection_context: Arc<InjectionContext>) -> Self {
		Self { injection_context }
	}

	#[must_use]
	pub fn injection_context(&self) -> &InjectionContext {
		self.injection_context.as_ref()
	}

	#[must_use]
	pub fn as_arc(&self) -> Arc<InjectionContext> {
		Arc::clone(&self.injection_context)
	}
}
