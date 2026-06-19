use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum_di_delion::{Di, DiLayer, DiRejection};
use reinhardt::di::{
	DependencyRegistry, DependencyScope, Depends, DiError, DiResult, FactoryOutput, Injectable,
	InjectableKey, InjectionContext, SingletonScope,
};
use rstest::rstest;
use tower::ServiceExt;

#[derive(Clone, Debug)]
struct AuthService {
	prefix: &'static str,
}

impl AuthService {
	fn authorize(&self, id: u64) -> String {
		format!("{}:{id}", self.prefix)
	}
}

#[derive(Debug)]
struct AuthKey;

impl InjectableKey for AuthKey {}

#[derive(Clone, Debug)]
struct CounterService {
	id: usize,
}

#[derive(Debug)]
struct CounterKey;

impl InjectableKey for CounterKey {}

#[derive(Debug)]
struct FailingDependency;

#[reinhardt::async_trait::async_trait]
impl Injectable for FailingDependency {
	async fn inject(_ctx: &InjectionContext) -> DiResult<Self> {
		Err(DiError::ProviderError("planned failure".to_string()))
	}
}

fn root_context_with_registry(registry: DependencyRegistry) -> InjectionContext {
	let singleton = Arc::new(SingletonScope::new());
	InjectionContext::builder(singleton)
		.with_registry(Arc::new(registry))
		.build()
}

fn auth_context() -> InjectionContext {
	let registry = DependencyRegistry::new();
	registry.register_async::<FactoryOutput<AuthKey, AuthService>, _, _>(
		DependencyScope::Request,
		|_ctx| async {
			Ok(FactoryOutput::<AuthKey, AuthService>::new(AuthService {
				prefix: "authorized",
			}))
		},
	);
	root_context_with_registry(registry)
}

fn counter_context(scope: DependencyScope, next_id: Arc<AtomicUsize>) -> InjectionContext {
	let registry = DependencyRegistry::new();
	registry.register_async::<FactoryOutput<CounterKey, CounterService>, _, _>(
		scope,
		move |_ctx| {
			let next_id = Arc::clone(&next_id);
			async move {
				let id = next_id.fetch_add(1, Ordering::SeqCst);
				Ok(FactoryOutput::<CounterKey, CounterService>::new(
					CounterService { id },
				))
			}
		},
	);
	root_context_with_registry(registry)
}

async fn auth_handler(Di(auth): Di<Depends<AuthKey, AuthService>>, Path(id): Path<u64>) -> String {
	auth.authorize(id)
}

#[derive(serde::Deserialize)]
struct EchoQuery {
	value: String,
}

async fn query_handler(
	Di(auth): Di<Depends<AuthKey, AuthService>>,
	Query(query): Query<EchoQuery>,
) -> String {
	format!("{}:{}", auth.authorize(7), query.value)
}

async fn double_counter_handler(
	Di(first): Di<Depends<CounterKey, CounterService>>,
	Di(second): Di<Depends<CounterKey, CounterService>>,
) -> String {
	format!("{}:{}", first.id, second.id)
}

async fn missing_context_handler(Di(_auth): Di<Depends<AuthKey, AuthService>>) -> &'static str {
	"unreachable"
}

async fn failing_handler(Di(_failing): Di<FailingDependency>) -> &'static str {
	"unreachable"
}

#[rstest]
#[tokio::test]
async fn layer_resolves_depends_dependency_in_axum_handler() {
	// Arrange
	let app = Router::new()
		.route("/users/{id}", get(auth_handler))
		.layer(DiLayer::new(auth_context()));

	let request = Request::builder()
		.uri("/users/42")
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();
	let status = response.status();
	let body = axum::body::to_bytes(response.into_body(), usize::MAX)
		.await
		.unwrap();

	// Assert
	assert_eq!(status, StatusCode::OK);
	assert_eq!(&body[..], b"authorized:42");
}

#[rstest]
#[tokio::test]
async fn di_extractor_composes_with_axum_query_extractor() {
	// Arrange
	let app = Router::new()
		.route("/echo", get(query_handler))
		.layer(DiLayer::new(auth_context()));

	let request = Request::builder()
		.uri("/echo?value=ok")
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();
	let status = response.status();
	let body = axum::body::to_bytes(response.into_body(), usize::MAX)
		.await
		.unwrap();

	// Assert
	assert_eq!(status, StatusCode::OK);
	assert_eq!(&body[..], b"authorized:7:ok");
}

#[rstest]
#[tokio::test]
async fn request_scoped_dependency_is_cached_within_request_and_isolated_between_requests() {
	// Arrange
	let next_id = Arc::new(AtomicUsize::new(1));
	let app = Router::new()
		.route("/counter", get(double_counter_handler))
		.layer(DiLayer::new(counter_context(
			DependencyScope::Request,
			next_id,
		)));

	// Act
	let first_response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/counter")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	let first_body = axum::body::to_bytes(first_response.into_body(), usize::MAX)
		.await
		.unwrap();

	let second_response = app
		.oneshot(
			Request::builder()
				.uri("/counter")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	let second_body = axum::body::to_bytes(second_response.into_body(), usize::MAX)
		.await
		.unwrap();

	// Assert
	assert_eq!(&first_body[..], b"1:1");
	assert_eq!(&second_body[..], b"2:2");
}

#[rstest]
#[tokio::test]
async fn singleton_dependency_is_shared_across_requests() {
	// Arrange
	let next_id = Arc::new(AtomicUsize::new(1));
	let app = Router::new()
		.route("/counter", get(double_counter_handler))
		.layer(DiLayer::new(counter_context(
			DependencyScope::Singleton,
			next_id,
		)));

	// Act
	let first_response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/counter")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	let first_body = axum::body::to_bytes(first_response.into_body(), usize::MAX)
		.await
		.unwrap();

	let second_response = app
		.oneshot(
			Request::builder()
				.uri("/counter")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	let second_body = axum::body::to_bytes(second_response.into_body(), usize::MAX)
		.await
		.unwrap();

	// Assert
	assert_eq!(&first_body[..], b"1:1");
	assert_eq!(&second_body[..], b"1:1");
}

#[rstest]
#[tokio::test]
async fn missing_layer_returns_internal_server_error() {
	// Arrange
	let app = Router::new().route("/missing", get(missing_context_handler));

	let request = Request::builder()
		.uri("/missing")
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();
	let status = response.status();

	// Assert
	assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[rstest]
#[tokio::test]
async fn resolve_failure_returns_internal_server_error() {
	// Arrange
	let app = Router::new()
		.route("/failing", get(failing_handler))
		.layer(DiLayer::new(auth_context()));

	let request = Request::builder()
		.uri("/failing")
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();
	let status = response.status();

	// Assert
	assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[rstest]
fn rejection_response_statuses_are_internal_server_error() {
	// Arrange
	let missing = DiRejection::MissingContext;
	let resolve = DiRejection::Resolve {
		message: "provider failed".to_string(),
	};

	// Act
	let missing_response = missing.into_response();
	let resolve_response = resolve.into_response();

	// Assert
	assert_eq!(missing_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
	assert_eq!(resolve_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
