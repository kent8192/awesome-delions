# Pull Request Guidelines

## Purpose

This file defines the pull request (PR) policy for the awesome-delions project. These rules ensure clear communication, proper review process, and consistent PR formatting across the development lifecycle.

---

## Language Requirements

### LR-1 (MUST): English-Only Policy

- **ALL** PR titles MUST be written in English
- **ALL** PR descriptions MUST be written in English
- **ALL** PR comments and discussions MUST be written in English
- This ensures accessibility for international contributors and maintainers

---

## PR Creation Policy

### PC-1 (MUST): Use GitHub MCP or CLI

- **MUST** prefer GitHub MCP tools (`create_pull_request`) for creating pull requests when available
- **Fallback**: Use GitHub CLI (`gh pr create`) when GitHub MCP is not available
- **NEVER** use web browser UI for PR creation when MCP or CLI is available

The following diagram summarizes the PR creation flow:

```mermaid
flowchart TD
    A[Create new PR] --> B{GitHub MCP available?}
    B -->|Yes| C[Use create_pull_request MCP tool]
    B -->|No| D[Use gh pr create CLI]
    C --> E[Follow PR template structure]
    D --> E
    E --> F[Add appropriate labels]
    F --> G[Run pre-review checks]
    G --> H[Request review]
```

**PR Template Location:** `.github/PULL_REQUEST_TEMPLATE.md`

**Example:**
```bash
gh pr create --title "feat(auth-delion): add exponential backoff to refresh-token issuance" \
  --body "$(cat <<'EOF'
## Summary

- Implement exponential backoff in refresh-token issuance path
- Add configurable max retry duration
- Include unit tests for backoff calculation

## Changed Delions

- `auth-delion`

## How Was This Tested?

- [x] `cargo nextest run --workspace` passes
- [x] All existing tests pass
- [x] Manual testing with simulated upstream errors

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

### PC-2 (MUST): Follow PR Template Structure

**PR Template Location:** `.github/PULL_REQUEST_TEMPLATE.md`

When creating PRs via `gh pr create`, the `--body` content MUST follow the PR template structure defined in `.github/PULL_REQUEST_TEMPLATE.md`. In particular the `## Changed Delions` section MUST list every delion crate touched by the PR.

**CLI Note:** GitHub CLI does not automatically apply the PR template like the Web UI. Read the template file and include its structure in your `--body` content.

### PC-3 (MUST): Branch Naming

- Branch names SHOULD follow the pattern: `<type>/<delion-or-scope>-<short-description>`
- Types: `feature`, `fix`, `refactor`, `docs`, `test`, `chore`, etc.
- Scope: Delion name (without `-delion` suffix) or a workspace-level scope (`template`, `workspace`, `ci`)
- Short description: Kebab-case brief summary

**Examples:**
```
feature/auth-delion-oauth-backend
fix/session-delion-timeout-handling
refactor/audit-log-delion-event-schema
docs/auth-delion-usage-guide
test/session-delion-integration
chore/ci-update-rust-toolchain
chore/template-bump-reinhardt-version
```

### PC-4 (SHOULD): Draft PRs for Work in Progress

- Use draft PRs for incomplete work
- Convert to ready for review when all tests pass
- Draft PRs allow early feedback without formal review requests

**Example:**
```bash
gh pr create --draft --title "feat(auth-delion): add OAuth backend (WIP)"
```

### PC-5 (MUST): PR Labels

- **MUST** add appropriate labels to every PR
- Use GitHub MCP, GitHub CLI, or web UI to add labels

The canonical list of labels is defined in `.github/labels.yml`.

**Required Labels by PR Type:**

| PR Type | Required Label | Additional Labels |
|---------|---------------|-------------------|
| New feature | `enhancement` | Scope-specific labels (`plugin-api`, `wasm`, `template`) |
| New delion crate | `enhancement`, `new-delion` | - |
| Bug fix | `bug` | Severity labels if available |
| Documentation | `documentation` | - |
| Dependency updates | `dependencies` | - |
| Breaking change | Primary label + `breaking-change` | - |

