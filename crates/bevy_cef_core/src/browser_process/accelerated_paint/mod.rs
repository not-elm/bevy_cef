//! macOS GPU OSR: OnAcceleratedPaint + IOSurface texture import.
#![cfg(target_os = "macos")]

mod iosurface;
use iosurface::import_iosurface_to_wgpu;

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
/// ## Retain/release lifetime model
/// - `on_accelerated_paint` `CFRetain`s the freshly delivered IOSurface and stores
///   it in the per-webview latest-frame slot, **dropping (CFRelease-ing) the
///   previously stored retain** — latest-frame-wins, 1-deep. So the in-flight
///   surface is always the freshest CEF produced, and at most one stale surface is
///   held at a time (no unbounded accumulation).
/// - The main-world collect system *moves* the retain out of the slot into the
///   render world (the wrapper is `Send`), where the render-graph node imports +
///   blits it; the retain is released on the next frame's extract. The CF retain
///   only needs to keep the surface alive **until the Metal import**: the
///   `MTLTexture` created via `newTextureWithDescriptor:iosurface:plane:` holds
///   its own reference to the IOSurface, and wgpu keeps the recorded texture
///   alive until the submitted command buffer finishes on the GPU — so the blit
///   source stays valid even after our retain drops.
/// - Net: balanced CFRetain/CFRelease per frame, bounded to ~1 in-flight surface;
///   measured RSS stays flat under a 60fps hue-cycling page (no IOSurface leak).
///
/// ## Known deviation from CEF's documented contract
/// CEF documents that the delivered surface "cannot be accessed outside of this
/// callback" and is released back to Chromium's frame pool when the callback
/// returns. Deferring the copy to the render graph (and keeping a sticky clone
/// for alpha hit-testing) is therefore an accepted, contract-violating
/// optimization: the `CFRetain` guarantees *memory safety* (the object cannot be
/// freed under us), but Chromium may recycle the pooled buffer and render a
/// newer frame into it while we still read it. The pool is several buffers
/// deep, so in practice the ~1-frame blit window is clean under normal load,
/// but torn/newer content is possible under heavy load, and the sticky alpha
/// component may hit-test against a newer frame than the one displayed.
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

        // Bound the row index by the surface's OWN allocation, not just the
        // CEF-reported `coded_size` captured into `self.height`: the unsafe read
        // below must stay inside the mapped region even if CEF's metadata ever
        // disagrees with the actual IOSurface allocation.
        if py as usize >= surface_ref.height() {
            return None;
        }

        let base_ptr = surface_ref.base_address().as_ptr() as *const u8;
        let stride = surface_ref.bytes_per_row();
        // BGRA: the alpha byte sits at `px * 4 + 3` within the row. Guard against a
        // row stride smaller than that (e.g. an unexpected non-BGRA surface) so the
        // read below cannot step past the end of the row — and therefore stays
        // within the mapped region, since `py < IOSurfaceGetHeight` (checked above).
        // For a normal BGRA surface `stride >= width * 4`, so this never trips.
        let row_byte = px as usize * 4 + 3;
        if row_byte >= stride {
            return None;
        }
        let offset = py as usize * stride + row_byte;
        // Safety: `row_byte < stride` and `py < IOSurfaceGetHeight`, so
        // `offset < (py + 1) * stride <= height * stride`, i.e. inside the mapped
        // region. The read-only lock is held (guard above), keeping `base_ptr` valid.
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
// Both of our accesses are reads — the render path imports it as a Metal texture
// (GPU read) and hit-testing takes a read-only `IOSurfaceLock` (CPU read). The
// read-only `IOSurfaceLock` is a CPU cache-coherency primitive, not GPU mutual
// exclusion; soundness rests on there being NO writer in this process — CEF's GPU
// process is the sole producer and has finished the frame before delivering it.
unsafe impl Send for RetainedIoSurface {}
unsafe impl Sync for RetainedIoSurface {}

/// Per-webview owned destination texture (MVP: single buffer).
///
/// The blit node copies into it from the imported IOSurface texture; Bevy
/// materials sample it through `RenderAssets<GpuImage>`.
///
/// `texture` and `view` are stored as bevy's wrapper types so a
/// `GpuImage { texture, texture_view, .. }` can be built directly without
/// conversion.
pub struct WebviewGpuSurface {
    pub texture: Texture,
    pub view: TextureView,
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
        Self { texture, view }
    }

    /// Recreates the owned texture when the requested size differs.
    pub fn ensure_size(&mut self, device: &RenderDevice, width: u32, height: u32) {
        if self.texture.width() == width && self.texture.height() == height {
            return;
        }
        *self = Self::new(device, width, height);
    }

    /// Records a blit from `src` (an imported IOSurface texture) into the owned
    /// texture, copying the intersection of the two extents.
    ///
    /// Clamping (rather than asserting `src >= dst`) keeps a mismatched pair —
    /// e.g. two webviews of different sizes sharing one material surface handle —
    /// from recording an out-of-bounds copy, which wgpu turns into a validation
    /// panic. The shared-handle case still cannot render both pages correctly
    /// (one texture, two producers), so it is warned about, but it no longer
    /// aborts the app.
    pub fn blit_from(&self, encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture) {
        let width = self.texture.width().min(src.width());
        let height = self.texture.height().min(src.height());
        if width < self.texture.width() || height < self.texture.height() {
            bevy::log::warn_once!(
                "[macos-gpu-osr] blit source {}x{} smaller than destination {}x{}; copying the \
                 intersection (do multiple webviews share one material surface handle?)",
                src.width(),
                src.height(),
                self.texture.width(),
                self.texture.height()
            );
        }
        encoder.copy_texture_to_texture(
            src.as_image_copy(),
            self.texture.as_image_copy(),
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Imports the retained IOSurface into a transient wgpu texture and records a
    /// blit into this owned surface via `encoder`. Returns `false` if the import
    /// failed (non-Metal backend or Metal allocation failure).
    ///
    /// This is the single place where raw `wgpu`/`metal`/IOSurface naming lives, so
    /// the Bevy render-graph node (in the root crate) can call it using only Bevy
    /// types.
    ///
    /// Safe API: the `&RetainedIoSurface` borrow proves the +1 CF retain is alive
    /// for the duration of the call, which is all the unsafe import needs. The
    /// transient texture is dropped on return — recording the copy makes wgpu hold
    /// the texture (and, via the MTLTexture's own IOSurface reference, the surface)
    /// alive until the submitted command buffer completes on the GPU.
    pub fn import_and_blit(
        &self,
        device: &RenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        surface: &RetainedIoSurface,
    ) -> bool {
        // Use Bgra8UnormSrgb to match the owned surface format so the copy is a
        // same-format blit.
        // Safety: `surface.ptr()` is valid while the `&RetainedIoSurface` borrow
        // lives (the wrapper owns a +1 CF reference).
        let imported = unsafe {
            import_iosurface_to_wgpu(
                device.wgpu_device(),
                surface.ptr(),
                surface.width,
                surface.height,
                wgpu::TextureFormat::Bgra8UnormSrgb,
            )
        };
        let Some(imported) = imported else {
            return false;
        };
        self.blit_from(encoder, &imported);
        true
    }
}
