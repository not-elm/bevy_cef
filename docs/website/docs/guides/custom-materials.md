---
sidebar_position: 5
---

# Custom Materials

bevy_cef supports custom materials through Bevy's `MaterialExtension` system, letting you apply custom shaders to your webview textures. This is useful for effects like masks, distortions, or any post-processing on the rendered web content.

## Setting Up the Plugin

Register `WebviewExtendMaterialPlugin` with your custom extension type:

```rust
use bevy::prelude::*;
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
```

## Defining a Material Extension

Create a struct that implements `MaterialExtension`, `AsBindGroup`, `Asset`, and `Reflect`:

```rust
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
```

## Writing the Shader

Place your WGSL shader in the `assets/shaders/` directory. The webview texture is available through the base material bindings, and your custom bindings (like the mask texture above) are accessible at the binding indices you specified:

```wgsl
#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var mask_texture: texture_2d<f32>;
@group(2) @binding(1) var mask_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let mask = textureSample(mask_texture, mask_sampler, in.uv);
    // Apply mask alpha to the final output
    return vec4<f32>(in.color.rgb, mask.a);
}
```

## Spawning with the Custom Material

Use `WebviewExtendedMaterial<CustomExtension>` as the material type:

```rust
fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendedMaterial<CustomExtension>>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        WebviewSource::new("https://example.com"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendedMaterial {
            extension: CustomExtension {
                mask: asset_server.load("textures/mask.png"),
            },
            ..default()
        })),
    ));
}
```

:::note

`WebviewExtendStandardMaterial` (used in the getting-started guide) is actually `WebviewExtendedMaterial<()>` -- the default case with no custom extension.

:::
