# awesome-delions

Official collection of Reinhardt Dentdelion plugins. This repository serves as a CI/CD-managed monorepo for independent delion crates.

## Overview

Each delion is an independent Rust crate located under `delions/`. The repository itself is a virtual workspace that provides:

- Shared code quality configuration (rustfmt, clippy, lints)
- Automated CI/CD via GitHub Actions
- Automated releases via [release-plz](https://release-plz.ieni.dev/)
- `cargo-generate` template for creating new delions

## Creating a New Delion

### Using cargo-generate

```bash
cargo generate --path template/ --name my-delion --destination delions/
```

### Manual Creation

1. Create a directory under `delions/`:
   ```bash
   mkdir -p delions/my-delion/src
   ```

2. Add `Cargo.toml`:
   ```toml
   [package]
   name = "my-delion"
   version = "0.1.0"
   edition.workspace = true
   license.workspace = true
   repository.workspace = true
   authors.workspace = true
   description = "My awesome delion plugin"

   [dependencies]
   reinhardt = { version = "0.1.0-alpha", default-features = false, features = ["dentdelion"] }

   [lints]
   workspace = true
   ```

3. Add `dentdelion.toml` manifest and implement your plugin in `src/lib.rs`.

## Naming Convention

All delion crates follow the `xxx-delion` naming pattern:
- `auth-delion` - Authentication plugin
- `rate-limit-delion` - Rate limiting plugin
- `cors-delion` - CORS configuration plugin

## Release Workflow

This project uses release-plz for automated releases:

1. Commit changes following [Conventional Commits](https://www.conventionalcommits.org/) format
2. Push to `main` branch
3. release-plz creates a Release PR with version bumps and CHANGELOG updates
4. Review and merge the Release PR
5. release-plz publishes changed crates to crates.io and creates Git tags

### Tag Format

Tags follow per-crate format: `<crate-name>@v<version>` (e.g., `auth-delion@v0.1.0`)

## CI/CD

### Pull Request Checks

On every PR, the CI pipeline:
1. Detects which delions have changed
2. Runs quality checks only on affected crates:
   - `cargo check`
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `cargo nextest run`

### Release Automation

On merge to `main`:
1. release-plz detects version-worthy changes
2. Creates/updates a Release PR
3. On Release PR merge, publishes to crates.io

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
