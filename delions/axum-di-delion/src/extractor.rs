use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use reinhardt::di::Injectable;

use crate::{DiRejection, RequestDiContext};

/// Axum extractor that resolves `T` from the request DI context.
pub struct Di<T>(pub T);

impl<S, T> FromRequestParts<S> for Di<T>
where
	T: Injectable,
{
	type Rejection = DiRejection;

	fn from_request_parts(
		parts: &mut Parts,
		_state: &S,
	) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		let request_context = parts.extensions.get::<RequestDiContext>().cloned();

		async move {
			let request_context = request_context.ok_or(DiRejection::MissingContext)?;
			let value = T::inject(request_context.injection_context())
				.await
				.map_err(DiRejection::from)?;

			Ok(Self(value))
		}
	}
}
