.PHONY: fix install setup-windows

BIN := bevy_cef_debug_render_process
CEF_LIB := $(HOME)/.local/share/Chromium Embedded Framework.framework/Libraries
CARGO_BIN := $(HOME)/.cargo/bin

fix-lint:
	cargo clippy --fix --allow-dirty --allow-staged --workspace --all --all-features
	cargo fmt --all

install-debug-render-process:
	cargo install --path ./crates/bevy_cef_debug_render_process --force
	mv "$(CARGO_BIN)/$(BIN)" "$(CEF_LIB)/$(BIN)"

setup-windows:
	cargo install export-cef-dir --force
	export-cef-dir --force "$(USERPROFILE)/.local/share/cef"
