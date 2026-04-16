//! Axum web framework integration for Reinhardt.
//!
//! This crate provides a dentdelion plugin that bridges Reinhardt's
//! dependency injection, ORM, and authentication with axum's ecosystem.
//!
//! # Usage
//!
//! ```rust,ignore
//! use axum::{Router, routing::get};
//! use axum_delion::{ReinhardtLayer, Inject};
//!
//! async fn handler(Inject(ctx): Inject<reinhardt::InjectionContext>) -> &'static str {
//!     "Hello from Reinhardt + Axum!"
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let layer = ReinhardtLayer::builder().build();
//!     let app = Router::new()
//!         .route("/", get(handler))
//!         .layer(layer);
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

mod extractors;
mod layer;
mod plugin;
mod state;

pub use extractors::Inject;
pub use layer::ReinhardtLayer;
pub use plugin::AxumDelionPlugin;
pub use state::ReinhardtState;
