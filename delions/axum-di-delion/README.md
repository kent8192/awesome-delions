# axum-di-delion

Axum integration delion for Reinhardt dependency injection.

This crate defines the public API for pairing standalone Axum handlers with
`reinhardt::di` dependencies while keeping Axum's own extractors for request
data.

## Usage

The intended handler shape is:

```rust
use axum::{Router, extract::Path, routing::get};
use axum_di_delion::{Di, DiLayer};
use reinhardt::di::{
    DependencyRegistry, DependencyScope, Depends, FactoryOutput, InjectableKey,
    InjectionContext, SingletonScope,
};
use std::sync::Arc;

struct AuthKey;
impl InjectableKey for AuthKey {}

#[derive(Clone)]
struct AuthService {
    prefix: &'static str,
}

impl AuthService {
    async fn authorize(&self, id: u64) -> String {
        format!("{}:{id}", self.prefix)
    }
}

let registry = DependencyRegistry::new();
registry.register_async::<FactoryOutput<AuthKey, AuthService>, _, _>(
    DependencyScope::Request,
    |_ctx| async {
        Ok(FactoryOutput::<AuthKey, AuthService>::new(AuthService {
            prefix: "authorized",
        }))
    },
);

async fn handler(
    Di(auth): Di<Depends<AuthKey, AuthService>>,
    Path(id): Path<u64>,
) -> String {
    auth.authorize(id).await
}

let singleton = Arc::new(SingletonScope::new());
let root_context = InjectionContext::builder(singleton)
    .with_registry(Arc::new(registry))
    .build();

let app: Router<()> = Router::new()
    .route("/users/{id}", get(handler))
    .layer(DiLayer::new(root_context));
```

`Path`, `Query`, `Json`, headers, and body extraction remain Axum
responsibilities. `Di<T>` is the intended wrapper for service dependencies
where `T` implements `reinhardt::di::Injectable`, including `Depends<K, T>`.

`DiLayer` forks the root `InjectionContext` for every request. Request-scoped
dependencies are cached within one request, while singleton dependencies remain
shared through the root context's `SingletonScope`.
