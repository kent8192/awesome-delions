# CLAUDE.md

## Purpose

This file contains project-specific instructions for the awesome-delions repository. These rules ensure code quality, maintainability, and consistent practices across all delion crates.

---

## Project Overview

See README.md for project details.

**Repository URL**: https://github.com/kent8192/awesome-delions

---

## Tech Stack

- **Language**: Rust 2024 Edition
- **Module System**: MUST use 2024 edition (NO `mod.rs`)
- **Plugin Framework**: reinhardt-dentdelion (via `reinhardt` facade)
- **Testing**: Rust's built-in framework + cargo-nextest
- **Build**: Cargo virtual workspace with independent delion crates
- **Release**: release-plz for automated versioning and publishing

---

## Critical Rules

### Repository Structure

- All delions MUST be placed under `delions/` directory
- Each delion is an independent crate with its own `Cargo.toml`
- NO inter-delion dependencies (all external via crates.io)
- Delion naming convention: `xxx-delion` (e.g., `auth-delion`)

### Dependency Policy

- Each delion depends on `reinhardt` facade crate with feature flags
- Example: `reinhardt = { version = "0.1.0-alpha", default-features = false, features = ["dentdelion"] }`
- Additional reinhardt features can be added as needed (e.g., `"auth"`, `"database"`)
- NEVER depend directly on `reinhardt-dentdelion` crate

### Module System

**MUST use `module.rs` + `module/` directory structure (Rust 2024 Edition)**
**NEVER use `mod.rs` files** (deprecated)

**Basic Patterns:**

Pattern 1 — Small Module (single file):
```
src/
├── lib.rs          // mod utils;
└── utils.rs        // pub fn helper() {}
```

Pattern 2 — Medium Module (with submodules):
```
src/
├── lib.rs          // mod database;
├── database.rs     // pub mod pool; pub mod connection;
└── database/
    ├── pool.rs
    └── connection.rs
```

Pattern 3 — Large Module (hierarchical, max 4 levels deep):
```
src/
├── lib.rs             // mod api;
├── api.rs             // pub mod handlers; pub mod middleware;
└── api/
    ├── handlers.rs    // pub mod user; pub mod auth;
    ├── handlers/
    │   ├── user.rs
    │   └── auth.rs
    └── middleware.rs
```

**Visibility Control with `pub use`:**
```rust
// database.rs (entry point)
mod pool;           // Private submodule
mod connection;     // Private submodule

// Public API - explicitly re-export
pub use pool::{Pool, PoolConfig};
pub use connection::Connection;
```

**Rules:**
- Use `pub use` in module entry points to control exposed API
- **NEVER** use glob imports (`pub use module::*`) — use explicit imports (exception: `use super::*` in test modules)
- Avoid nesting beyond 4 levels

### Code Style

**Key Requirements:**
- **ALL code comments MUST be written in English** (no exceptions)
- MINIMIZE `.to_string()` calls - prefer borrowing
- DELETE obsolete code immediately
- NO deletion record comments in code
- Mark ALL placeholders with `todo!()` or `// TODO:` comment
- Document ALL `#[allow(...)]` attributes with explanatory comments

**Unimplemented Features Notation:**
- `todo!()` - Features that WILL be implemented
- `unimplemented!()` - Features that WILL NOT be implemented (intentionally omitted)
- `// TODO:` - Planning notes
- **DELETE** `todo!()` and `// TODO:` when implemented
- **KEEP** `unimplemented!()` for permanently excluded features
- **NEVER** use alternative notations (`FIXME:`, `Implementation Note:`, etc.)

### Testing

**Core Principles:**
- NO skeleton tests (all tests MUST have meaningful assertions)
- Use strict assertions (`assert_eq!`) instead of loose matching (`contains`)
- Follow Arrange-Act-Assert (AAA) pattern for test structure
- Use `rstest` for ALL test cases (no plain `#[test]`)
- ALL test artifacts MUST be cleaned up

**Arrange-Act-Assert (AAA) Pattern:**

| Phase | Label | Purpose |
|-------|-------|---------|
| Arrange | `// Arrange` | Set up test preconditions and inputs |
| Act | `// Act` | Execute the behavior under test |
| Assert | `// Assert` | Verify the expected outcomes |

