use bevy::asset::{RenderAssetUsages, load_internal_asset, weak_handle};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy_cef_core::prelude::*;

const WEBVIEW_UTIL_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("6c7cb871-4208-4407-9c25-306c6f069e2b");

pub(super) struct WebviewMaterialPlugin;

impl Plugin for WebviewMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WebviewMaterial>::default())
            .add_event::<RenderTexture>()
            .add_systems(Update, send_render_textures);
        load_internal_asset!(
            app,
            WEBVIEW_UTIL_SHADER_HANDLE,
            "./webview_util.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Asset, Reflect, Default, Debug, Clone, AsBindGroup)]
pub struct WebviewMaterial {
    /// Holds the texture handle for the webview.
    ///
    /// This texture is automatically updated.
    #[texture(101)]
    #[sampler(102)]
    pub surface: Option<Handle<Image>>,
}

impl Material for WebviewMaterial {}

fn send_render_textures(mut ew: EventWriter<RenderTexture>, browsers: NonSend<Browsers>) {
    while let Ok(texture) = browsers.try_receive_texture() {
        ew.write(texture);
    }
}

pub(crate) fn update_webview_image(texture: RenderTexture, image: &mut Image) {
    *image = Image::new(
        Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture.buffer,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::all(),
    );
}
