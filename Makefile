.PHONY: fix-lint install-debug-render-process setup-windows

BIN := bevy_cef_debug_render_process
CEF_LIB := $(HOME)/.local/share/Chromium Embedded Framework.framework/Libraries
CARGO_BIN := $(HOME)/.cargo/bin

fix-lint:
	cargo clippy --fix --allow-dirty --allow-staged --workspace --all --all-features
	cargo fmt --all

install-debug-render-process:
	cargo install --path ./crates/bevy_cef_debug_render_process --force
	cp "$(CARGO_BIN)/$(BIN)" "$(CEF_LIB)/$(BIN)"

setup-macos:
	brew install cmake ninja
	cargo install export-cef-dir
	export-cef-dir --force "$(HOME)/.local/share-dir@145.6.1+145.0.28"
	cargo install bevy_cef_debug_render_process
	cp "$(HOME)/.cargo/bin/bevy_cef_debug_render_process" "$(HOME)/.local/share/Chromium Embedded Framework.framework/Libraries/bevy_cef_debug_render_process"

setup-windows:
	cargo install export-cef-dir@145.6.1+145.0.28 --force
	export-cef-dir --force "$(USERPROFILE)/.local/share/cef"
	cargo install --path ./crates/bevy_cef_render_process --root "$(USERPROFILE)/.local/share/cef" --force