- Use ONLY standard labels: `// Arrange`, `// Act`, `// Assert`
- **NEVER** use non-standard labels: `// Setup`, `// Execute`, `// Verify`
- AAA comments MAY be omitted when the test body is **5 lines or fewer**
- rstest fixtures serve as the Arrange phase — use `// Arrange: provided by <fixture>` or omit entirely

**Assertion Strictness:**
- **Prefer** `assert_eq!`, `assert_ne!`, `assert!(matches!(...))` for exact verification
- **Avoid** `assert!(string.contains(...))`, `assert!(result.is_ok())` without checking value
- **Exception**: Loose assertions ONLY when strict is impossible (random values, system-dependent values, non-deterministic operations). Add `// NOTE:` comment explaining why

**rstest Basic Pattern:**
```rust
use rstest::*;

#[fixture]
fn test_data() -> Vec<String> {
    vec!["item1".to_string(), "item2".to_string()]
}

#[rstest]
fn test_with_fixture(test_data: Vec<String>) {
    // Act
    let len = test_data.len();

    // Assert
    assert_eq!(len, 2);
}
```

**Global State Tests:**
- Tests modifying global state MUST use `#[serial(group_name)]` from `serial_test` crate
- Name groups descriptively: `#[serial(i18n)]`, `#[serial(registry)]`
- **ALWAYS** call cleanup functions in test teardown

### File Management

**Critical Rules:**
- **NEVER** save temp files to project directory (use `/tmp`)
- **IMMEDIATELY** delete `/tmp` files when no longer needed
- **IMMEDIATELY** delete backup files (`.bak`, `.backup`, `.old`, `~` suffix)

### Documentation

**Update Requirements:**
- **ALWAYS** update docs when code changes (same workflow)
- **NEVER** document user requests or AI assistant interactions in project documentation
- Focus on the "why" (technical rationale) not the "who asked"

**Documentation Locations:**

| Location | Contents | When to Update |
|----------|----------|----------------|
| `README.md` | Project overview, installation, features (implemented only) | Major features, structure changes |
| `delions/<name>/README.md` | Delion-specific overview, features, usage | Adding delion features, API changes |
| `delions/<name>/src/lib.rs` | Module docs (`//!`), planned features, code examples | Planned features, API changes |

**Planned Features Location:**
- Planned features MUST go in `lib.rs` header (`//! ## Planned Features`)
- **NEVER** put planned features in README.md — README shows implemented features only

**Documentation Consistency Checklist:**
- [ ] Terminology consistent across all docs
- [ ] Code examples use the same style and actually work
- [ ] Version numbers match
- [ ] Links are valid
- [ ] API signatures match implementation

**Documentation Quality:**
- All code examples MUST be tested (use doc tests)
- All links MUST be verified
- Code examples MUST be kept current with API changes
- Run `cargo test --doc` to verify doc tests

**Rustdoc Formatting Rules:**

| Pattern | Incorrect | Correct |
|---------|-----------|---------|
| Generic types | `Option<T>` | `` `Option<T>` `` |
| Macro attributes | `#[derive]` | `` `#[derive]` `` |
| URLs | `https://...` | `<https://...>` or `` `https://...` `` |
| Code blocks | ` ``` ` | ` ```rust ` (specify language) |
| Array/bracket access | `arr[0]` | `` `arr[0]` `` |
| Feature-gated items | `` [`TypeName`] `` (intra-doc link) | `` `TypeName` `` (backticks) |

- Generic types like `<T>` are interpreted as HTML tags — backticks required
- Macro attributes like `#[inject]` are interpreted as markdown links — backticks required
- Feature-gated items may not exist when docs are built without that feature — use backticks, not intra-doc links

### Git Workflow

**Commit Policy:**
- **NEVER** commit without explicit user instruction
- **NEVER** push without explicit user instruction
- **EXCEPTION**: Plan Mode approval is considered explicit commit authorization
- Split commits by specific intent (NOT feature-level goals)
- Each commit MUST be small enough to explain in one line
- **NEVER** execute batch commits without user confirmation

