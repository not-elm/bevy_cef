# Codex PR Review Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a GitHub Actions workflow that uses OpenAI Codex to automatically review every pull request for code quality.

**Architecture:** Two-job pipeline — Job 1 runs `openai/codex-action@v1` to generate review feedback, Job 2 posts feedback as a GitHub PR Review (COMMENT event). A separate prompt file instructs Codex to read the project's CLAUDE.md for context.

**Tech Stack:** GitHub Actions, `openai/codex-action@v1`, `actions/github-script@v7`

---

### Task 1: Create the review prompt file

**Files:**
- Create: `.github/codex/prompts/review.md`

**Step 1: Create the directory structure**

Run: `mkdir -p .github/codex/prompts`

**Step 2: Write the prompt file**

Create `.github/codex/prompts/review.md` with:

```markdown
# Code Review Instructions

You are reviewing a pull request for **bevy_cef**, a Bevy plugin integrating the Chromium Embedded Framework (CEF).

## Context

First, read `CLAUDE.md` at the repository root. It describes:
- Project architecture (multi-process design, browser/render processes)
- Core components and plugin structure
- IPC system patterns (JsEmit, HostEmit, BRP)
- Platform notes (macOS primary, NonSend resources)

## Review Checklist

Review the PR diff for:

1. **Bugs & Logic Errors** — incorrect logic, off-by-one, null/None/unwrap misuse
2. **Performance** — unnecessary allocations, cloning, inefficient patterns
3. **Security** — unsafe usage, input validation, potential vulnerabilities
4. **Style & Idioms** — Rust idioms, naming conventions, code clarity
5. **API Design** — public API consistency, breaking changes

## Guidelines

- Focus only on the changed lines (the diff)
- Be concise — one sentence per issue
- Flag only actionable issues with specific suggestions
- If the changes look good, say so briefly
- Do NOT comment on formatting (rustfmt handles that)
- Do NOT comment on missing tests unless the change is clearly risky
```

**Step 3: Commit**

```bash
git add .github/codex/prompts/review.md
git commit -m "ci: add Codex PR review prompt"
```

---

### Task 2: Create the GitHub Actions workflow

**Files:**
- Create: `.github/workflows/codex-pr-review.yml`

**Step 1: Write the workflow file**

Create `.github/workflows/codex-pr-review.yml` with:

```yaml
name: Codex PR Review

on:
  pull_request:
    types: [opened, synchronize, reopened]

jobs:
  codex:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
    outputs:
      final_message: ${{ steps.run_codex.outputs.final-message }}
    steps:
      - uses: actions/checkout@v5
        with:
          ref: refs/pull/${{ github.event.pull_request.number }}/merge

      - name: Pre-fetch base and head refs
        run: |
          git fetch --no-tags origin \
            ${{ github.event.pull_request.base.ref }} \
            +refs/pull/${{ github.event.pull_request.number }}/head

      - name: Run Codex
        id: run_codex
        uses: openai/codex-action@v1
        with:
          openai-api-key: ${{ secrets.OPENAI_API_KEY }}
          prompt-file: .github/codex/prompts/review.md
          output-file: codex-output.md
          safety-strategy: drop-sudo
          sandbox: workspace-write

  post_review:
    runs-on: ubuntu-latest
    needs: codex
    if: needs.codex.outputs.final_message != ''
    permissions:
      pull-requests: write
    steps:
      - name: Post Codex review
        uses: actions/github-script@v7
        with:
          github-token: ${{ github.token }}
          script: |
            await github.rest.pulls.createReview({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.payload.pull_request.number,
              body: process.env.CODEX_FINAL_MESSAGE,
              event: 'COMMENT',
            });
        env:
          CODEX_FINAL_MESSAGE: ${{ needs.codex.outputs.final_message }}

concurrency:
  group: codex-review-${{ github.event.pull_request.number }}
  cancel-in-progress: true
```

**Step 2: Commit**

```bash
git add .github/workflows/codex-pr-review.yml
git commit -m "ci: add Codex PR review workflow"
```

---

### Task 3: Verify the workflow syntax

**Step 1: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/codex-pr-review.yml'))"`
Expected: No output (valid YAML)

If `yaml` module not available, run: `ruby -ryaml -e "YAML.load_file('.github/workflows/codex-pr-review.yml')"`

**Step 2: Check with actionlint if available**

Run: `which actionlint && actionlint .github/workflows/codex-pr-review.yml || echo "actionlint not installed, skipping"`

---

### Task 4: Final commit and summary

**Step 1: Verify all files are committed**

Run: `git status`
Expected: Clean working tree (no untracked workflow files)

**Step 2: Review commit log**

Run: `git log --oneline -3`
Expected: Two new commits (prompt file + workflow file)

---

## Prerequisites (manual, not automated)

Before this workflow will function in CI:

1. Go to repository Settings > Secrets and variables > Actions
2. Add a new repository secret: `OPENAI_API_KEY` with your OpenAI API key
3. Ensure the API key has access to the Codex model
