# Publish & Release Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a GitHub Actions workflow that publishes all workspace crates to crates.io and creates a GitHub Release with auto-generated notes on tag push.

**Architecture:** Single workflow file triggered by `v*` tag push. One job publishes crates sequentially in dependency order, then creates a GitHub Release using the existing `.github/release.yml` template.

**Tech Stack:** GitHub Actions, `cargo publish`, `gh` CLI

---

### Task 1: Create the publish workflow file

**Files:**
- Create: `.github/workflows/publish.yml`

**Step 1: Write the workflow file**

```yaml
name: Publish

on:
  push:
    tags:
      - "v*"

jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: stable

      - name: Install alsa and udev
        run: sudo apt-get update; sudo apt-get install --no-install-recommends libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev pkg-config

      - name: Publish bevy_cef_core
        run: cargo publish -p bevy_cef_core
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Publish bevy_cef_bundle_app
        run: cargo publish -p bevy_cef_bundle_app
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Wait for crates.io index
        run: sleep 30

      - name: Publish bevy_cef
        run: cargo publish -p bevy_cef
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Publish bevy_cef_render_process
        run: cargo publish -p bevy_cef_render_process
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Publish bevy_cef_debug_render_process
        run: cargo publish -p bevy_cef_debug_render_process
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Create GitHub Release
        env:
          GH_TOKEN: ${{ github.token }}
        run: gh release create ${{ github.ref_name }} --generate-notes
```

**Step 2: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/publish.yml'))"`
Expected: No output (valid YAML)

**Step 3: Commit**

```bash
git add .github/workflows/publish.yml docs/plans/2026-03-04-publish-release-workflow-design.md docs/plans/2026-03-04-publish-release-workflow.md
git commit -m "feat: add publish and release workflow"
```

---

### Task 2: Push and create PR

**Step 1: Push the branch**

```bash
git push -u origin publish-release-workflow
```

**Step 2: Create the PR**

```bash
gh pr create --title "Add publish and release workflow" --body "$(cat <<'EOF'
## Summary
- Adds `.github/workflows/publish.yml` triggered on `v*` tag push
- Publishes all 5 workspace crates to crates.io in dependency order
- Creates a GitHub Release with auto-generated notes from `.github/release.yml`

## Crate publish order
1. `bevy_cef_core` (no internal deps)
2. `bevy_cef_bundle_app` (no internal deps)
3. 30s sleep for crates.io index propagation
4. `bevy_cef` (depends on core)
5. `bevy_cef_render_process` (depends on core)
6. `bevy_cef_debug_render_process` (depends on core)

## Required setup
Add `CARGO_REGISTRY_TOKEN` secret to the repository settings with your crates.io API token.

## Test plan
- [ ] Verify YAML is valid
- [ ] After merge, push a tag `v0.x.0` to test the full workflow

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

**Step 3: Verify PR was created**

Run: `gh pr view --web`
Expected: PR opens in browser
