# CLAUDE.md

## Purpose

This file contains project-specific instructions for the awesome-delions repository. These rules ensure code quality, maintainability, and consistent practices across all delion crates.

For detailed standards, see documentation in the `instructions/` directory.

---

## Project Overview

See `README.md` for project details.

**Repository URL**: https://github.com/kent8192/awesome-delions

awesome-delions is a workspace of independent **delion** plugin crates for the reinhardt web framework. Each delion lives under `delions/<name>-delion/`, depends on the `reinhardt` facade with explicit feature flags, and is released independently via release-plz. See @instructions/DELION_PATTERNS.md for the full delion contract.

---

## Tech Stack

- **Language**: Rust 2024 Edition (`resolver = "3"`)
- **Module System**: MUST use 2024 edition (NO `mod.rs`)
- **Plugin Framework**: `reinhardt-dentdelion`, consumed via the `reinhardt` facade
- **Testing**: `rstest` + cargo-nextest; TestContainers where a delion integrates real infrastructure
- **Build**: Cargo virtual workspace (`[workspace] members = ["delions/*"]`)
- **Release**: release-plz for automated per-crate versioning and publishing
- **Static checks**: Clippy (`all = deny`, `pedantic = warn`), Semgrep (`.semgrep/todo-comments.yml`), `unsafe_code = "forbid"` at the workspace level

---

## Critical Rules

### Module System

**MUST use `module.rs` + `module/` directory structure (Rust 2024 Edition)**
**NEVER use `mod.rs` files** (deprecated)

See @instructions/MODULE_SYSTEM.md for comprehensive module system standards including:
- Basic module patterns (small, medium, large)
- Visibility control with `pub use`
- Anti-patterns to avoid
- Migration guide from old style

### Delion Contract

Every delion in this workspace MUST follow the delion contract: directory layout, `reinhardt` facade dependency, `dentdelion.toml` manifest, template-based creation, and per-crate release tagging.

See @instructions/DELION_PATTERNS.md for the full contract (DP-1 ~ DP-8):
- Delion directory layout (DP-1)
- `reinhardt` facade dependency policy (DP-2)
- Feature flag hygiene (DP-3)
- No inter-delion dependencies (DP-4)
- Template-based creation (DP-5)
- `dentdelion.toml` plugin manifest (DP-6)
- Public API boundary (DP-7)
- Release integration (DP-8)

### Code Style

**Key Requirements:**
- **ALL code comments MUST be written in English** (no exceptions)
- MINIMIZE `.to_string()` calls - prefer borrowing
- DELETE obsolete code immediately
- NO deletion record comments in code
- NO relative paths beyond `../` (use absolute paths from crate root)
- Mark ALL placeholders with `todo!()` or `// TODO:` comment
- Document ALL `#[allow(...)]` attributes with explanatory comments (see @instructions/ANTI_PATTERNS.md)

**Unimplemented Features Notation:**
- `todo!()` - Features that WILL be implemented
- `unimplemented!()` - Features that WILL NOT be implemented (intentionally omitted)
- `// TODO:` - Planning notes
- **DELETE** `todo!()` and `// TODO:` when implemented
- **KEEP** `unimplemented!()` for permanently excluded features
- **NEVER** use alternative notations (`FIXME:`, `Implementation Note:`, etc.)

**CI Enforcement (TODO Check):**
- New `todo!()`, `// TODO`, and `// FIXME` added in PRs are detected and blocked by the TODO Check CI workflow and `.semgrep/todo-comments.yml` rules
- `unimplemented!()` is exempt (reserved for permanently excluded features)
- Existing TODOs are not flagged due to diff-aware scanning
- Clippy enforces `clippy::todo`, `clippy::unimplemented`, and `clippy::dbg_macro` via workspace lints
- The `template/` directory is excluded from Semgrep scans

**Workaround Comments:**
- When introducing workaround code for upstream issues, MUST include the ideal implementation (see @instructions/UPSTREAM_ISSUE_REPORTING.md WP-3)
- The ideal implementation comment enables future developers to remove the workaround without re-investigating the intended design

