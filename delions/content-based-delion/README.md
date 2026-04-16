# content-based-delion

TF-IDF and feature similarity content-based recommendation plugin for the
[reinhardt](https://github.com/kent8192/awesome-delions) ecosystem.

## Features

- **TF-IDF Feature Extraction** -- Compute term frequency-inverse document frequency vectors
  from tokenized documents, with optional sublinear TF scaling
- **Cosine Similarity** -- Measure similarity between feature vectors using cosine distance
- **Euclidean Distance Similarity** -- Alternative similarity metric using transformed
  Euclidean distance
- **Item Profiles** -- Build item feature profiles from document corpora via TF-IDF
- **User Profiles** -- Construct user preference vectors as weighted averages of rated item
  profiles
- **Content-Based Recommendations** -- Generate top-N item recommendations based on
  user-item feature similarity

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
content-based-delion = "0.1.0"
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or
  `<http://www.apache.org/licenses/LICENSE-2.0>`)
- MIT License ([LICENSE-MIT](../../LICENSE-MIT) or
  `<http://opensource.org/licenses/MIT>`)

at your option.
