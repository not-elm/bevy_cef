# Codex PR Review Workflow Design

## Goal

Add a GitHub Actions workflow that uses OpenAI Codex to automatically review pull requests for code quality.

## Approach

Use the official `openai/codex-action@v1` GitHub Action with a two-job pipeline:
1. Run Codex to generate review feedback
2. Post the feedback as a PR Review comment

## Workflow Configuration

**File:** `.github/workflows/codex-pr-review.yml`

**Trigger:** `pull_request` events: `opened`, `synchronize`, `reopened`

### Job 1: `codex`

- Checkout PR merge ref via `actions/checkout@v5`
- Pre-fetch base and head refs for diff comparison
- Run `openai/codex-action@v1` with:
  - `openai-api-key` from GitHub Secrets (`OPENAI_API_KEY`)
  - `prompt-file`: `.github/codex/prompts/review.md`
  - `output-file`: `codex-output.md`
  - `safety-strategy`: `drop-sudo` (prevents Codex from accessing API key)
  - `sandbox`: `workspace-write`
- Output: `final-message` as job output

### Job 2: `post_review`

- Depends on `codex` job
- Runs only if `final_message` is not empty
- Uses `actions/github-script@v7` to call GitHub PR Review API
- Event type: `COMMENT` (no auto-approve/reject to avoid false positives)

### Concurrency

```yaml
concurrency:
  group: codex-review-${{ github.event.pull_request.number }}
  cancel-in-progress: true
```

### Permissions

- `contents: read`
- `pull-requests: write`

## Review Prompt

**File:** `.github/codex/prompts/review.md`

Instructs Codex to:
1. Read `CLAUDE.md` for project-specific architecture and conventions
2. Review changes for: bugs, performance, security, style/idioms, API design
3. Focus on the diff only, be concise, flag only actionable issues

## Files Created

1. `.github/workflows/codex-pr-review.yml` - Workflow definition
2. `.github/codex/prompts/review.md` - Review prompt

## Prerequisites

- `OPENAI_API_KEY` must be set in repository Secrets (Settings > Secrets and variables > Actions)

## Security

- `drop-sudo`: Codex cannot escalate privileges or read secrets
- `workspace-write` sandbox: Codex can only write within the repo workspace
- API key stored as GitHub Secret, never exposed in logs

## References

- [Codex GitHub Action](https://developers.openai.com/codex/github-action/)
- [openai/codex-action](https://github.com/openai/codex-action)
