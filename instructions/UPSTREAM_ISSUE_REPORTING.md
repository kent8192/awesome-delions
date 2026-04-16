# Upstream Issue Reporting

## Purpose

This file defines the policy for reporting issues discovered in upstream dependencies during awesome-delions development. awesome-delions is a collection of plugin crates (delions) built on top of [reinhardt-web](https://github.com/kent8192/reinhardt-web) via the `reinhardt` facade, and issues found in reinhardt-web during awesome-delions work MUST be reported immediately to the upstream repository.

---

## Scope

### US-1 (MUST): Target Repositories

This policy applies to the following upstream repositories:

| Repository | URL | Relationship |
|------------|-----|-------------|
| reinhardt-web | `https://github.com/kent8192/reinhardt-web` | Core framework that awesome-delions consumes via the `reinhardt` facade (incl. `reinhardt-dentdelion`) |

**Future upstream dependencies** should be added to this table as the project grows.

---

## Reporting Policy

### UR-1 (MUST): Immediate Reporting

When a bug, missing feature, documentation gap, or unexpected behavior in reinhardt-web is discovered during awesome-delions development, an issue MUST be created in the reinhardt-web repository **immediately** upon discovery.

**Rationale:** Delaying upstream issue reporting increases the risk of:
- Forgetting the issue details
- Building workarounds that mask the root cause
- Other delion authors hitting the same problem without context

The following diagram summarizes the upstream issue reporting flow:

```mermaid
flowchart TD
    A[Discover issue during awesome-delions development] --> B{Is the issue in a delion itself?}
    B -->|Yes| C[Create issue in awesome-delions repo]
    B -->|No| D{Is the issue in reinhardt-web?}
    D -->|Yes| E[Create issue in reinhardt-web repo immediately]
    D -->|No| F[Investigate further]
    E --> E2["Create tracking issue in awesome-delions repo<br/>with upstream-tracking label"]
    E2 --> E3["Cross-reference both issues"]
    E3 --> G{Does the delion need a workaround?}
    G -->|Yes| H["Add workaround in the delion with<br/>comment referencing both issues"]
    G -->|No| I[Continue delion development]
    H --> I
```

### UR-2 (MUST): Use GitHub CLI with Repository Flag

Issues in upstream repositories MUST be created using `gh issue create` with the `-R` flag:

```bash
# Create issue in reinhardt-web
gh issue create -R kent8192/reinhardt-web \
  --title "Bug: description of the issue" \
  --body "$(cat <<'EOF'
## Description

[Clear description of the issue]

## Reproduction Steps

1. [Step 1]
2. [Step 2]

## Expected Behavior

[What should happen]

## Actual Behavior

[What actually happens]

## Context

Discovered during awesome-delions development while [brief context].

awesome-delions tracking issue: https://github.com/kent8192/awesome-delions/issues/N

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

### UR-3 (MUST): Issue Content Requirements

Upstream issues MUST:
- Be written in **English**
- Follow the upstream repository's issue templates if available
- Include clear reproduction steps
- Include the discovery context (e.g., "discovered while implementing auth-delion handler")
- Reference related awesome-delions issues or PRs if applicable
- Include Claude Code attribution footer
- **NOT** include absolute local paths or user-specific information

### UR-4 (MUST): Cross-Referencing with Tracking Issues

When an upstream issue is created, a corresponding **tracking issue** MUST also be created in the awesome-delions repository. Both issues MUST reference each other.

**Rationale:** Creating a tracking issue in awesome-delions ensures that:
- Upstream dependencies are visible in the awesome-delions issue tracker
- Workaround removal can be tracked alongside delion development milestones
- Contributors can discover upstream blockers without checking external repositories

**Procedure:**

1. **Create the upstream issue** in reinhardt-web (UR-1, UR-2)
2. **Create a tracking issue** in the awesome-delions repository referencing the upstream issue
3. **Update the upstream issue** to reference the awesome-delions tracking issue
4. **In the delion codebase**: Add a comment referencing both issues where a workaround is applied

The following diagram shows the cross-referencing workflow:

```mermaid
sequenceDiagram
    participant D as Developer
    participant RW as reinhardt-web
    participant AD as awesome-delions

    D->>RW: 1. Create upstream issue (reinhardt-web#42)
    D->>AD: 2. Create tracking issue (awesome-delions#15)<br/>References reinhardt-web#42
    D->>RW: 3. Update reinhardt-web#42<br/>Add link to awesome-delions#15
    D->>AD: 4. Add workaround code comment<br/>referencing both issues
```

**awesome-delions tracking issue template:**

```bash
gh issue create \
  --title "Upstream: [brief description] (reinhardt-web#N)" \
  --label upstream-tracking \
  --body "$(cat <<'EOF'
## Upstream Issue

Tracking upstream issue: https://github.com/kent8192/reinhardt-web/issues/N

## Impact on awesome-delions

[Describe how this upstream issue affects one or more delions]

## Affected Delions

- `auth-delion`
- `session-delion`

## Workaround

- [ ] Workaround applied in affected delions (if needed)
- [ ] Code comment added referencing upstream issue

## Resolution Criteria

This issue should be closed when:
- The upstream issue is resolved AND
- The delion workaround (if any) is removed

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

**Workaround comment format:**
```rust
// Workaround for kent8192/reinhardt-web#42 (tracked in awesome-delions#15)
// Remove this workaround when the upstream issue is resolved.
//
// Ideal implementation (without workaround):
//   [code showing the intended implementation without the workaround]
```

### UR-5 (SHOULD): Label Application

Apply appropriate labels to upstream issues based on the issue type:

| Issue Type | Labels |
|------------|--------|
| Bug | `bug` |
| Missing feature | `enhancement` |
| Documentation gap | `documentation` |
| Performance issue | `performance` |

**Note:** Available labels depend on the upstream repository's configuration. Check available labels before applying.

---

## Issue Categories

### IC-1: What Qualifies as an Upstream Issue

Report to the upstream repository when:

- A reinhardt-web API behaves unexpectedly or inconsistently
- A reinhardt-web feature is missing that a delion requires
- reinhardt-web documentation is incorrect, incomplete, or misleading
- A reinhardt-web dependency causes a conflict or vulnerability
- reinhardt-web build or test infrastructure has issues that affect downstream consumers
- reinhardt-web type signatures or trait implementations are incorrect
- The `reinhardt` facade does not re-export a required item from an underlying crate

### IC-2: What Does NOT Qualify

Do **NOT** report to the upstream repository when:

- The issue is in delion-specific code (report in awesome-delions repo)
- The issue is an awesome-delions design decision that differs from reinhardt-web conventions
- The issue is a feature request specific to a single delion's use case with no general applicability
- The issue is a misunderstanding of reinhardt-web's intended behavior (use Discussions instead)

---

## Workaround Policy

### WP-1 (SHOULD): Temporary Workarounds

When an upstream issue blocks delion development:

1. Create the upstream issue first (UR-1)
2. Create the awesome-delions tracking issue with `upstream-tracking` label (UR-4)
3. Cross-reference both issues (UR-4)
4. Implement a minimal workaround in the affected delion(s)
5. Mark the workaround with a comment referencing both issues (UR-4)
6. Track the upstream issue for resolution; close the awesome-delions tracking issue when resolved

**Workaround rules:**
- Keep workarounds minimal and isolated
- Document the workaround clearly
- Remove the workaround when the upstream issue is resolved

### WP-2 (MUST): No Silent Workarounds

**NEVER** implement workarounds for upstream issues without:
1. Creating an upstream issue first
2. Adding a reference comment in the workaround code

### WP-3 (MUST): Include Ideal Implementation in Workaround Comments

Every workaround comment MUST include the **ideal implementation** — the code that should replace the workaround once the upstream issue is resolved. This enables future developers to remove the workaround without re-investigating the intended design.

**Rationale:**
- Issue references explain *why* a workaround exists, but not *what the code should look like* without it
- The ideal implementation reduces the cost and risk of workaround removal
- Without it, developers must reverse-engineer the intended behavior from issue discussions

**Extended workaround comment template:**
```rust
// Workaround for kent8192/reinhardt-web#42 (tracked in awesome-delions#15)
// Remove this workaround when the upstream issue is resolved.
//
// Ideal implementation (without workaround):
//   let handler = reinhardt::dentdelion::Handler::from_context(ctx);
//   // Handler resolves directly from the facade context without
//   // needing the manual delion-side adapter below.
```

**Rules:**
- The ideal implementation MUST be syntactically plausible (not necessarily compilable against the current upstream API)
- Keep the ideal implementation concise — show only the key difference, not the entire function
- If the ideal implementation depends on an upstream API that does not yet exist, describe it in pseudocode with a brief note

**Example with pseudocode:**
```rust
// Workaround for kent8192/reinhardt-web#99 (tracked in awesome-delions#30)
// Remove this workaround when the upstream issue is resolved.
//
// Ideal implementation (without workaround):
//   // Requires reinhardt to re-export `dentdelion::Lifecycle::on_shutdown()`
//   lifecycle.on_shutdown(|| async { /* cleanup */ }).await?;
```

---

## Quick Reference

### ✅ MUST DO
- Create issues in reinhardt-web immediately upon discovering upstream bugs (UR-1)
- Use `gh issue create -R kent8192/reinhardt-web` for upstream issue creation (UR-2)
- Write all upstream issues in English (UR-3)
- Follow upstream repository's issue templates when available (UR-3)
- Create a tracking issue in awesome-delions for every upstream issue with `upstream-tracking` label (UR-4)
- Cross-reference between awesome-delions tracking issue and upstream issue bidirectionally (UR-4)
- Add workaround comments referencing both upstream and awesome-delions tracking issues (UR-4)
- Include ideal implementation in all workaround comments (WP-3)
- Create upstream issue before implementing any workaround (WP-2)

### ❌ NEVER DO
- Delay reporting upstream issues discovered during awesome-delions development
- Implement workarounds without creating upstream issues first (WP-2)
- Introduce workaround code without an ideal implementation comment (WP-3)
- Create upstream issues without corresponding awesome-delions tracking issues (UR-4)
- Include absolute local paths in upstream issues (UR-3)
- Report delion-specific issues to the reinhardt-web repository (IC-2)
- Forget to cross-reference between awesome-delions and upstream issues (UR-4)

---

## Related Documentation

- **Issue Guidelines**: @instructions/ISSUE_GUIDELINES.md
- **Issue Handling**: @instructions/ISSUE_HANDLING.md
- **GitHub Interaction**: @instructions/GITHUB_INTERACTION.md
- **Delion Patterns**: @instructions/DELION_PATTERNS.md
- **Main Quick Reference**: @CLAUDE.md (see Quick Reference section)
- **reinhardt-web Repository**: <https://github.com/kent8192/reinhardt-web>

---

**Note**: This document focuses on reporting issues to upstream dependencies. For awesome-delions-specific issue management, see @instructions/ISSUE_GUIDELINES.md. For batch issue handling strategy, see @instructions/ISSUE_HANDLING.md.
