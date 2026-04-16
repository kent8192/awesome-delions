# collaborative-filtering-delion

User-based and item-based collaborative filtering recommendation plugin for the reinhardt ecosystem.

## Features

- User-based collaborative filtering with k-NN similarity
- Item-based collaborative filtering with k-NN similarity
- Multiple similarity metrics: Cosine, Pearson correlation, Jaccard
- Sparse rating matrix for efficient storage and lookup
- Configurable k-neighbors and minimum similarity threshold

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
collaborative-filtering-delion = "0.1.0"
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
