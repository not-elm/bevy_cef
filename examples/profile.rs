//! Keeps a webview's login state across restarts with a named, disk-backed profile.
//!
//! Run `cargo run --example profile -- work`, log in to GitHub, close the window, and run the
//! same command again: the page is still logged in. A different name (`-- other`) gets its own
//! profile, which is logged out. Without an argument the profile is named `default`.
//!
//! The profiles live in `bevy_cef_profile_example` under the OS temp directory, one
//! `profile-NAME` directory per name. Only one process can use that root at a time.

use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    #[cfg(not(target_os = "macos"))]
    bevy_cef::prelude::early_exit_if_subprocess();

    let name = std::env::args().nth(1).unwrap_or_else(|| "default".into());
    let profile = match WebviewProfile::new(name) {
        Ok(profile) => profile,
        Err(error) => {
            eprintln!("invalid profile name: {error}");
            std::process::exit(2);
        }
    };
    let root = std::env::temp_dir().join("bevy_cef_profile_example");

    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin {
                root_cache_path: Some(root.to_string_lossy().into_owned()),
                ..default()
            },
        ))
        .insert_resource(ExampleProfile(profile))
        .add_observer(log_profile_error)
        .add_observer(log_load_finished)
        .add_systems(Startup, (spawn_camera, spawn_webview))
        .run();
}

#[derive(Resource)]
struct ExampleProfile(WebviewProfile);

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_webview(
    mut commands: Commands,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
    profile: Res<ExampleProfile>,
) {
    info!("Using profile {:?}", profile.0.name().as_str());
    commands.spawn((
        WebviewSource::new("https://github.com/login"),
        profile.0.clone(),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        MaterialNode(materials.add(WebviewUiMaterial::default())),
    ));
}

fn log_profile_error(error: On<WebviewProfileError>) {
    error!(
        "WebviewProfileError on {}: {:?}",
        error.webview, error.reason
    );
}

fn log_load_finished(finished: On<LoadFinished>) {
    info!(
        "LoadFinished on {} with HTTP status {}",
        finished.webview, finished.http_status_code
    );
}
