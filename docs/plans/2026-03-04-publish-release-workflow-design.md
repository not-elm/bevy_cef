# Publish & Release Workflow Design

## Overview

A GitHub Actions workflow that publishes all workspace crates to crates.io and creates a GitHub Release with auto-generated release notes when a version tag is pushed.

## Trigger

Push of tags matching `v*` (e.g., `v0.3.0`).

## Workflow Structure

**File:** `.github/workflows/publish.yml`
**Runner:** `ubuntu-latest`
**Required secret:** `CARGO_REGISTRY_TOKEN` (crates.io API token)
**Permissions:** `contents: write` (for GitHub Release creation)

Single job with sequential steps.

## Publish Order

Crates are published in dependency-safe order:

1. `bevy_cef_core` — no internal dependencies
2. `bevy_cef_bundle_app` — no internal dependencies
3. (30s sleep for crates.io index propagation)
4. `bevy_cef` — depends on `bevy_cef_core`
5. `bevy_cef_render_process` — depends on `bevy_cef_core`
6. `bevy_cef_debug_render_process` — depends on `bevy_cef_core`

Each crate is published with `cargo publish -p <crate>` using the `CARGO_REGISTRY_TOKEN` environment variable.

## GitHub Release

After all crates are published, a GitHub Release is created on the tag using `gh release create --generate-notes`. Release notes are auto-generated from PR labels using the existing `.github/release.yml` template with these categories:

- Breaking Changes
- Exciting New Features
- Improvements
- Bug Fixes
- Other Changes

## Delivery

1. Write the workflow file to `.github/workflows/publish.yml`
2. Commit and push to `publish-release-workflow` branch
3. Create PR to `main` (triggers existing Codex PR review)
