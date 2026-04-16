# popularity-delion

Time-decay popularity-based recommendation plugin for the
[reinhardt](https://github.com/kent8192/reinhardt) framework.

## Features

- **View count scoring** -- rank items by total view count within a time window
- **Rating count scoring** -- rank items by total rating count within a time window
- **Trending scoring with time decay** -- rank items using configurable decay
  functions (exponential, linear, or no decay)
- **Category-based recommendations** -- filter recommendations by item category
- **Configurable time windows** -- scope scoring to arbitrary time ranges

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
popularity-delion = "0.1.0"
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
