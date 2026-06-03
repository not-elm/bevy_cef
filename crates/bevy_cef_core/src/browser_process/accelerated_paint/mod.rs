//! macOS GPU OSR: OnAcceleratedPaint + IOSurface texture import.
#![cfg(target_os = "macos")]

mod iosurface;
pub use iosurface::import_iosurface_to_wgpu;

use std::os::raw::c_void;

use bevy::render::render_resource::{
    Extent3d, Texture, TextureDimension, TextureFormat, TextureUsages, TextureView,
};
use bevy::render::render_resource::{TextureDescriptor, TextureViewDescriptor};
use bevy::render::renderer::RenderDevice;

/// Per-webview "latest IOSurface" slot (Approach 2).
///
/// `on_accelerated_paint` runs on the CEF UI thread (= Bevy main thread under
/// `external_message_pump`), so a non-atomic `Rc<RefCell<_>>` is sufficient. The
/// callback retains the freshly delivered IOSurface and stores it here, releasing
/// the previously stored one (latest-frame-wins, 1-deep release-previous). The
/// main-world collect system drains this for the render world.
pub type SharedRetainedIoSurface = std::rc::Rc<std::cell::RefCell<Option<RetainedIoSurface>>>;

/// A CEF IOSurface that this code has retained to keep its **object** alive
/// beyond the `on_accelerated_paint` callback.
///
/// CEF guarantees the IOSurface pointer is valid only for the duration of the
/// callback, and empirically the surface is torn down the instant the callback
/// returns. `IOSurfaceIncrementUseCount` is only an advisory purgeability hint
/// and does **not** keep the object alive (verified: `IOSurfaceGetUseCount`
/// aborts on the retained pointer after the callback returns). To keep it alive
/// until the render-graph node imports + blits it, we hold a real CoreFoundation
/// reference via [`CFRetained`] (`CFRetain` on construction, `CFRelease` on drop).
///
/// # Safety / lifetime
/// `surface` owns a +1 CF reference to the IOSurface, so the object stays alive
/// for as long as this wrapper exists — across the main→render world handoff
/// (even one frame behind under pipelined rendering).
pub struct RetainedIoSurface {
    surface: objc2_core_foundation::CFRetained<objc2_io_surface::IOSurfaceRef>,
    pub width: u32,
    pub height: u32,
}

impl RetainedIoSurface {
    /// Takes a +1 CoreFoundation reference on the IOSurface at `ptr`, keeping the
    /// object alive until this wrapper drops.
    ///
    /// # Safety
    /// `ptr` must be a valid, non-null `IOSurfaceRef *` (as delivered by CEF's
    /// `on_accelerated_paint`).
    pub unsafe fn retain(ptr: *mut c_void, width: u32, height: u32) -> Self {
        let nn = std::ptr::NonNull::new(ptr as *mut objc2_io_surface::IOSurfaceRef)
            .expect("RetainedIoSurface::retain called with null pointer");
        // Safety: `nn` points to a live IOSurface (valid during the callback);
        // `CFRetained::retain` adds a CF reference and returns the owning handle.
        let surface = unsafe { objc2_core_foundation::CFRetained::retain(nn) };
        Self { surface, width, height }
    }

    /// Raw `IOSurfaceRef *` for the Metal import. Valid while `self` is alive.
    pub fn ptr(&self) -> *mut c_void {
        let r: &objc2_io_surface::IOSurfaceRef = &self.surface;
        (r as *const objc2_io_surface::IOSurfaceRef) as *mut c_void
    }
}

// Safety: `RetainedIoSurface` owns a +1 CF reference, so the underlying IOSurface
// stays alive for as long as the wrapper exists — even across the main→render
// world handoff. CF reference counting is thread-safe and the surface object is
// process-wide (not thread-affine), so moving ownership to the render thread is
// sound. We never alias the wrapper (it is moved, not shared).
unsafe impl Send for RetainedIoSurface {}
unsafe impl Sync for RetainedIoSurface {}

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

    /// Imports the retained IOSurface `ptr` (size `width`×`height`) into a transient
    /// wgpu texture and records a full-surface blit into this owned surface's
    /// `encoder`. Returns the imported transient texture so the caller can keep it
    /// alive until the encoder is submitted (wgpu keeps recorded resources alive,
    /// but holding it explicitly until end-of-frame avoids any ambiguity).
    ///
    /// This is the single place where raw `wgpu`/`metal`/IOSurface naming lives, so
    /// the Bevy render-graph node (in the root crate) can call it using only Bevy
    /// types. Returns `None` if the import failed (logged by the caller).
    ///
    /// # Safety
    /// `ptr` must be a valid `IOSurfaceRef *` that stays alive for the duration of
    /// this call (guaranteed while the `RetainedIoSurface` wrapper is held).
    pub unsafe fn import_and_blit(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        ptr: *mut std::os::raw::c_void,
        width: u32,
        height: u32,
    ) -> Option<Texture> {
        // Use Bgra8UnormSrgb to match the owned surface format so the copy is a
        // same-format blit.
        let imported = unsafe {
            import_iosurface_to_wgpu(
                device.wgpu_device(),
                ptr,
                width,
                height,
                wgpu::TextureFormat::Bgra8UnormSrgb,
            )
        }?;
        self.blit_from(encoder, &imported);
        // Wrap in bevy's `Texture` so the caller (render-graph node in the root
        // crate) can hold it alive without naming the raw `wgpu` crate.
        Some(Texture::from(imported))
    }
}