**GitHub Integration:**
- **MUST** use GitHub CLI (`gh`) for all GitHub operations

### Commit Message Format

**Subject Line:**
```
<type>[optional scope][optional !]: <description>
```

**Description Rules:**
- **MUST** start with lowercase letter
- **MUST NOT** end with a period
- **MUST** be specific, not vague
- Write descriptions as standalone CHANGELOG entries (they appear directly in release notes)

**Examples:**
- ✅ `feat(auth): add password validation with bcrypt`
- ✅ `fix(orm): resolve connection pool exhaustion under high concurrency`
- ✅ `feat(api)!: change response format to JSON:API specification`
- ❌ `feat(auth): Improve authentication` (uppercase, vague)
- ❌ `fix: fix issue.` (vague, ends with period)

**Commit Types (unified with release-plz.toml commit_parsers):**

| Type | Description | Version Bump | CHANGELOG Section |
|------|-------------|--------------|-------------------|
| `feat` | A new feature | MINOR | Added |
| `fix` | A bug fix | PATCH | Fixed |
| `perf` | Performance improvements | PATCH | Performance |
| `refactor` | Code change (no bug fix or feature) | PATCH | Changed |
| `docs` | Documentation only changes | PATCH | Documentation |
| `revert` | Reverts a previous commit | PATCH | Reverted |
| `deprecated` | Marks features/APIs as deprecated | PATCH | Deprecated |
| `security` | Security vulnerability fixes | PATCH | Security |
| `chore` | Maintenance tasks (no production code) | PATCH | Maintenance |
| `ci` | CI configuration changes | PATCH | Maintenance |
| `build` | Build system or dependency changes | PATCH | Maintenance |
| `test` | Adding or modifying tests | PATCH | Testing |
| `style` | Code style changes (formatting) | PATCH | Styling |

**Choosing Between Similar Types:**
- **`security` vs `fix`**: Use `security` for fixes addressing security vulnerabilities (CVEs, injection flaws). Use `fix` for general bug fixes
- **`docs` vs `chore(docs)`**: Use `docs` for user-facing documentation (README, API docs). Use `chore(docs)` for internal docs (code comments)
- **`refactor` vs `feat`/`fix`**: Use `refactor` when behavior does not change. If refactoring changes behavior, use `feat!:` or `fix!:`
- **`deprecated` vs `feat`**: Use `deprecated` when primary purpose is marking features as deprecated. Use `feat` if deprecation is part of a larger replacement

**Breaking Changes (SemVer MAJOR):**

Three ways to indicate:
1. **`!` notation (preferred):** `feat!: remove deprecated endpoints`
2. **Footer:** `BREAKING CHANGE: legacy auth no longer supported`
3. **Both:** `refactor(db)!: change pool impl` + `BREAKING CHANGE:` footer

**Auto-Skipped Commits (excluded from CHANGELOG):**

| Pattern | Reason |
|---------|--------|
| `chore: release` | release-plz automation |
| `Merge ...` | Git merge commits |
| `Revert "..."` | GitHub-generated reverts (manual `revert:` type IS included) |

Note: Even skipped commits with breaking changes are included due to `protect_breaking_commits = true`.

### Release & Publishing Policy

**Automated Releases with release-plz:**

Version bumps are determined by commit types (see Commit Types table above for full mapping).

**Tagging Strategy (Per-Crate Tagging):**
- Format: `[delion-name]@v[version]` (e.g., `auth-delion@v0.1.0`)
- Tags are created automatically by release-plz upon Release PR merge
- **NEVER** create release tags manually

**Release Workflow:**
1. Write commits following Conventional Commits format
2. Push to main branch
3. release-plz creates Release PR with version bumps and CHANGELOG updates
4. Review and merge Release PR
5. release-plz publishes to crates.io and creates Git tags

**Auto-Skipped Commits:** `chore: release`, `Merge ...`, `Revert "..."` are excluded from CHANGELOG (see Commit Message Format section)

**Critical Rules:**
- **MUST** use conventional commit format for proper version detection
- **MUST** review Release PRs before merging
- **NEVER** manually bump versions in feature branches
- **NEVER** change `pr_branch_prefix` from `"release-plz-"`

### Issue Management