**Comment template:**
```rust
// Workaround for kent8192/reinhardt-web#42 (tracked in awesome-delions#15)
// Remove this workaround when the upstream issue is resolved.
//
// Ideal implementation (without workaround):
//   let handler = reinhardt::dentdelion::Handler::from_context(ctx);
```

See @instructions/ANTI_PATTERNS.md for comprehensive anti-patterns guide.

### Testing

**Core Principles:**
- NO skeleton tests (all tests MUST have meaningful assertions)
- EVERY test MUST exercise at least one delion or `reinhardt` facade component
- Unit tests: Test single component behavior, place within the delion crate
- Integration tests: Place within the delion crate's own `tests/` directory (inter-delion dependencies are prohibited — see @instructions/DELION_PATTERNS.md DP-4)
- ALL test artifacts MUST be cleaned up
- Global state tests MUST use `#[serial(group_name)]`
- Use strict assertions (`assert_eq!`) instead of loose matching (`contains`)
- Follow Arrange-Act-Assert (AAA) pattern with `// Arrange`, `// Act`, `// Assert` labels
- Use `rstest` for ALL test cases (no plain `#[test]`)

See @instructions/TESTING_STANDARDS.md for comprehensive testing standards.

### File Management

**Critical Rules:**
- **NEVER** save temp files to project directory (use `/tmp`)
- **IMMEDIATELY** delete `/tmp` files when no longer needed
- **IMMEDIATELY** delete backup files (`.bak`, `.backup`, `.old`, `~` suffix)
- NO relative paths beyond one level up (`../..` is forbidden)
- Use absolute paths from crate root or single-level relative paths

### Documentation

**Update Requirements:**
- **ALWAYS** update docs when code changes (same workflow)
- Update all relevant: root `README.md`, `delions/<name>-delion/README.md`, `lib.rs` module docs
- Planned features go in `lib.rs` header (`//! ## Planned Features`), NOT in README.md
- Test all code examples (`cargo test --doc`)
- Verify all links are valid
- **NEVER** document user requests or AI assistant interactions in project documentation
  - Documentation must describe technical reasons, design decisions, and implementation details
  - Avoid phrases like "User requested...", "As requested by...", "User asked..."
  - Focus on the "why" (technical rationale), not the "who asked"

See @instructions/DOCUMENTATION_STANDARDS.md for comprehensive documentation standards, including rustdoc formatting rules (DM-7).

### Git Workflow

**Commit Policy:**
- **NEVER** commit without explicit user instruction
- **NEVER** push without explicit user instruction
- **EXCEPTION**: Plan Mode approval is considered explicit commit authorization
  - When user approves a plan via Exit Plan Mode, implementation and commits are both authorized
  - Upon successful implementation, all planned commits are created automatically without additional confirmation
  - If implementation fails or tests fail, NO commits are created (report to user instead)
- Split commits by specific intent (NOT feature-level goals)
- Each commit MUST be small enough to explain in one line
- Use `git add --patch` or a patch file for partial file commits
- **NEVER** execute batch commits without user confirmation

**Branch Operations:**
- When merging branches and resolving conflicts, execute immediately without entering Plan Mode
- Before creating branches, verify names don't conflict with existing ones using `git worktree list` and `git branch -a`

**PR Conflict Resolution:**
- **MUST** use worktree-based merge strategy for resolving PR conflicts (NOT rebase or force-push)
- **NEVER** use `git rebase` or `git push --force` to resolve PR conflicts

**GitHub Integration:**
- **MUST** use GitHub CLI (`gh`) for all GitHub operations
- For usage questions, prefer GitHub Discussions over Issues
- **NEVER** use raw `curl` or web browser for GitHub operations when `gh` is available
- When GitHub MCP tools return errors (e.g., 404), immediately fall back to `gh` CLI instead of retrying

**GitHub Comments & Interactions:**
- **NEVER** post comments on PRs or Issues without authorization
- Authorization = explicit user instruction OR Plan Mode approval
- Self-initiated comments MUST be previewed and approved by user before posting
- ALL comments MUST be in English and include Claude Code attribution footer
- Comments MUST reference specific code locations with repository-relative paths
- Comments MUST NOT contain user requests, AI interactions, or absolute local paths

See @instructions/GITHUB_INTERACTION.md for comprehensive GitHub interaction guidelines.