**NEVER** apply the `release` label to a PR manually — it is reserved for release-plz-generated Release PRs.

**Label Application Examples:**

```bash
# Feature PR with label
gh pr create --title "feat(auth-delion): add OAuth backend" \
  --label enhancement

# Bug fix PR with label
gh pr create --title "fix(session-delion): resolve timeout edge case" \
  --label bug

# Documentation PR with label
gh pr create --title "docs(auth-delion): add OAuth usage guide" \
  --label documentation

# Dependency update PR with label
gh pr create --title "chore(deps): bump reinhardt from 0.1 to 0.2" \
  --label dependencies
```

**Adding Labels to Existing PR:**

```bash
# Add single label
gh pr edit <number> --add-label enhancement

# Add multiple labels
gh pr edit <number> --add-label bug,help wanted

# Remove label
gh pr edit <number> --remove-label invalid
```

---

## PR Title Format

### TF-1 (MUST): Follow Conventional Commits

PR titles MUST follow the same format as commit messages:

```
<type>[optional scope][optional !]: <description>

Examples:
feat(auth-delion): add OAuth backend support
fix(session-delion): resolve timeout edge case
feat(auth-delion)!: rename AuthConfig.ttl field to ttl_seconds
```

**Requirements:**
- **Type**: One of the defined types (feat, fix, refactor, docs, etc.)
- **Scope**: Delion name (OPTIONAL but RECOMMENDED), or `workspace`/`template` for workspace-level changes
- **Breaking Change Indicator**: Append `!` for breaking changes
- **Description**: Concise summary in English
  - **MUST** start with lowercase letter
  - **MUST** be specific and descriptive
  - **MUST NOT** end with a period
  - Keep under 72 characters for readability

**See**: @instructions/COMMIT_GUIDELINE.md for detailed commit type definitions

---

## PR Description Format

### DF-1 (MUST): Standard Structure

PR descriptions MUST follow the structure defined in `.github/PULL_REQUEST_TEMPLATE.md`.

**Required Sections:** Summary, Type of Change, Changed Delions, Motivation and Context, How Was This Tested, Checklist, Labels to Apply

**Optional Sections:** Performance Impact, Breaking Changes, Related Issues, Additional Context

**Footer:** Include Claude Code attribution for AI-assisted PRs

### DF-2 (MUST): Linking PRs to Issues

PRs should be linked to related issues using GitHub's supported keywords:
- `close`, `closes`, `closed`
- `fix`, `fixes`, `fixed`
- `resolve`, `resolves`, `resolved`

**Examples:**
```markdown
## Related Issues

Fixes #42
Closes #43, closes #44
Refs #50 (related but not closed)
```

**Important Notes:**
- Keywords only work when PR targets the **default branch** (main)
- Use `Refs #N` for related issues that should NOT be auto-closed

### DF-3 (SHOULD): Additional Context

Include additional sections when relevant:

- **Migration Guide**: For breaking changes with complex migration
- **Performance Impact**: For performance-related changes
- **Security Considerations**: For security-related changes
- **Minimum reinhardt Version**: Note the minimum `reinhardt` facade version required

---

## PR Review Process

### RP-1 (MUST): Pre-Review Checklist

Before requesting review, ensure:

- [ ] All CI checks pass
- [ ] All tests pass locally
- [ ] Code follows project style guidelines
- [ ] Documentation is updated
- [ ] Commit history is clean and logical
- [ ] PR description is complete and accurate
- [ ] `## Changed Delions` section accurately lists all touched delion crates

**Commands to run:**
```bash
cargo check --workspace
cargo nextest run --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --doc --workspace
```

### RP-2 (SHOULD): Self-Review

- Review your own PR before requesting review from others
- Check for:
  - Unnecessary debug code or comments
  - Proper error handling (no `unwrap()` in library code paths)
  - Test coverage
  - Documentation completeness
  - Code clarity and readability

### RP-3 (MUST): Address Review Comments

