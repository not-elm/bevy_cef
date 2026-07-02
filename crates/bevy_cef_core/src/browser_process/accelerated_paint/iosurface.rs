//! IOSurface (CEF shared texture) → Metal MTLTexture → wgpu::Texture import for wgpu 29.
//!
//! # wgpu-hal 29 / objc2-metal
//! wgpu-hal 29's Metal backend is built on `objc2-metal` (not `metal-rs`):
//! `Device::raw_device()` returns `&Retained<ProtocolObject<dyn MTLDevice>>`
//! directly (wgpu 27 returned `&Mutex<metal::Device>` — no lock is needed now),
//! and `texture_from_raw` takes the `Retained` MTLTexture by value.
//!
//! The import uses objc2-metal's native `newTextureWithDescriptor:iosurface:plane:`
//! binding, so the old raw-`objc` `msg_send!` hack is gone. The raw CEF handle is
//! reborrowed as `&IOSurfaceRef` (objc2-io-surface 0.3 — the same version line
//! wgpu-hal 29 uses, so the type identities unify).
//!
//! The Metal calls run inside an `autoreleasepool`: although every object we
//! receive is `new`-family (+1 retained), Apple frameworks may autorelease
//! internal temporaries while servicing the calls, and Bevy's pipelined render
//! thread (a plain `std::thread`) has no ambient pool — without one, those
//! temporaries would leak a little every frame. wgpu-hal 29 wraps its own
//! structurally identical `MTLTextureDescriptor::new()` + create paths the
//! same way.

use std::os::raw::c_void;

use objc2_io_surface::IOSurfaceRef;
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

/// Imports a CEF IOSurface handle into a wgpu texture (COPY_SRC | TEXTURE_BINDING).
///
/// Returns `None` if:
/// - `handle` is null or dimensions are zero
/// - the `device` is not backed by the Metal API
/// - any Metal API call returns nil (allocation failure)
///
/// # Safety
/// `handle` must be a valid `IOSurfaceRef *` that remains alive for the duration of this
/// call. CEF guarantees validity only inside `on_accelerated_paint`; the caller must not
/// store or use the handle after that callback returns.
pub unsafe fn import_iosurface_to_wgpu(
    device: &wgpu::Device,
    handle: *mut c_void,
    width: u32,
    height: u32,
    format: TextureFormat,
) -> Option<wgpu::Texture> {
    if handle.is_null() || width == 0 || height == 0 {
        return None;
    }

    let texture_desc = TextureDescriptor {
        label: Some("cef-iosurface-imported"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    };

    // The sRGB variants MUST get their own arms: `texture_from_raw` is told
    // `texture_desc.format` (the sRGB format) below, so the Metal texture's
    // actual pixel format must agree, or `texture_from_raw`'s safety
    // invariant is violated.
    let metal_pixel_format = match format {
        TextureFormat::Bgra8Unorm => MTLPixelFormat::BGRA8Unorm,
        TextureFormat::Bgra8UnormSrgb => MTLPixelFormat::BGRA8Unorm_sRGB,
        TextureFormat::Rgba8Unorm => MTLPixelFormat::RGBA8Unorm,
        TextureFormat::Rgba8UnormSrgb => MTLPixelFormat::RGBA8Unorm_sRGB,
        _ => return None,
    };

    let hal_tex = objc2::rc::autoreleasepool(|_| {
        let hal_device = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }?;
        let raw_device = hal_device.raw_device();

        let metal_desc = MTLTextureDescriptor::new();
        unsafe {
            metal_desc.setWidth(width as usize);
            metal_desc.setHeight(height as usize);
        }
        metal_desc.setTextureType(MTLTextureType::Type2D);
        metal_desc.setPixelFormat(metal_pixel_format);
        metal_desc.setUsage(MTLTextureUsage::ShaderRead);
        // Storage mode must match the device's memory architecture: Shared
        // textures exist only on unified-memory (Apple-silicon) devices — on
        // Intel/AMD Macs `newTextureWithDescriptor:iosurface:plane:` rejects
        // Shared and the import would fail every frame (permanently black
        // webviews, since macOS has no CPU fallback). Chromium and the cef
        // crate's reference importer use Managed there. On Apple silicon,
        // Shared is required: with Managed, GPU writes from CEF's process may
        // not propagate, giving black/stale content.
        metal_desc.setStorageMode(if raw_device.hasUnifiedMemory() {
            MTLStorageMode::Shared
        } else {
            MTLStorageMode::Managed
        });

        // SAFETY: `handle` is a valid IOSurfaceRef* for the duration of this
        // call (see the function's safety contract); the reborrow does not
        // extend its lifetime.
        let iosurface: &IOSurfaceRef = unsafe { &*handle.cast::<IOSurfaceRef>() };
        // Returns nil on failure (e.g. OOM, invalid surface, or a Shared
        // texture requested on a non-unified-memory device) — `Option` return
        // preserves the old nil-check semantics. (Safe method in this
        // objc2-metal feature set — no `unsafe` block.)
        let mtl_texture =
            raw_device.newTextureWithDescriptor_iosurface_plane(&metal_desc, iosurface, 0)?;

        // Safety: mtl_texture was just created from this device; format, type,
        // and copy_size match texture_desc.
        Some(unsafe {
            <wgpu::hal::api::Metal as wgpu::hal::Api>::Device::texture_from_raw(
                mtl_texture,
                texture_desc.format,
                MTLTextureType::Type2D,
                1,
                1,
                wgpu::hal::CopyExtent {
                    width,
                    height,
                    depth: 1,
                },
            )
        })
    })?;

    // Safety: hal_tex was created from this device and respects texture_desc.
    Some(unsafe { device.create_texture_from_hal::<wgpu::hal::api::Metal>(hal_tex, &texture_desc) })
}
