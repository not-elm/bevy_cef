# GitHub PR Template Design

## Goal

Create a single GitHub PR template that enforces a consistent structure across all pull requests with three sections: Problem, Solution, and Test Pattern.

## Design

### File Location

`.github/pull_request_template.md`

GitHub automatically uses this file as the default PR body when creating a new pull request.

### Template Structure

Three sections with HTML comment guides:

1. **Problem** — What issue does this PR address? Why is the change needed?
2. **Solution** — How does this PR solve the problem? Key design decisions?
3. **Test Pattern** — How was the change verified? (e.g., which example was run, manual steps taken)

### Format Decisions

- **Language**: English (matches existing commit messages and codebase)
- **Single template**: One universal template for all PR types (no per-type variants)
- **HTML comments for guidance**: `<!-- -->` comments provide prompts without cluttering the rendered PR. Authors do not need to delete them.
- **No change-type checkboxes**: Labels via `release.yml` already categorize changes; duplicating in the template adds friction.
- **Test Pattern hint**: The comment includes `cargo run --example` as an example since the project relies on example-based testing rather than automated test suites.

### Template Content

```markdown
## Problem
<!-- What issue does this PR address? Why is the change needed? -->

## Solution
<!-- How does this PR solve the problem? Key design decisions? -->

## Test Pattern
<!-- How did you verify the change? (e.g., cargo run --example simple --features debug) -->
```