See @instructions/COMMIT_GUIDELINE.md for detailed commit guidelines including:
- Commit execution policy (CE-1 ~ CE-4)
- Commit message format (CM-1 ~ CM-3)
- Commit type to CHANGELOG mapping (aligned with `release-plz.toml`)

### Release & Publishing Policy

This project uses **release-plz** for automated per-delion versioning and publishing. See @instructions/DELION_PATTERNS.md DP-8 for the full release workflow.

**Tagging Strategy (Per-Crate Tagging):**
- Format: `<name>-delion@v<version>` (e.g., `auth-delion@v0.1.0`), defined by `git_tag_name = "{{ package }}@v{{ version }}"` in `release-plz.toml`
- Tags are created automatically by release-plz upon Release PR merge
- **NEVER** create release tags manually

**Release Workflow:**
1. Write commits following Conventional Commits format (see @instructions/COMMIT_GUIDELINE.md)
2. Push to `main` branch
3. release-plz opens a Release PR (branch prefix: `release-plz-`) with version bumps and CHANGELOG updates
4. Review and merge the Release PR
5. release-plz publishes to crates.io and creates Git tags automatically

**Commit-to-Version Mapping** (aligned with `release-plz.toml` `commit_parsers`):

| Commit Type | Version Bump | CHANGELOG Section |
|-------------|--------------|-------------------|
| `feat:` | MINOR | Added |
| `fix:` | PATCH | Fixed |
| `feat!:` or `BREAKING CHANGE:` | MAJOR | (respects original section) |
| `perf:` | PATCH | Performance |
| `refactor:` | PATCH | Changed |
| `docs:` | PATCH | Documentation |
| `deprecated:` | PATCH | Deprecated |
| `security:` | PATCH | Security |
| `chore:`, `ci:`, `build:` | PATCH | Maintenance |
| `test:` | PATCH | Testing |
| `style:` | PATCH | Styling |

**Critical Rules:**
- **MUST** use conventional commit format for proper version detection
- **MUST** review Release PRs before merging
- **NEVER** manually bump versions in feature branches (both `Cargo.toml` `version` and `dentdelion.toml` `version` are release-plz-managed)
- **NEVER** change `pr_branch_prefix` from `"release-plz-"` (breaks the two-stage release workflow)
- **NEVER** apply the `release` label to a PR manually — it is reserved for release-plz

### Workflow Best Practices

- Run dry-run for ALL batch operations before actual execution
- Use parallel agents for independent file edits
- NO batch commits (create one at a time with user confirmation)
- Execute straightforward operations (branch deletion, worktree cleanup) immediately without planning

### Issue Handling

**Batch Issue Strategy:**
- Group issues by fix pattern and process as a batch (HA-1)
- Divide work into phases ordered by severity (HA-2)
- Parallelize independent delion work using Agent Teams (HA-3)
- Organize phases into logically grouped branches and PRs (HA-4)

**Work Unit Principles:**
- 1 PR = 1 delion × 1 fix pattern as the basic work unit (WU-1)
- Same-delion related issues MAY be combined into a single PR (WU-2)
- Shared behavior across delions MUST go through the `reinhardt` facade, not inter-delion deps (WU-3, DP-4)

**Upstream Issue Reporting:**
- When a reinhardt-web issue is discovered during awesome-delions development, **immediately** create an issue in `kent8192/reinhardt-web` (UR-1)
- Use `gh issue create -R kent8192/reinhardt-web` for upstream issue creation (UR-2)
- Create a tracking issue in awesome-delions with `upstream-tracking` label for every upstream issue (UR-4)
- Cross-reference between awesome-delions tracking issue and upstream issue bidirectionally (UR-4)
- **NEVER** implement workarounds without creating an upstream issue first (WP-2)

See @instructions/ISSUE_HANDLING.md for comprehensive issue handling principles.

See @instructions/UPSTREAM_ISSUE_REPORTING.md for upstream issue reporting policy.

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
cargo test --doc --workspace  # Documentation tests
```

**Code Quality:**
```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

