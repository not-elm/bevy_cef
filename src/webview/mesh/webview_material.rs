use bevy::asset::*;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
#[cfg(not(target_os = "macos"))]
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
// `RenderTextureMessage` / `Browsers` from the core prelude are only used by the
// CPU `OnPaint` path (Linux/Windows). macOS uses the GPU IOSurface path.
#[cfg(not(target_os = "macos"))]
use bevy_cef_core::prelude::*;

const WEBVIEW_UTIL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("6c7cb871-4208-4407-9c25-306c6f069e2b");

pub(super) struct WebviewMaterialPlugin;

impl Plugin for WebviewMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WebviewMaterial>::default());

        // macOS uses the GPU IOSurface accelerated-paint path and never emits
        // `RenderTextureMessage`; the CPU `OnPaint` chain is Linux/Windows-only.
        #[cfg(not(target_os = "macos"))]
        app.add_message::<RenderTextureMessage>();

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        app.add_systems(Update, send_render_textures);

        #[cfg(target_os = "windows")]
        app.add_systems(Update, send_render_textures_win);

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

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn send_render_textures(mut ew: MessageWriter<RenderTextureMessage>, browsers: NonSend<Browsers>) {
    for texture in browsers.try_receive_textures() {
        ew.write(texture);
    }
}

#[cfg(target_os = "windows")]
fn send_render_textures_win(
    mut ew: MessageWriter<RenderTextureMessage>,
    texture_rx: Res<crate::common::TextureReceiverRes>,
) {
    while let Ok(texture) = texture_rx.0.try_recv() {
        ew.write(texture);
    }
}

/// Copies a CPU `OnPaint` frame into an `Image`. Used only by the CPU-path
/// consumers (Linux/Windows); macOS injects pixels directly via the GPU path.
#[cfg(not(target_os = "macos"))]
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
