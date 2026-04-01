use bevy::asset::*;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy_cef_core::prelude::*;

use crate::diagnostics::CefTextureDiagnostics;

const WEBVIEW_UTIL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("6c7cb871-4208-4407-9c25-306c6f069e2b");

/// System set for the texture sending system. Used for ordering diagnostics collection.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SendRenderTexturesSet;

pub(super) struct WebviewMaterialPlugin;

impl Plugin for WebviewMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WebviewMaterial>::default())
            .add_message::<RenderTextureMessage>()
            .add_systems(Update, send_render_textures.in_set(SendRenderTexturesSet));
        load_internal_asset!(
            app,
            WEBVIEW_UTIL_SHADER_HANDLE,
            "./webview_util.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct WebviewMaterial {
    /// Holds the texture handle for the webview.
    ///
    /// This texture is automatically updated.
    #[texture(101)]
    #[sampler(102)]
    pub surface: Option<Handle<Image>>,
}

impl Material for WebviewMaterial {}

fn send_render_textures(
    mut ew: MessageWriter<RenderTextureMessage>,
    browsers: NonSend<Browsers>,
    mut diagnostics: Option<ResMut<CefTextureDiagnostics>>,
) {
    if let Some(ref mut diag) = diagnostics {
        diag.total_buffer_bytes = 0;
    }
    while let Ok(texture) = browsers.try_receive_texture() {
        if let Some(ref mut diag) = diagnostics {
            diag.last_transfer_time = Some(texture.created_at.elapsed());
            diag.total_buffer_bytes += texture.buffer.len() as u64;
        }
        ew.write(texture);
    }
}

pub(crate) fn update_webview_image(texture: &RenderTextureMessage, image: &mut Image) {
    let expected_len = (texture.width * texture.height * 4) as usize;
    if let Some(data) = image.data.as_mut()
        && data.len() == expected_len
        && image.texture_descriptor.size.width == texture.width
        && image.texture_descriptor.size.height == texture.height
    {
        data.copy_from_slice(&texture.buffer);
    } else {
        *image = Image::new(
            Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture.buffer.clone(),
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::all(),
        );
    }
}
