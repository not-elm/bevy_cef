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
