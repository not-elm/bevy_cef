//! Registers a `demo://` custom scheme that streams a file from a temp dir with
//! a custom response header, and renders it in a world-space webview. Mirrors
//! `examples/inline_html.rs` plus a custom-scheme registration.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::*;
use bevy_cef::prelude::*;

/// Serves `demo://app/<path>` from a directory seeded at startup.
struct DemoHandler {
    root: PathBuf,
}

impl CefSchemeHandler for DemoHandler {
    fn handle(&self, request: &CefSchemeRequest) -> CefSchemeResponse {
        // demo://app/index.html -> <root>/index.html
        let rel = request
            .url
            .strip_prefix("demo://app/")
            .filter(|s| !s.is_empty())
            .unwrap_or("index.html");
        let path = self.root.join(rel);
        let Ok(file) = fs::File::open(&path) else {
            return CefSchemeResponse::not_found();
        };
        let len = file.metadata().ok().map(|m| m.len());
        let mime = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .essence_str()
            .to_string();
        CefSchemeResponse {
            status: 200,
            mime_type: mime,
            headers: vec![("Cache-Control".to_string(), "no-store".to_string())],
            body: CefSchemeBody::Reader {
                reader: Box::new(file),
                len,
            },
        }
    }
}

fn main() {
    #[cfg(not(target_os = "macos"))]
    bevy_cef::prelude::early_exit_if_subprocess();

    // Seed a temp dir with an index.html the demo scheme will serve.
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().to_path_buf();
    fs::write(
        root.join("index.html"),
        r#"<!DOCTYPE html><html><body style="background:#222;color:#0f0;font-family:sans-serif">
        <h1>Served via demo:// custom scheme</h1>
        <p>Streamed from a temp file with Cache-Control: no-store.</p>
        </body></html>"#,
    )
    .expect("write index.html");
    // NOTE: The TempDir guard must outlive the process; leak it intentionally
    // so the temp directory persists for the full run (CEF serves it lazily).
    std::mem::forget(dir);

    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin {
                custom_schemes: vec![CefCustomScheme {
                    name: "demo".to_string(),
                    options: CefSchemeOptions::STANDARD
                        | CefSchemeOptions::SECURE
                        | CefSchemeOptions::CORS_ENABLED
                        | CefSchemeOptions::FETCH_ENABLED
                        | CefSchemeOptions::DISPLAY_ISOLATED,
                    domain: None,
                    handler: Arc::new(DemoHandler { root }),
                }],
                ..default()
            },
        ))
        .add_systems(Startup, (spawn_camera, spawn_light, spawn_webview))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 0., 3.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_light(mut commands: Commands) {
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
        WebviewSource::new("demo://app/index.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
