# Testing Standards

## Purpose

This document defines comprehensive testing standards for the awesome-delions project, ensuring high-quality, maintainable test coverage across all delion crates.

---

## Testing Philosophy

### TP-1 (MUST): Test Completeness

**NO skeleton implementations** - All tests MUST contain meaningful assertions.

**Definition of Skeleton Test:**
- A test that always passes (e.g., empty test, `assert!(true)`)
- Tests without any assertions
- Tests that don't actually verify behavior

**Requirements:**
- Tests MUST be capable of failing when the code is incorrect
- Documentation tests must be performed for all features you implement
- Do not implement test cases that are identical to documentation tests as unit tests or integration tests

**Examples:**

❌ **BAD - Skeleton Tests:**
```rust
#[test]
fn test_handler_creation() {
    // Empty test - always passes
}

#[test]
fn test_config_validation() {
    let result = validate_config(&config);
    // No assertion - useless test
}
```

✅ **GOOD - Meaningful Tests:**
```rust
#[rstest]
fn test_handler_creation() {
    let handler = AuthHandler::new(config.clone());
    assert_eq!(handler.name(), "auth");
}

#[rstest]
fn test_config_validation() {
    assert!(validate_config(&valid_config).is_ok());
    assert!(validate_config(&invalid_config).is_err());
}
```

### TP-2 (MUST): Reinhardt Component Usage

**EVERY** test case MUST use at least one component from the `reinhardt` facade crate (via its feature flags, including `reinhardt-dentdelion` re-exports) or from the delion crate being tested.

**Qualifying Components:**
- Functions, variables, methods exposed through the `reinhardt` facade
- Structs, traits, enums from the delion crate under test
- Commands, macros, attribute re-exports
- Any public item of the delion crate itself

**Why?** This ensures tests actually verify delion functionality rather than testing third-party libraries or standard library behavior alone.

---

## Test Organization

### TO-1 (MUST): Unit vs Integration Tests

Clear separation based on the nature of what is being tested:

#### Unit Tests
**Definition:** Tests that verify the behavior of a **single component**

**Component:** A single function, method, struct, trait, enum, or closely related group of items that serve a unified purpose.

**Location:** Within the delion crate being tested

**Characteristics:**
- Tests a component in isolation
- Verifies the component's behavior and edge cases
- Does not test interactions between multiple components

**Structure:**
```
delions/auth-delion/
├── src/
│   ├── lib.rs
│   ├── handler.rs
│   └── config.rs
└── tests/
    └── handler_tests.rs

// Unit tests in the same file
// src/handler.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    fn test_handler_default_timeout() {
        let handler = AuthHandler::default();
        assert_eq!(handler.timeout(), Duration::from_secs(30));
    }
}
```

#### Integration Tests
**Definition:** Tests that verify the **integration points** (interfaces) between **two or more components**

**Location:** Each delion crate is independent (no inter-delion dependencies — see @instructions/DELION_PATTERNS.md DP-4), so integration tests MUST be placed within the delion crate's own `tests/` directory.

**Structure:**
```
delions/auth-delion/
└── tests/
    └── integration_tests.rs
```

### TO-2 (SHOULD): Test File Organization

Organize test files to mirror the source structure:

```
delions/auth-delion/
├── src/
│   ├── lib.rs
│   ├── handler.rs
│   └── config.rs
└── tests/
    ├── handler_tests.rs
    └── config_tests.rs
```

---

## Test Implementation

### TI-1 (SHOULD): TODO Comments

If tests cannot be fully implemented, leave a `// TODO:` comment explaining why.

**DELETE** the TODO comment when the test is implemented.

**Example:**
```rust
#[rstest]
fn test_oauth_flow() {
    // TODO: Implement after adding OAuth backend support to auth-delion
    todo!("Waiting for OAuth integration")
}
```

### TI-2 (MUST): Unimplemented Feature Notation

For unimplemented features, use one of the following:

#### Option 1: `todo!()` macro
Use for features that **WILL** be implemented later

```rust
fn refresh_token(token: &str) -> Result<Token> {
    todo!("Add token refresh - planned for next sprint")
}
```

#### Option 2: `unimplemented!()` macro
Use for features that **WILL NOT** be implemented (intentionally omitted)

```rust
fn legacy_v1_endpoint() -> String {
    unimplemented!("v1 API is intentionally not supported")
}
```

#### Option 3: `// TODO:` comment
Use for planning without runtime panics

