//! Registers a `demo://` custom scheme that serves a file from an in-memory
//! virtual file system (VFS) with a custom response header, and renders it in a
//! world-space webview. Mirrors `examples/inline_html.rs` plus a custom-scheme
//! registration.

use bevy::prelude::*;
use bevy_cef::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
    <body style="background:#222;color:#0f0;font-family:sans-serif">
        <h1>Served via demo:// custom scheme</h1>
        <p>Served from an in-memory VFS with Cache-Control: no-store.</p>
    </body>
</html>"#;

fn main() {
    let mut vfs = Vfs::new();
    vfs.insert("index.html", "text/html", INDEX_HTML);

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
                    handler: Arc::new(DemoHandler { vfs }),
                }],
                ..default()
            },
        ))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        .run();
}

/// Tiny in-memory virtual file system: maps request paths to `(MIME, bytes)`.
#[derive(Default)]
struct Vfs(HashMap<String, (String, Vec<u8>)>);

impl Vfs {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, path: &str, mime: &str, bytes: impl Into<Vec<u8>>) {
        self.0
            .insert(path.to_string(), (mime.to_string(), bytes.into()));
    }

    fn get(&self, path: &str) -> Option<&(String, Vec<u8>)> {
        self.0.get(path)
    }
}

/// Serves `demo://app/<path>` from an in-memory [`Vfs`] seeded at startup.
/// Lookups are exact `HashMap` keys (no disk path join), so there is no
/// path-traversal concern.
struct DemoHandler {
    vfs: Vfs,
}

impl CefSchemeHandler for DemoHandler {
    fn handle(&self, request: &CefSchemeRequest) -> CefSchemeResponse {
        let rel = request
            .url
            .strip_prefix("demo://app/")
            .filter(|s| !s.is_empty())
            .unwrap_or("index.html");
        let Some((mime, bytes)) = self.vfs.get(rel) else {
            return CefSchemeResponse::not_found();
        };
        CefSchemeResponse {
            status: 200,
            mime_type: mime.clone(),
            headers: vec![("Cache-Control".to_string(), "no-store".to_string())],
            body: CefSchemeBody::Bytes(bytes.clone()),
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
        WebviewSource::new("demo://app/index.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
