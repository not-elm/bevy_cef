.PHONY: fix install

BIN := bevy_cef_debug_render_process
CEF_LIB := $(HOME)/.local/share/cef/Chromium Embedded Framework.framework/Libraries


.PHONY: fix install

BIN := bevy_cef_debug_render_process
CEF_LIB := $(HOME)/.local/share/cef/Chromium Embedded Framework.framework/Libraries
CARGO_BIN := $(HOME)/.cargo/bin

fix:
	cargo clippy --fix --allow-dirty --allow-staged --workspace --all --all-features
	cargo fmt --all

install:
	cargo install --path ./crates/bevy_cef_debug_render_process --force
	mv "$(CARGO_BIN)/$(BIN)" "$(CEF_LIB)/$(BIN)"