- Respond to all review comments
- Mark conversations as resolved when addressed
- Request re-review after making changes
- Be respectful and constructive in discussions

### RP-6 (MUST): Copilot Review Workflow

After creating a PR, Claude Code MUST wait for GitHub Copilot's automated review and handle it as part of the PR completion process.

**Workflow:**

1. **Wait for Copilot review** — After PR creation, poll for Copilot's review using `gh pr checks` and `gh api` to monitor review status
2. **Retrieve review comments** — Once Copilot review arrives, fetch all review comments using `gh pr view <number> --comments` and `gh api repos/{owner}/{repo}/pulls/{number}/comments`
3. **Evaluate each comment** — For each Copilot suggestion:
   - Assess whether the suggestion is valid and improves code quality
   - Check if the suggestion aligns with project conventions (CLAUDE.md, instructions/)
   - Determine if the change is necessary or cosmetic
4. **Act on evaluation:**
   - **Valid suggestion** → Fix the code, commit the change, then resolve the conversation
   - **Invalid or unnecessary suggestion** → Resolve the conversation without changes (reply with brief technical justification if needed)
5. **Verify completion** — Ensure all Copilot review conversations are resolved before considering the PR ready

The following diagram summarizes the Copilot review handling flow:

```mermaid
flowchart TD
    A[PR created] --> B[Poll for Copilot review]
    B --> C{Copilot review received?}
    C -->|No| D[Wait and retry]
    D --> C
    C -->|Yes| E[Fetch all review comments]
    E --> F[Evaluate each comment]
    F --> G{Suggestion valid?}
    G -->|Yes| H[Fix code and commit]
    H --> I[Resolve conversation]
    G -->|No| J[Resolve conversation<br/>with justification]
    I --> K{More comments?}
    J --> K
    K -->|Yes| F
    K -->|No| L[All conversations resolved]
```

**Polling Commands:**
```bash
# Check PR review status
gh pr checks <number>

# List reviews on a PR
gh api repos/{owner}/{repo}/pulls/{number}/reviews

# Get review comments
gh api repos/{owner}/{repo}/pulls/{number}/comments

# Resolve a review thread (GraphQL)
gh api graphql -f query='
  mutation {
    resolveReviewThread(input: {threadId: "<thread_id>"}) {
      thread { isResolved }
    }
  }
'
```

**Important Notes:**
- Copilot review is treated as automated feedback, NOT as a blocking human review
- Plan Mode approval authorizes handling Copilot review comments (no additional user confirmation needed)
- Fixes for Copilot suggestions follow the same commit policy as other changes
- If Copilot review does not arrive within a reasonable time, proceed without it

### RP-4 (SHOULD): Keep PRs Small

- Aim for PRs under 400 lines of changes
- Split large features into multiple PRs (1 delion × 1 fix pattern — see @instructions/ISSUE_HANDLING.md WU-1)
- Each PR should have a single, clear purpose
- Smaller PRs are easier to review and less risky to merge

### RP-5 (MUST): Use Three-Dot Diff for PR Verification

- **MUST** use three-dot diff (`...`) to verify PR changes from the merge base
- Three-dot diff excludes merge history noise and shows only changes introduced by the PR

**Commands:**
```bash
# Three-dot diff: shows changes from merge base (CORRECT)
git diff main...feature-branch

# GitHub CLI (uses three-dot diff by default)
gh pr diff <number>
```

---

## PR Conflict Resolution

### CR-1 (MUST): Worktree-Based Merge Strategy

PR conflicts MUST be resolved using a worktree-based merge strategy. Rebase and force-push are NOT allowed for conflict resolution.

**Procedure:**

1. Create a worktree for the source branch:
   ```bash
   git worktree add /tmp/<worktree-name> <source-branch>
   ```
2. In the worktree, merge the target branch:
   ```bash
   cd /tmp/<worktree-name>
   git merge <target-branch>
   ```
3. Resolve conflicts and commit:
   ```bash
   # Resolve conflicts in files
   git add <resolved-files>
   git commit
   ```
