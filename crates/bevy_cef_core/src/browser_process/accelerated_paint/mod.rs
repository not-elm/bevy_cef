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
///
/// ## Retain/release lifetime model (verified leak/tearing-free under continuous repaint, C1)
/// - `on_accelerated_paint` `CFRetain`s the freshly delivered IOSurface and stores
///   it in the per-webview latest-frame slot, **dropping (CFRelease-ing) the
///   previously stored retain** — latest-frame-wins, 1-deep. So the in-flight
///   surface is always the freshest CEF produced, and at most one stale surface is
///   held at a time (no unbounded accumulation).
/// - The main-world collect system *moves* the retain out of the slot into the
///   render world (the wrapper is `Send`), where the render-graph node imports +
///   blits it. The retain is finally released on the **next** frame's extract
///   (when the render-world `ExtractedWebviewIoSurfaces` is cleared) — which is
///   strictly after the current frame's blit was submitted and after the imported
///   transient texture (which aliases the surface) was dropped. So the surface
///   always outlives every GPU resource that references it.
/// - Net: balanced CFRetain/CFRelease per frame, bounded to ~1 in-flight surface;
///   measured RSS stays flat under a 60fps hue-cycling page (no IOSurface leak),
///   and the owned texture updates every frame with no tearing.
#[derive(Clone)]
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
        Self {
            surface,
            width,
            height,
        }
    }

    /// Raw `IOSurfaceRef *` for the Metal import. Valid while `self` is alive.
    pub fn ptr(&self) -> *mut c_void {
        let r: &objc2_io_surface::IOSurfaceRef = &self.surface;
        (r as *const objc2_io_surface::IOSurfaceRef) as *mut c_void
    }

    /// Reads the alpha byte (BGRA: offset +3) of the single physical pixel at
    /// `(px, py)`, on demand.
    ///
    /// Bounds-checks BEFORE locking (returns `None` for out-of-range coords to
    /// avoid a needless lock). Locks read-only, reads one byte, and unlocks via
    /// an RAII guard so the unlock always runs (even on a panic). Returns `None`
    /// if `(px, py)` is out of range or the lock fails.
    ///
    /// Runs on the Bevy main thread (= CEF UI thread under `external_message_pump`)
    /// only when a pointer is over the webview — far cheaper than per-frame
    /// full-plane alpha extraction.
    pub fn read_alpha_at(&self, px: u32, py: u32) -> Option<u8> {
        use objc2_io_surface::IOSurfaceLockOptions;

        if px >= self.width || py >= self.height {
            return None;
        }

        let surface_ref: &objc2_io_surface::IOSurfaceRef = &self.surface;
        // Safety: surface_ref is valid while `self` is alive (+1 CF ref).
        let lock_result =
            unsafe { surface_ref.lock(IOSurfaceLockOptions::ReadOnly, std::ptr::null_mut()) };
        if lock_result != 0 {
            return None;
        }

        // RAII: unlock on every exit path with the same ReadOnly flag.
        struct UnlockGuard<'a>(&'a objc2_io_surface::IOSurfaceRef);
        impl Drop for UnlockGuard<'_> {
            fn drop(&mut self) {
                // Safety: balanced with the ReadOnly lock above.
                unsafe {
                    self.0
                        .unlock(IOSurfaceLockOptions::ReadOnly, std::ptr::null_mut());
                }
            }
        }
        let _guard = UnlockGuard(surface_ref);

        let base_ptr = surface_ref.base_address().as_ptr() as *const u8;
        let stride = surface_ref.bytes_per_row();
        let offset = py as usize * stride + px as usize * 4 + 3;
        // Safety: offset is within the mapped region (px < width, py < height).
        Some(unsafe { *base_ptr.add(offset) })
    }
}

// Safety: `RetainedIoSurface` owns a +1 CF reference, so the underlying IOSurface
// stays alive for as long as the wrapper exists — even across the main→render
// world handoff. CF reference counting is thread-safe and the surface object is
// process-wide (not thread-affine), so moving ownership to the render thread is
// sound. The surface MAY be aliased: the render path moves one retain into the
// render world while the main world holds a second, independent `CFRetain` (via
// `Clone`) in a `WebviewIoSurface` component for on-demand alpha hit-testing.
// Both accesses are READ-ONLY — the render path imports it as a Metal texture
// (GPU read) and hit-testing takes a read-only `IOSurfaceLock` (CPU read) — so
// the alias is sound (no writer on our side; CEF's GPU process is the producer).
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
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        Self {
            texture,
            view,
            width,
            height,
        }
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
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
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
