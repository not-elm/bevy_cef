//! `WebviewUiMaterial`: a `UiMaterial` that samples the webview surface texture.

use bevy::asset::uuid_handle;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub(crate) const WEBVIEW_UI_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3f2b6e8a-0c1d-4e7a-9b2c-7a1e5d4c8f02");

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
        WEBVIEW_UI_SHADER_HANDLE.into()
    }
}
