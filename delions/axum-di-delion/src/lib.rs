//! # axum-di-delion
//!
//! Axum integration delion for Reinhardt dependency injection.
//!
//! This crate defines [`DiLayer`] and [`Di`] as the public API for pairing
//! standalone Axum handlers with `reinhardt::di` dependencies while keeping
//! Axum's standard extractors for request data.
//!
//! ## Example
//!
//! The public API is designed for this handler shape:
//!
//! ```rust,ignore
//! use axum::{Router, extract::Path, routing::get};
//! use axum_di_delion::{Di, DiLayer};
//! use reinhardt::di::{Depends, InjectableKey, InjectionContext, SingletonScope};
//! use std::sync::Arc;
//!
//! struct AuthKey;
//! impl InjectableKey for AuthKey {}
//!
//! struct AuthService;
//! impl AuthService {
//!     async fn authorize(&self, id: u64) -> String {
//!         format!("authorized:{id}")
//!     }
//! }
//!
//! async fn handler(
//!     Di(auth): Di<Depends<AuthKey, AuthService>>,
//!     Path(id): Path<u64>,
//! ) -> String {
//!     auth.authorize(id).await
//! }
//!
//! let singleton = Arc::new(SingletonScope::new());
//! let root_context = InjectionContext::builder(singleton).build();
//! let app = Router::new()
//!     .route("/users/{id}", get(handler))
//!     .layer(DiLayer::new(root_context));
//! # let _ = app;
//! ```
//!
//! ## Planned Features
//!
//! - Optional handler macro sugar for users who want a more compact annotated
//!   handler form.

mod context;
mod error;
mod extractor;
mod layer;
mod plugin;

pub use context::RequestDiContext;
pub use error::DiRejection;
pub use extractor::Di;
pub use layer::{DiLayer, DiService};
pub use plugin::AxumDiPlugin;
