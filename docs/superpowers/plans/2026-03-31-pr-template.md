# GitHub PR Template Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a GitHub PR template that structures all PRs with Problem, Solution, and Test Pattern sections.

**Architecture:** Single Markdown file at `.github/pull_request_template.md`. GitHub automatically loads this as the default body for new PRs.

**Tech Stack:** GitHub PR templates (Markdown)

---

### Task 1: Create PR template

**Files:**
- Create: `.github/pull_request_template.md`

- [ ] **Step 1: Create the template file**

```markdown
## Problem
<!-- What issue does this PR address? Why is the change needed? -->

## Solution
<!-- How does this PR solve the problem? Key design decisions? -->

## Test Pattern
<!-- How did you verify the change? (e.g., cargo run --example simple --features debug) -->
```

- [ ] **Step 2: Verify the file is picked up by GitHub**

Run: `cat .github/pull_request_template.md`
Expected: The three-section template with HTML comment guides is displayed.

- [ ] **Step 3: Commit**

```bash
git add .github/pull_request_template.md
git commit -m "chore: add GitHub PR template with Problem/Solution/Test Pattern sections"
```
