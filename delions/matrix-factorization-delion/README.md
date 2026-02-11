# matrix-factorization-delion

SVD and ALS matrix factorization recommendation plugin for the reinhardt plugin framework.

## Features

- Truncated SVD factorization via power iteration
- Alternating Least Squares (ALS) factorization
- Configurable latent factor model parameters (factors, regularization, iterations, tolerance)
- Top-N recommendation generation for users
- Rating prediction for user-item pairs

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
matrix-factorization-delion = "0.1.0"
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