**TODO Comment Check:**
```bash
# Semgrep: full scan for TODO/FIXME comments
docker run --rm -v "$(pwd):/src" semgrep/semgrep semgrep scan --config .semgrep/ --error --metrics off

# Semgrep: diff-aware scan (compare against main branch)
docker run --rm -v "$(pwd):/src" semgrep/semgrep semgrep scan --config .semgrep/ --baseline-commit origin/main --error --metrics off
```

**Creating a New Delion:**
```bash
cargo generate --path template/ --name <name>-delion --destination delions/
```

See @instructions/DELION_PATTERNS.md DP-5 for template-based creation rules.

**GitHub Operations (using GitHub CLI):**
```bash
# Pull Requests
gh pr create --title "feat(auth-delion): add OAuth backend" --body "Description" --label enhancement
gh pr view [number]
gh pr list --state open
gh pr checks

# Issues
gh issue create --title "Bug: description" --body "Description"
gh issue view [number]
gh issue list

# Releases (managed by release-plz; avoid manual creation)
gh release list
gh release view [tag]
```

**PR/Issue Template Compliance:**
- **PR Template:** `.github/PULL_REQUEST_TEMPLATE.md` (see @instructions/PR_GUIDELINE.md)
- **Issue Templates:** `.github/ISSUE_TEMPLATE/*.yml` (see @instructions/ISSUE_GUIDELINES.md)
- **Note:** GitHub CLI does not auto-apply templates; include template structure in `--body`
- The PR body MUST populate the `## Changed Delions` section

**Linking PRs to Issues:**

Use keywords to auto-close issues on merge: `Fixes #N`, `Closes #N`, `Resolves #N`. Use `Refs #N` for related issues that should NOT be auto-closed.

---

## Review Process

**CI Failure Diagnosis (Known Patterns):**
- Check these recurring patterns first:
  1. rustdoc intra-doc link errors with `-D warnings`
  2. docs.rs build issues from empty code blocks or missing `include` in `Cargo.toml`
  3. release-plz complaining about `Cargo.toml`/`dentdelion.toml` version drift
  4. TODO Check CI blocking new `// TODO` / `// FIXME` / `todo!()`
- Always run `cargo doc --no-deps` locally before pushing doc-related fixes

Before submitting code:

