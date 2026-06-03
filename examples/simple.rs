//! A simple example that shows how to spawn a webview in world space.

use bevy::prelude::*;
// TEMP: verification screenshot (remove later)
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy_cef::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // [poc-osr] Set a dedicated root_cache_path so cef_initialize does
            // not fall into the default-profile process-singleton path (which
            // makes initialize() return 0 / "Opening in existing browser session").
            CefPlugin {
                root_cache_path: Some("/tmp/bevy_cef_poc_cache".into()),
                ..default()
            },
        ))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        // TEMP: verification screenshot (remove later)
        .add_systems(Update, verification_screenshot)
        .run();
}

// TEMP: verification screenshot (remove later)
// Fires once ~6s after startup: captures the primary window framebuffer
// (in-app GPU readback, no macOS Screen Recording permission needed) and
// saves it to /tmp/poc-task3.png. ~3s later it sends AppExit so the app
// terminates cleanly and the async PNG readback flushes.
fn verification_screenshot(
    mut commands: Commands,
    time: Res<Time>,
    mut shot_timer: Local<Option<Timer>>,
    mut exit_timer: Local<Option<Timer>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let shot = shot_timer.get_or_insert_with(|| Timer::new(Duration::from_secs(6), TimerMode::Once));
    if !shot.is_finished() {
        shot.tick(time.delta());
        if shot.just_finished() {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk("/tmp/poc-task3.png"));
            *exit_timer = Some(Timer::new(Duration::from_secs(3), TimerMode::Once));
        }
        return;
    }

    if let Some(exit) = exit_timer.as_mut() {
        exit.tick(time.delta());
        if exit.just_finished() {
            app_exit.write(AppExit::Success);
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 0., 3.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(1., 1., 1.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::new("https://github.com/not-elm/bevy_cef"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
