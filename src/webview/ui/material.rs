//! `WebviewUiMaterial`: a `UiMaterial` that samples the webview surface texture.

use bevy::asset::{AssetPath, embedded_path};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// A `UiMaterial` that draws the webview's offscreen surface into a `bevy_ui`
/// node. `surface` is `None` until the surface-copy system inserts the texture;
/// `AsBindGroup` falls back to a default texture in the meantime.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct WebviewUiMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub surface: Option<Handle<Image>>,
}

impl UiMaterial for WebviewUiMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("webview_ui.wesl")).with_source("embedded"),
        )
    }
}
