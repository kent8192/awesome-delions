# axum-delion

Axum web framework integration for Reinhardt via the dentdelion plugin system.

## Overview

`axum-delion` bridges Reinhardt's dependency injection, ORM, and authentication
with axum's Tower-based middleware ecosystem.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
reinhardt = { version = "0.1", default-features = false, features = ["headless-db", "db-postgres"] }
axum-delion = { version = "0.1", features = ["db-postgres"] }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

### Basic Example

```rust,ignore
use axum::{Router, routing::get};
use axum_delion::{ReinhardtLayer, Inject, ReinhardtState};

async fn handler(Inject(state): Inject<ReinhardtState>) -> &'static str {
    "Hello from Reinhardt + Axum!"
}

#[tokio::main]
async fn main() {
    let layer = ReinhardtLayer::builder()
        .build()
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(handler))
        .layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Features

| Feature | Description |
|---------|-------------|
| `database` | Enable Reinhardt ORM integration (default) |
| `auth` | Enable Reinhardt authentication |
| `db-postgres` | PostgreSQL backend |
| `db-mysql` | MySQL backend |
| `db-sqlite` | SQLite backend |
| `auth-jwt` | JWT authentication |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
