//! Axum extractors for Reinhardt types.

use axum::extract::FromRequestParts;
use http::request::Parts;

use crate::ReinhardtState;

/// Extractor that resolves a type `T` from the Reinhardt injection context.
///
/// # Example
///
/// ```rust,ignore
/// use axum_delion::Inject;
/// use reinhardt::InjectionContext;
///
/// async fn handler(Inject(ctx): Inject<ReinhardtState>) -> String {
///     format!("Got injection context")
/// }
/// ```
pub struct Inject<T>(pub T);

impl FromRequestParts<()> for Inject<ReinhardtState> {
	type Rejection = axum::response::Response;

	async fn from_request_parts(parts: &mut Parts, _state: &()) -> Result<Self, Self::Rejection> {
		let state = parts
			.extensions
			.get::<ReinhardtState>()
			.cloned()
			.ok_or_else(|| {
				axum::response::Response::builder()
					.status(500)
					.body(axum::body::Body::from(
						"ReinhardtState not found in request extensions. Did you add ReinhardtLayer?",
					))
					.unwrap()
			})?;
		Ok(Inject(state))
	}
}
