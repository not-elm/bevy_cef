# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Public `bevy_cef` crate API surface.
- `crates/bevy_cef_core`: Core CEF integration and IPC.
- `crates/bevy_cef_debug_render_process`: Debug render-process tool.
- `examples/`: Runnable samples (e.g., `simple.rs`, `js_emit.rs`, `brp.rs`).
- `assets/`: Local HTML/CSS/JS served via `cef://localhost/`.
- `docs/`: Supporting documentation.

## Build, Test, and Development Commands
- `cargo build --features debug`: Build with macOS debug tooling enabled.
- `cargo run --example simple --features debug`: Run a basic webview example.
- `cargo test --workspace --all-features`: Run tests (currently no automated tests; validates compilation).
- `make fix`: Run `cargo clippy --fix` and `cargo fmt --all`.
- `make install`: Install and copy the debug render process into the CEF framework (macOS).

## Coding Style & Naming Conventions
- Rust 2024 edition; format with `cargo fmt` before committing.
- Lint with `cargo clippy` (see `make fix`).
- Naming: `snake_case` for modules/functions/files, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Prefer small, focused modules; keep public API in `src/` and implementation in `crates/`.

## Testing Guidelines
- No dedicated test suite yet; rely on examples for manual verification.
- Use `examples/` to validate features (IPC, navigation, zoom, devtools).
- If adding tests, keep names descriptive (e.g., `test_ipc_roundtrip`) and run with `cargo test --workspace --all-features`.

## Commit & Pull Request Guidelines
- Commit subjects are short, imperative; common prefixes include `add:`, `fix:`, `update:`, `remove:`.
- Include PR or issue references when relevant (e.g., `Support Bevy 0.17 (#11)`).
- PRs should describe changes, testing performed, and target platform (macOS/Windows/Linux).
- For webview or UI changes, include screenshots or short clips when possible.

## Platform & Configuration Notes
- Primary development target is macOS; CEF framework should exist at `$HOME/.local/share/cef`.
- Local assets are served as `cef://localhost/<file>`; prefer `CefWebviewUri::local("file.html")`.