```rust
// TODO: Add support for custom token claims
fn build_token(user: &User) -> Token {
    // Temporary implementation
    Token::default()
}
```

**Macro Selection Guidelines:**
- `todo!()` → Features that WILL be implemented
- `unimplemented!()` → Features that WILL NOT be implemented
- `// TODO:` → Planning notes

**DELETE `todo!()` and `// TODO:` when implemented**
**KEEP `unimplemented!()` for permanently excluded features**

### TI-3 (MUST): Test Cleanup

**ALL** files, directories, or environmental changes created during tests **MUST** be deleted upon test completion.

**Techniques:**
- Test fixtures with `Drop` implementations
- `tempfile` crate for temporary files
- Explicit cleanup in test teardown

**Example:**
```rust
#[rstest]
fn test_config_file_loading() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("delion.toml");

    // Test code that creates files
    std::fs::write(&config_path, "key = \"value\"").unwrap();

    // Cleanup happens automatically when temp_dir drops
}
```

### TI-4 (MUST): Global State Management

Tests that modify global state MUST be serialized using the `serial_test` crate.

Use named serial groups: `#[serial(group_name)]` to serialize only related tests.

**Setup:**
```toml
# Cargo.toml
[dev-dependencies]
serial_test = { workspace = true }
```

**Example:**
```rust
use serial_test::serial;

#[rstest]
#[serial(env_config)]
fn test_env_override() {
    std::env::set_var("DELION_CONFIG", "/tmp/test-delion.toml");
    assert_eq!(load_config_path(), "/tmp/test-delion.toml");
    std::env::remove_var("DELION_CONFIG");  // ✅ Cleanup
}
```

### TI-5 (MUST): Assertion Strictness

**Use strict assertions with exact value comparisons instead of loose pattern matching.**

**Preferred Methods:**
- `assert_eq!(actual, expected)` - For exact value equality
- `assert_ne!(actual, unexpected)` - For exact value inequality
- `assert!(matches!(value, Pattern))` - For pattern matching with specific variants

**Avoid Loose Assertions:**
- ❌ `assert!(string.contains("substring"))` - Too permissive, may match unintended content
- ❌ `assert!(result.is_ok())` without checking the contained value
- ❌ `assert!(value > 0)` when you know the exact expected value

**Exception:**
Loose assertions are acceptable ONLY when strict assertions are impossible or impractical:
- Random values (e.g., UUIDs, generated tokens)
- System-dependent values (e.g., timestamps, temp paths)
- Non-deterministic operations

**Justification Requirement:**
When using loose assertions, add a comment explaining why strict assertions are not possible.

### TI-6 (SHOULD): Arrange-Act-Assert (AAA) Pattern

All tests SHOULD follow the **Arrange-Act-Assert (AAA)** pattern for clear, consistent structure.

**AAA Phases:**

| Phase | Purpose | BDD Equivalent |
|-------|---------|----------------|
| **Arrange** | Set up test preconditions and inputs | Given |
| **Act** | Execute the behavior under test | When |
| **Assert** | Verify the expected outcomes | Then |

**Comment Labels:**

Use ONLY these standard labels:
- `// Arrange` - Setup phase
- `// Act` - Execution phase
- `// Assert` - Verification phase

❌ **Non-standard labels are prohibited:** `// Setup`, `// Execute`, `// Verify`, `// Given`, `// When`, `// Then`

**Examples:**

```rust
#[rstest]
fn test_auth_config_defaults() {
    // Arrange
    let raw = AuthConfig {
        secret: "s3cret".to_string(),
        ttl_seconds: None,
    };

    // Act
    let resolved = raw.with_defaults();

    // Assert
    assert_eq!(resolved.ttl_seconds, Some(3600));
}
```

```rust
#[rstest]
fn test_handler_dispatch(handler_fixture: AuthHandler) {
    // Arrange: provided by handler_fixture

    // Act
    let outcome = handler_fixture.dispatch(request());

    // Assert
    assert!(matches!(outcome, Dispatch::Accepted { .. }));
}
```

**Comment Omission:**

AAA comments MAY be omitted when the test body is **5 lines or fewer** and the phases are self-evident.

---

## Infrastructure Testing

### IT-1 (SHOULD): TestContainers for Infrastructure

Delions generally do not own infrastructure, but when a delion integrates with a real service (e.g., a database-backed cache, a Redis-backed session store), use **TestContainers** instead of mocks:
- Databases (PostgreSQL, MySQL)
- Message queues (Redis, RabbitMQ)
- Cache systems

