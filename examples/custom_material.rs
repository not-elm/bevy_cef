//!  You can create a custom material based on [`WebviewMaterial`].
//!
//! This example creates a custom material that blends an image.

use bevy::pbr::MaterialExtension;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin::default(),
            WebviewExtendMaterialPlugin::<CustomExtension>::default(),
        ))
        .add_systems(Startup, (spawn_camera, spawn_webview))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 0., 3.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendedMaterial<CustomExtension>>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        CefWebviewUri::new("https://github.com/not-elm/bevy_cef"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendedMaterial {
            extension: CustomExtension {
                mask: asset_server.load("images/rustacean-flat-gesture.png"),
            },
            ..default()
        })),
    ));
}

#[derive(Asset, Reflect, Clone, Debug, AsBindGroup, Default)]
struct CustomExtension {
    #[texture(0)]
    #[sampler(1)]
    mask: Handle<Image>,
}

impl MaterialExtension for CustomExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}