**Language:** ALL issue titles, descriptions, and comments MUST be written in English (no exceptions)

**Before Creating an Issue:**
1. Search existing issues: `gh issue list --search "<keywords>"`
2. Check closed issues too: `gh issue list --state closed --search "<keywords>"`
3. Review related issues for context

**Issue Template Usage:**
- **MUST** use appropriate issue template for all issues
- Templates apply type labels automatically

**Issue Title Format:**
- Specific, concise, max 72 characters
- Start with uppercase letter
- Use type prefix (recommended): `Bug: ...`, `Feature: ...`, `Docs: ...`

**Security Vulnerabilities:**
- **NEVER** create public issues for security vulnerabilities
- **MUST** report via GitHub Security Advisories (private)

---

## Common Commands

**Check & Build:**
```bash
cargo check --workspace
cargo build --workspace
```

**Testing:**
```bash
cargo nextest run --workspace
```

**Code Quality:**
```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

**Creating a New Delion:**
```bash
cargo generate --path template/ --name my-delion --destination delions/
```

---

## Review Process

Before submitting code:

1. **Run all commands:**
   - `cargo check --workspace`
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo nextest run --workspace`

2. **Iterate until all issues resolved**

---

## Additional Instructions

@CLAUDE.local.md - Project-specific local preferences

---

## Quick Reference

### MUST DO
- Write ALL code comments in English (no exceptions)
- Use `module.rs` + `module/` directory (NO `mod.rs`)
- Place all delions under `delions/` directory
- Depend on `reinhardt` facade with feature flags (NOT `reinhardt-dentdelion` directly)
- Follow `xxx-delion` naming convention
- Use conventional commit format: `<type>[scope]: <description>`
- Start commit description with lowercase letter
- Use `!` notation for breaking changes (e.g., `feat!:`, `feat(scope)!:`)
- Write commit descriptions as standalone CHANGELOG entries (meaningful without context)
- Use `security` type for security vulnerability fixes (dedicated CHANGELOG section)
- Use `deprecated` type for marking features/APIs as deprecated (dedicated CHANGELOG section)
- Use `rstest` for ALL test cases (no plain `#[test]`)
- Follow Arrange-Act-Assert (AAA) pattern with `// Arrange`, `// Act`, `// Assert` labels
- Use `#[serial(group_name)]` for global state tests
- Wait for explicit user instruction before commits
- Wrap generic types in backticks in doc comments: `` `Result<T>` ``
- Wrap macro attributes in backticks: `` `#[inject]` ``
- Wrap URLs in angle brackets or backticks: `<https://...>`
- Specify language for code blocks: ` ```rust `
- Wrap bracket patterns in backticks: `` `array[0]` ``
- Use backticks (not intra-doc links) for feature-gated types: `` `FeatureType` ``
- Planned features go in `lib.rs` header, NOT in README.md
- Search existing issues before creating new ones (`gh issue list --search`)
- Report security vulnerabilities privately via GitHub Security Advisories
- Document ALL `#[allow(...)]` attributes with explanatory comments

### NEVER DO
- Use `mod.rs` files
- Depend directly on `reinhardt-dentdelion`
- Create inter-delion dependencies
- Commit without user instruction (except Plan Mode approval)
- Manually bump versions
- Create release tags manually
- Change `pr_branch_prefix` from `"release-plz-"`
- Save files to project directory (use `/tmp`)
- Create skeleton tests
- Use non-English code comments
- Use glob imports (`use module::*`) except in test modules
- Use `#[allow(...)]` without explanatory comments
- Write vague commit descriptions (e.g., "fix issue", "update code")
- Start commit description with uppercase letter
- End commit description with a period
- Omit `!` or `BREAKING CHANGE:` for API-breaking changes
- Create public issues for security vulnerabilities
- Create duplicate issues without searching first
- Use non-standard AAA labels (`// Setup`, `// Execute`, `// Verify`)
- Write generic types without backticks in doc comments (causes HTML tag warnings)
- Write macro attributes without backticks in doc comments (causes unresolved link warnings)
- Put planned features in README.md (use `lib.rs` instead)
- Use plain `#[test]` instead of `#[rstest]`
