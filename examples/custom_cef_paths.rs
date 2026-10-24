//! Temporary smoke test for custom CEF framework and subprocess paths.
//!
//! macOS setup (build the subprocess from this PR, rather than an installed copy):
//! ```sh
//! cargo build -p bevy_cef_render_process
//! ```
//! Test environment-variable configuration (CEF_PATH can replace CEF_FRAMEWORK_DIR):
//! ```sh
//! CEF_FRAMEWORK_DIR="$HOME/.local/share/cef" \
//! CEF_SUBPROCESS_PATH="$PWD/target/debug/bevy_cef_render_process" \
//! cargo run --example custom_cef_paths --features debug
//! ```
//! Test CefPlugin fields with two positional arguments:
//! ```sh
//! CEF_FRAMEWORK_DIR="$HOME/.local/share/cef" \
//! cargo run --example custom_cef_paths --features debug -- \
//!   "$HOME/.local/share/cef" "$PWD/target/debug/bevy_cef_render_process"
//! ```
//! On macOS the CEF argument may be the distribution directory, the .framework
//! directory, or the framework binary. Use absolute paths. The separate render
//! process reads its framework path from the environment, so keep
//! CEF_FRAMEWORK_DIR set even when testing the plugin fields. This example does
//! not set it automatically, so path propagation problems remain observable.
//!
//! Success: the page's counter advances and the console prints `IPC OK` every
//! second. Click the button to check pointer input. A window alone or
//! CefStatus::Running does not prove that the render subprocess is working.

use bevy::prelude::*;
use bevy_cef::prelude::*;
use serde::Deserialize;
use std::path::PathBuf;

fn main() {
    #[cfg(not(target_os = "macos"))]
    early_exit_if_subprocess();

    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (cef_path, subprocess_path) = match args.as_slice() {
        [] => (None, None),
        [cef_path, subprocess_path] => (
            Some(PathBuf::from(cef_path)),
            Some(PathBuf::from(subprocess_path)),
        ),
        _ => {
            eprintln!("Usage: custom_cef_paths [CEF_PATH SUBPROCESS_PATH]");
            std::process::exit(2);
        }
    };
    eprintln!("CefPlugin.cef_path: {cef_path:?}");
    eprintln!("CefPlugin.subprocess_path: {subprocess_path:?}");
    for name in ["CEF_FRAMEWORK_DIR", "CEF_PATH", "CEF_SUBPROCESS_PATH"] {
        eprintln!("{name}: {:?}", std::env::var_os(name));
    }

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Custom CEF paths — smoke test".into(),
                    ..default()
                }),
                ..default()
            }),
            CefPlugin {
                cef_path,
                subprocess_path,
                root_cache_path: Some(
                    std::env::temp_dir()
                        .join(format!("bevy_cef_custom_paths_{}", std::process::id()))
                        .to_string_lossy()
                        .into_owned(),
                ),
                ..default()
            },
            JsEmitEventPlugin::<PageReport>::default(),
        ))
        .init_resource::<ReceivedReport>()
        .add_systems(Startup, setup)
        .add_systems(Update, check_renderer)
        .add_observer(receive_report)
        .add_observer(|event: On<LoadError>| {
            error!("Page load failed: {} ({})", event.url, event.error_code);
        })
        .run();
}

#[derive(Deserialize)]
struct PageReport {
    seconds: u32,
    clicks: u32,
}

#[derive(Resource, Default)]
struct ReceivedReport(bool);

fn receive_report(event: On<Receive<PageReport>>, mut received: ResMut<ReceivedReport>) {
    received.0 = true;
    info!("IPC OK: seconds={}, clicks={}", event.seconds, event.clicks);
}

fn setup(
    mut commands: Commands,
    status: Res<CefStatus>,
    mut exit: MessageWriter<AppExit>,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
) {
    match &*status {
        CefStatus::Running => info!("CEF initialized; waiting for renderer IPC..."),
        CefStatus::Failed(error) => {
            error!("CEF startup failed: {error}");
            exit.write(AppExit::error());
            return;
        }
    }
    commands.spawn(Camera2d);
    commands.spawn((
        WebviewSource::inline(PAGE),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        MaterialNode(materials.add(WebviewUiMaterial::default())),
    ));
}

fn check_renderer(time: Res<Time<Real>>, received: Res<ReceivedReport>, mut warned: Local<bool>) {
    if !received.0 && !*warned && time.elapsed_secs() > 15.0 {
        *warned = true;
        error!("No renderer IPC after 15 seconds. Check framework/subprocess paths and CEF logs.");
    }
}

const PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<meta charset="UTF-8">
<title>Custom CEF paths</title>
<style>
  body { margin: 48px; background: #18212f; color: #e8eef6; font: 20px system-ui; }
  button { padding: 12px 24px; font: inherit; cursor: pointer; }
  #status { color: #7ee3ad; }
</style>
<h1>Custom CEF paths</h1>
<p id="status">HTML rendered. Waiting for JavaScript...</p>
<p>Check the Rust console for <b>IPC OK</b> to verify delivery.</p>
<button id="button">Test pointer input</button>
<p id="clicks">Clicks: 0</p>
<script>
  let seconds = 0;
  let clicks = 0;
  document.getElementById('button').onclick = () => {
    document.getElementById('clicks').textContent = `Clicks: ${++clicks}`;
  };
  setInterval(() => {
    document.getElementById('status').textContent = `JavaScript OK: ${++seconds}s`;
    try {
      window.cef.emit({ seconds, clicks });
    } catch (error) {
      document.getElementById('status').textContent = `IPC failed: ${error}`;
    }
  }, 1000);
</script>
</html>"#;