1. **Run all commands:**
   - `cargo check --workspace`
   - `cargo nextest run --workspace`
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo test --doc --workspace`

2. **Iterate until all issues resolved**

3. **Review compliance with standards:**
   - [ ] Module system (@instructions/MODULE_SYSTEM.md)
   - [ ] Delion contract (@instructions/DELION_PATTERNS.md)
   - [ ] Testing standards (@instructions/TESTING_STANDARDS.md)
   - [ ] No anti-patterns (@instructions/ANTI_PATTERNS.md)
   - [ ] Documentation updated (@instructions/DOCUMENTATION_STANDARDS.md)
   - [ ] Git commit policy (@instructions/COMMIT_GUIDELINE.md)
   - [ ] PR guidelines (@instructions/PR_GUIDELINE.md)
   - [ ] GitHub interaction policy (@instructions/GITHUB_INTERACTION.md)
   - [ ] Issue handling principles (@instructions/ISSUE_HANDLING.md)
   - [ ] Upstream issues reported to reinhardt-web (@instructions/UPSTREAM_ISSUE_REPORTING.md)
   - [ ] No unresolved TODO/FIXME comments in new code (TODO Check CI)

---

## Additional Instructions

@CLAUDE.local.md - Project-specific local preferences

---

## Quick Reference

### ✅ MUST DO
- Write ALL code comments in English (no exceptions)
- Use `module.rs` + `module/` directory (NO `mod.rs`)
- Place all delions under `delions/<name>-delion/` with the full layout (DP-1)
- Depend on `reinhardt` facade with feature flags (NOT `reinhardt-dentdelion` directly) (DP-2)
- Keep delion feature surfaces minimal and purpose-named (DP-3)
- Follow `xxx-delion` kebab-case naming convention (DP-1, DP-5)
- Create new delions with `cargo generate --path template/` (DP-5)
- Ship a `dentdelion.toml` whose `name`/`version` match `Cargo.toml` (DP-6)
- Expose a deliberate public API via explicit `pub use` (DP-7)
- Scope PRs and commits so release-plz produces clean per-delion version bumps (DP-8)
- Update docs with code changes (same workflow)
- Clean up ALL test artifacts
- Delete temp files from `/tmp` immediately
- Wait for explicit user instruction before commits
- Understand that Plan Mode approval authorizes both implementation and commits
- Mark placeholders with `todo!()` or `// TODO:`
- Use `#[serial(group_name)]` for global state tests
- Split commits by specific intent, not features
- Follow Conventional Commits v1.0.0 format: `<type>[scope]: <description>`
- Start commit description with lowercase letter
- Use `!` notation for breaking changes
- Write commit descriptions as standalone CHANGELOG entries
- Use `security` type for security vulnerability fixes (dedicated CHANGELOG section)
- Use `deprecated` type for marking features/APIs as deprecated (dedicated CHANGELOG section)
- Use GitHub CLI (`gh`) for all GitHub operations
- Search existing issues before creating new ones
- Use appropriate issue templates for all issues
- Apply at least one type label to every issue
- Report security vulnerabilities privately via GitHub Security Advisories
- Use `.github/labels.yml` as source of truth for label definitions
- Follow PR/Issue template structure when creating via `gh` CLI
- Populate the `## Changed Delions` section in every PR body
- Use 1 PR = 1 delion × 1 fix pattern as the basic work unit
- Promote shared behavior to the `reinhardt` facade instead of introducing inter-delion deps (DP-4, WU-3)
- Use `rstest` for ALL test cases (no plain `#[test]`)
- Follow Arrange-Act-Assert (AAA) pattern with `// Arrange`, `// Act`, `// Assert`
- Wrap generic types in backticks in doc comments: `` `Result<T>` ``
- Wrap macro attributes in backticks: `` `#[inject]` ``
- Wrap URLs in angle brackets or backticks: `<https://...>`
- Specify language for code blocks: ` ```rust `
- Wrap bracket patterns in backticks: `` `array[0]` ``
- Use backticks (not intra-doc links) for feature-gated types
- Prefer Mermaid diagrams (via `aquamarine`) for architecture documentation instead of ASCII art
- Resolve all `todo!()` and `// TODO:` before merging PR (enforced by TODO Check CI)
- Preview and get user confirmation before posting self-initiated GitHub comments
- Include Claude Code attribution footer on all GitHub comments
- Use repository-relative paths (not absolute) in GitHub comments
- Provide structured agent context using AC-2 template format
- Fall back to `gh` CLI when GitHub MCP tools return errors
- Wait for Copilot review after PR creation and handle all comments (RP-6)
- Resolve all Copilot review conversations before considering PR complete
- Verify branch name uniqueness before creation
- Check known CI failure patterns before deep investigation
- Run `cargo doc --no-deps` locally before pushing doc-related fixes
- Execute merge/conflict resolution and straightforward operations immediately without Plan Mode
- Use worktree-based merge strategy for PR conflict resolution (NOT rebase/force-push)
- Apply `agent-suspect` label to all agent-detected bug Issues
- Verify agent-detected bugs independently before removing `agent-suspect` label
- Use three-dot diff (`main...branch`) for PR diff verification
- Create issues in reinhardt-web immediately upon discovering upstream bugs
- Create a tracking issue in awesome-delions with `upstream-tracking` label for every upstream issue (UR-4)
- Cross-reference between awesome-delions tracking issue and reinhardt-web issue bidirectionally (UR-4)
- Create upstream issue before implementing any workaround (WP-2)
- Include the ideal implementation as a comment when introducing workaround code (WP-3)