4. Push and clean up:
   ```bash
   git push origin <source-branch>
   cd -
   git worktree remove /tmp/<worktree-name>
   ```

The following sequence diagram shows the worktree-based conflict resolution workflow:

```mermaid
sequenceDiagram
    participant D as Developer
    participant M as Main Repo
    participant W as Worktree
    participant R as Remote

    D->>M: git worktree add /tmp/wt source-branch
    D->>W: cd /tmp/wt
    D->>W: git merge target-branch
    Note over W: Resolve conflicts in files
    D->>W: git add + git commit
    D->>R: git push origin source-branch
    D->>M: git worktree remove /tmp/wt
```

### CR-2 (NEVER): Prohibited Approaches

- **NEVER** use `git rebase` to resolve PR conflicts
- **NEVER** use `git push --force` or `git push --force-with-lease` for conflict resolution
- **NEVER** use `git reset --hard` as part of conflict resolution workflow

---

## PR Merge Policy

### MP-1 (MUST): Merge Requirements

A PR can only be merged when:

- All CI checks pass
- All conversations are resolved
- At least one approval from a maintainer (if required by repo settings)
- No merge conflicts with base branch
- All commits follow commit guidelines (@instructions/COMMIT_GUIDELINE.md)

### MP-2 (MUST): Merge Strategy

**Squash and Merge** (Default):
- Combine all PR commits into a single commit
- Use PR title as commit message
- Use for feature branches with multiple interim commits

**Rebase and Merge**:
- Preserve individual commits
- Prefer for PRs with clean, logical commit history

**Merge Commit** (Avoid for features):
- Only use for merging long-lived branches

**Release PR exception:** Release PRs generated by release-plz are merged according to release-plz's own workflow — do NOT hand-squash them.

### MP-3 (SHOULD): Delete Branch After Merge

- Delete feature branches after successful merge
- Keeps repository clean

---

## Special Cases

### Documentation-Only PRs

For documentation changes:

**Title Format:**
```
docs(<scope>): <description>

Example:
docs(auth-delion): add OAuth usage guide
docs(workspace): clarify delion naming convention
```

---

## Quick Reference

### ✅ MUST DO
- Write all PR content in English
- Use GitHub MCP (`create_pull_request`) or `gh pr create` for creating PRs
- Follow PR template structure from `.github/PULL_REQUEST_TEMPLATE.md`, including `## Changed Delions`
- Follow Conventional Commits format for titles
- Include Summary, Type of Change, Changed Delions, Motivation and Context, How Was This Tested, Checklist sections
- Include Labels to Apply section with appropriate type and scope labels
- Run all checks before requesting review
- Address all review comments
- Wait for Copilot review after PR creation and handle all comments (RP-6)
- Resolve all Copilot review conversations before considering PR complete
- Ensure all CI checks pass before merge
- Use three-dot diff (`main...branch`) for PR verification to exclude merge history noise

### ❌ NEVER DO
- Write PR titles or descriptions in non-English languages
- Create PRs without following PR template structure
- Skip required sections (Summary, Type of Change, Changed Delions, Motivation and Context, How Was This Tested, Checklist)
- Skip Labels to Apply section
- Apply `release` label to a non-release PR (reserved for release-plz)
- Merge with failing CI checks
- Leave unresolved review comments (including Copilot review)
- Force push after review has started (unless explicitly requested)
- Use rebase or force-push to resolve PR conflicts (use worktree merge instead)
- Use two-dot diff (`main..branch`) for PR verification (includes merge history noise)

---

## Related Documentation

- **Main Quick Reference**: @CLAUDE.md (see Quick Reference section)
- **Issue Handling Principles**: @instructions/ISSUE_HANDLING.md
- **Commit Guidelines**: @instructions/COMMIT_GUIDELINE.md
- **GitHub Interaction**: @instructions/GITHUB_INTERACTION.md
- **Delion Patterns**: @instructions/DELION_PATTERNS.md
