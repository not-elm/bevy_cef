//! macOS GPU OSR: OnAcceleratedPaint + IOSurface texture import.
#![cfg(target_os = "macos")]

mod iosurface;
pub use iosurface::import_iosurface_to_wgpu;

use bevy::render::render_resource::{
    Extent3d, Texture, TextureDimension, TextureFormat, TextureUsages, TextureView,
};
use bevy::render::render_resource::{TextureDescriptor, TextureViewDescriptor};
use bevy::render::renderer::RenderDevice;

/// Per-webview owned destination texture (MVP: single buffer).
/// CEF の IOSurface を import したテクスチャからここへ blit する。
/// Bevy のマテリアルはこの texture を `RenderAssets<GpuImage>` 経由でサンプルする。
///
/// `texture` and `view` are stored as bevy's wrapper types so that Task 7 can
/// build a `GpuImage { texture, texture_view, .. }` directly without conversion.
pub struct WebviewGpuSurface {
    pub texture: Texture,
    pub view: TextureView,
    pub width: u32,
    pub height: u32,
}

impl WebviewGpuSurface {
    pub fn new(device: &RenderDevice, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("webview-gpu-surface"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        Self { texture, view, width, height }
    }

    /// Returns `true` if the surface had to be (re)created for a new size.
    pub fn ensure_size(&mut self, device: &RenderDevice, width: u32, height: u32) -> bool {
        if self.width == width && self.height == height {
            return false;
        }
        *self = Self::new(device, width, height);
        true
    }

    /// Records a full-surface blit from `src` (an imported IOSurface texture) into the owned
    /// texture. `src` must be at least `width`×`height`.
    ///
    /// `src` takes a raw `&wgpu::Texture` because `import_iosurface_to_wgpu` returns
    /// `wgpu::Texture` directly.  Bevy's `Texture` also implements `Deref<Target =
    /// wgpu::Texture>`, so callers with a bevy wrapper can pass `&*bevy_tex`.
    pub fn blit_from(&self, encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture) {
        encoder.copy_texture_to_texture(
            src.as_image_copy(),
            // bevy Texture derefs to wgpu::Texture, so as_image_copy() is available.
            self.texture.as_image_copy(),
            Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
    }
}
