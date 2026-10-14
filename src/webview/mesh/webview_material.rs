#[cfg(not(target_os = "macos"))]
use crate::prelude::WebviewExtendStandardMaterial;
#[cfg(not(target_os = "macos"))]
use bevy::asset::{AssetMut, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
#[cfg(not(target_os = "macos"))]
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
#[cfg(not(target_os = "macos"))]
use bevy::render::{
    Render, RenderApp, RenderSystems, erased_render_asset::prepare_erased_assets,
    render_asset::prepare_assets, texture::GpuImage,
};
// `RenderTextureMessage` / `Browsers` from the core prelude are only used by the
// CPU `OnPaint` path (Linux/Windows). macOS uses the GPU IOSurface path.
#[cfg(not(target_os = "macos"))]
use bevy_cef_core::prelude::*;

pub(super) struct WebviewMaterialPlugin;

impl Plugin for WebviewMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WebviewMaterial>::default());

        // macOS uses the GPU IOSurface accelerated-paint path and never emits
        // `RenderTextureMessage`; the CPU `OnPaint` chain is Linux/Windows-only.
        #[cfg(not(target_os = "macos"))]
        app.add_message::<RenderTextureMessage>()
            .add_systems(Update, send_render_textures);

        // NOTE: Sprite and mesh webview material preparation must run after `GpuImage`
        // preparation, but Bevy declares no such dependency, so this empty system anchors the
        // order. Without it, a material touched in the frame its image is reallocated can
        // capture the old texture view, and a static page never repaints to repair it.
        #[cfg(not(target_os = "macos"))]
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Render,
                order_webview_materials_after_images
                    .in_set(RenderSystems::PrepareAssets)
                    .after(prepare_assets::<GpuImage>)
                    .before(prepare_erased_assets::<MeshMaterial2d<SpriteMeshMaterial>>)
                    .before(prepare_erased_assets::<MeshMaterial3d<WebviewExtendStandardMaterial>>),
            );
        }
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

/// Copies a CPU `OnPaint` frame into an `Image`, reallocating the image when the frame size
/// differs from it. Returns `true` when the image was reallocated, which replaces its GPU
/// texture and makes any material bound to the old texture stale.
#[cfg(not(target_os = "macos"))]
pub(crate) fn update_webview_image(
    texture: &RenderTextureMessage,
    mut image: AssetMut<'_, Image>,
) -> bool {
    let expected_len = (texture.width * texture.height * 4) as usize;
    let dimensions_match = image.texture_descriptor.size.width == texture.width
        && image.texture_descriptor.size.height == texture.height;
    if let Some(data) = image.data.as_mut()
        && dimensions_match
        && data.len() == expected_len
    {
        data.copy_from_slice(&texture.buffer);
        false
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
        true
    }
}

/// Anchors webview material preparation after `GpuImage` preparation: order it
/// `.after(prepare_assets::<GpuImage>)` and `.before(...)` each material's `prepare_erased_assets`.
#[cfg(not(target_os = "macos"))]
pub(super) fn order_webview_materials_after_images() {}

#[cfg(not(target_os = "macos"))]
fn send_render_textures(mut ew: MessageWriter<RenderTextureMessage>, browsers: NonSend<Browsers>) {
    for texture in browsers.try_receive_textures() {
        ew.write(texture);
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;

    fn paint(width: u32, height: u32) -> RenderTextureMessage {
        RenderTextureMessage {
            webview: World::new().spawn_empty().id(),
            ty: RenderPaintElementType::View,
            width,
            height,
            buffer: vec![0; (width * height * 4) as usize],
        }
    }

    /// Asserts that a paint whose size differs from the image's reports a reallocation.
    ///
    /// Case: the first CEF paint replaces the 1×1 `Image::default()` a sprite webview starts
    /// with, or the user resizes the webview.
    #[test]
    fn resized_paint_reports_reallocation() {
        let mut images = Assets::<Image>::default();
        let handle = images.add(Image::default());
        let image = images.get_mut(handle.id()).expect("image was just added");
        assert!(update_webview_image(&paint(4, 2), image));
    }

    /// Asserts that a paint matching the image's size is copied in place without a
    /// reallocation.
    ///
    /// Case: CEF repaints an animated page at an unchanged webview size.
    #[test]
    fn same_size_paint_reuses_image() {
        let mut images = Assets::<Image>::default();
        let handle = images.add(Image::default());
        let first = images.get_mut(handle.id()).expect("image was just added");
        update_webview_image(&paint(4, 2), first);
        let second = images.get_mut(handle.id()).expect("image still exists");
        assert!(!update_webview_image(&paint(4, 2), second));
    }
}