**Benefits:**
- Tests use real infrastructure, not mocks
- More confidence in production behavior

**Example:**
```rust
use testcontainers::{ContainerAsync, GenericImage};

#[rstest]
#[tokio::test]
async fn test_session_store_roundtrip(#[future] redis_container: RedisFixture) {
    // Arrange
    let (_container, client) = redis_container.await;

    // Act
    client.set("k", "v").await.unwrap();
    let value = client.get("k").await.unwrap();

    // Assert
    assert_eq!(value, Some("v".to_string()));
}
```

Delions that have no external infrastructure dependency MAY skip TestContainers entirely — the dependency should not be added "just in case".

### IT-2 (MUST): Prevent Flaky Tests with TestContainers

When using TestContainers for parallel test execution, limit concurrency:

```toml
# .cargo/nextest.toml
[profile.default]
max-tests-per-run = 8
slow-timeout = "60s"
timeout = "120s"
retries = { backoff = "exponential", max-retries = 2, seed = 12345 }
```

---

## rstest Best Practices

### TF-0 (MUST): rstest for All Test Cases

**ALL** test cases in this project MUST use **rstest** as the test framework.

**Requirements:**
- Import `rstest::*` in all test modules
- Use `#[rstest]` attribute instead of `#[test]`
- Use `#[rstest]` with `#[tokio::test]` for async tests
- Leverage fixtures for setup/teardown

❌ **BAD - Using standard #[test]:**
```rust
#[test]
fn test_basic_operation() {
    let handler = AuthHandler::new();
    assert!(handler.is_ready());
}
```

✅ **GOOD - Using rstest:**
```rust
use rstest::*;

#[rstest]
fn test_basic_operation(handler_fixture: AuthHandler) {
    assert!(handler_fixture.is_ready());
}

#[rstest]
#[tokio::test]
async fn test_async_operation(#[future] redis_container: RedisFixture) {
    let (_container, client) = redis_container.await;
    assert!(client.ping().await.is_ok());
}
```

### TF-1 (SHOULD): rstest Fixture Pattern

Use **rstest** fixtures for reusable test setup and dependency injection.

Fixtures serve as the **Arrange** phase in the AAA pattern.

#### Basic Fixture

```rust
use rstest::*;

#[fixture]
fn auth_config() -> AuthConfig {
    AuthConfig {
        secret: "test-secret".to_string(),
        ttl_seconds: Some(3600),
    }
}

#[rstest]
fn test_with_fixture(auth_config: AuthConfig) {
    assert_eq!(auth_config.ttl_seconds, Some(3600));
}
```

#### Async Fixture

```rust
#[fixture]
async fn redis_fixture() -> (ContainerAsync<GenericImage>, RedisClient) {
    // Setup Redis container and client
    // ...
}

#[rstest]
#[tokio::test]
async fn test_with_async_fixture(
    #[future] redis_fixture: (ContainerAsync<GenericImage>, RedisClient)
) {
    let (_container, client) = redis_fixture.await;
    // Test code
}
```

**IMPORTANT**: Always include `#[future]` for async fixtures, and `.await` them in the test body.

---

## Quick Reference

### ✅ MUST DO
- Use `rstest` for ALL test cases (no plain `#[test]`)
- Every test MUST have at least one meaningful assertion
- Every test MUST exercise at least one delion or `reinhardt` facade component
- Follow Arrange-Act-Assert (AAA) pattern with `// Arrange`, `// Act`, `// Assert` comments
- Use strict assertions (`assert_eq!`) instead of loose matching
- Use `#[serial(group_name)]` for global state tests
- Clean up ALL test artifacts

### ❌ NEVER DO
- Use plain `#[test]` instead of `#[rstest]`
- Create skeleton tests (tests without assertions)
- Use loose assertions without justification comment
- Use non-standard phase labels (`// Setup`, `// Execute`, `// Verify`)
- Leave test artifacts uncleaned
- Add TestContainers dependencies to a delion that has no infrastructure integration

---

## Related Documentation

- **Main Quick Reference**: @CLAUDE.md (see Quick Reference section)
- **Anti-Patterns**: @instructions/ANTI_PATTERNS.md
- **Module System**: @instructions/MODULE_SYSTEM.md
- **Delion Patterns**: @instructions/DELION_PATTERNS.md