### ❌ NEVER DO
- Use `mod.rs` files (deprecated pattern)
- Depend directly on `reinhardt-dentdelion` or any internal reinhardt crate (DP-2)
- Use wildcard or `path`-based `reinhardt` dependencies (DP-2)
- Declare another delion as a dependency (DP-4)
- Hand-create a delion by copying an existing one (DP-5)
- Hand-edit `Cargo.toml`/`dentdelion.toml` `version` in a feature branch (DP-6, DP-8)
- Change `pr_branch_prefix` in `release-plz.toml` (DP-8)
- Apply the `release` label manually (reserved for release-plz)
- Commit without user instruction (except Plan Mode approval)
- Leave docs outdated after code changes
- Document user requests or AI interactions in project documentation
- Put planned features in README.md (use `lib.rs` instead)
- Save files to project directory (use `/tmp`)
- Leave backup files (`.bak`, `.backup`, `.old`, `~`)
- Create skeleton tests (tests without assertions)
- Use loose assertions (`contains`) without justification
- Use glob imports (`use module::*`)
- Create circular dependencies
- Leave unmarked placeholder implementations
- Use `#[allow(...)]` without explanatory comments
- Use alternative TODO notations (`FIXME:`, `NOTE:` for unimplemented features)
- Create batch commits without user confirmation
- Use relative paths beyond `../`
- Write vague commit descriptions (e.g., "fix issue", "update code")
- Start commit description with uppercase letter
- End commit description with a period
- Omit `!` or `BREAKING CHANGE:` for API-breaking changes
- Create issues without appropriate labels
- Create public issues for security vulnerabilities
- Create duplicate issues without searching first
- Skip issue templates when creating issues
- Use non-English in issue titles or descriptions
- Mix changes to unrelated delions in a single PR
- Mix unrelated fix patterns in a single PR
- Use plain `#[test]` instead of `#[rstest]`
- Use non-standard phase labels in tests (`// Setup`, `// Execute`, `// Verify`)
- Write generic types without backticks in doc comments (causes HTML tag warnings)
- Write macro attributes without backticks in doc comments (causes unresolved link warnings)
- Write bare URLs in doc comments (causes bare URL warnings)
- Use intra-doc links for feature-gated items (causes unresolved link warnings)
- Create new ASCII art diagrams in doc comments (use Mermaid instead)
- Merge PR with unresolved `todo!()` or `// TODO:` comments (blocked by TODO Check CI)
- Post GitHub comments without authorization
- Include absolute local paths in GitHub comments
- Post vague or non-actionable GitHub comments
- Skip Claude Code attribution footer on GitHub comments
- Create PRs/Issues without following template structure
- Skip the `## Changed Delions` section in PR bodies
- Enter Plan Mode for merge operations, branch deletion, or worktree cleanup
- Retry GitHub MCP tools after errors instead of falling back to `gh` CLI
- Leave Copilot review conversations unresolved on PRs
- Accept Copilot suggestions that contradict project conventions without evaluation
- Use rebase or force-push to resolve PR conflicts (use worktree merge instead)
- Remove `agent-suspect` label without independent verification
- Use two-dot diff (`main..branch`) for PR verification (includes merge history noise)
- Delay reporting reinhardt-web issues discovered during awesome-delions development
- Implement workarounds for reinhardt-web issues without creating an upstream issue first
- Introduce workaround code without an ideal implementation comment (WP-3)
- Create upstream issues without corresponding awesome-delions tracking issues (UR-4)
- Report delion-specific issues to the reinhardt-web repository

### 📚 Detailed Standards

For comprehensive guidelines, see:
- **Module System**: @instructions/MODULE_SYSTEM.md
- **Delion Patterns**: @instructions/DELION_PATTERNS.md
- **Testing**: @instructions/TESTING_STANDARDS.md
- **Anti-Patterns**: @instructions/ANTI_PATTERNS.md
- **Documentation**: @instructions/DOCUMENTATION_STANDARDS.md
- **Git Commits**: @instructions/COMMIT_GUIDELINE.md
- **Issues**: @instructions/ISSUE_GUIDELINES.md
- **Issue Handling**: @instructions/ISSUE_HANDLING.md
- **Pull Requests**: @instructions/PR_GUIDELINE.md
- **GitHub Interactions**: @instructions/GITHUB_INTERACTION.md
- **Upstream Issue Reporting**: @instructions/UPSTREAM_ISSUE_REPORTING.md
- **GitHub Discussions**: https://github.com/kent8192/awesome-delions/discussions
- **Label Definitions**: `.github/labels.yml`
- **cargo-generate template**: `template/`
- **release-plz configuration**: `release-plz.toml`
- **Project Overview**: `README.md`

---

**Note**: This CLAUDE.md focuses on core rules and quick reference. All detailed standards, examples, and comprehensive guides are in the `instructions/` directory. Always review CLAUDE.md before starting work, and consult detailed documentation as needed.
