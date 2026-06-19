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
use reinhardt::di::{Depends, InjectableKey, InjectionContext, SingletonScope};
use std::sync::Arc;

struct AuthKey;
impl InjectableKey for AuthKey {}

struct AuthService;
impl AuthService {
    async fn authorize(&self, id: u64) -> String {
        format!("authorized:{id}")
    }
}

async fn handler(
    Di(auth): Di<Depends<AuthKey, AuthService>>,
    Path(id): Path<u64>,
) -> String {
    auth.authorize(id).await
}

let singleton = Arc::new(SingletonScope::new());
let root_context = InjectionContext::builder(singleton).build();

let app = Router::new()
    .route("/users/{id}", get(handler))
    .layer(DiLayer::new(root_context));
```

`Path`, `Query`, `Json`, headers, and body extraction remain Axum
responsibilities. `Di<T>` is the intended wrapper for service dependencies
where `T` implements `reinhardt::di::Injectable`, including `Depends<K, T>`.
