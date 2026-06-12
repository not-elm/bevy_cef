//! Headless webview: render a page with NO bevy_cef display component and
//! sample its texture from a third-party `UiMaterial` — the pattern a terminal
//! emulator uses to composite an inline webview in its own shader.
//!
//! macOS only: the headless texture path rides the GPU IOSurface pipeline.
//! Run with: `cargo run --example headless_texture --features debug`

use bevy::asset::AssetId;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin::default(),
            UiMaterialPlugin::<TintedWebviewMaterial>::default(),
            WebviewTargetUiMaterialPlugin::<TintedWebviewMaterial>::default(),
        ))
        .add_systems(Startup, (spawn_camera, spawn_headless_webview))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_headless_webview(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TintedWebviewMaterial>>,
) {
    // The user owns the handle from frame 0; bevy_cef manages the contents.
    let target = images.add(Image::default());

    commands.spawn((
        WebviewSource::new("https://github.com/not-elm/bevy_cef"),
        WebviewTextureTarget(target.clone()),
    ));

    // A full-screen node drawn by OUR material — bevy_cef knows nothing about
    // this entity.
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        MaterialNode(materials.add(TintedWebviewMaterial {
            webview: Some(target),
        })),
    ));
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
struct TintedWebviewMaterial {
    #[texture(0)]
    #[sampler(1)]
    webview: Option<Handle<Image>>,
}

impl UiMaterial for TintedWebviewMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/headless_texture.wgsl".into()
    }
}

impl WebviewTextureSlot for TintedWebviewMaterial {
    fn webview_targets(&self) -> impl Iterator<Item = AssetId<Image>> {
        self.webview.iter().map(Handle::id)
    }
}
