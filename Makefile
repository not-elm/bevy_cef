fix:
	cargo clippy --fix --allow-dirty --allow-staged --workspace --all --all-features
	cargo fmt --all

install:
	cargo install --path ./crates/bevy_cef_debug_render_process --force
	mv $HOME/.cargo/bin/bevy_cef_debug_render_process "$HOME/.local/share/cef/Chromium Embedded Framework.framework/Libraries/bevy_cef_debug_render_process"