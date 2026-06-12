//! IOSurface (CEF shared texture) → Metal MTLTexture → wgpu::Texture import for wgpu 27.
//!
//! # wgpu 27 vs wgpu 28 differences
//! In wgpu-hal 27, `Device::raw_device()` returns `&parking_lot::Mutex<metal::Device>` (not
//! `&metal::Device` as in wgpu 28). We must `.lock()` the mutex to obtain a
//! `MutexGuard<metal::Device>`, then deref to `&DeviceRef` for the `msg_send!` call.
//!
//! # IOSurfaceRef compatibility
//! `IOSurfaceRef` from `objc2-io-surface` 0.3 implements `objc2::RefEncode`, NOT `objc::Encode`
//! (the older `objc` 0.2 crate trait used by `msg_send!`). We therefore pass the raw
//! `*mut c_void` CEF handle directly — `*mut c_void` does implement `objc::Encode` ("^v").
//! The Metal runtime accepts any opaque IOSurface pointer at this position.

use std::os::raw::c_void;

use objc::{msg_send, sel, sel_impl};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

/// Imports a CEF IOSurface handle into a wgpu texture (COPY_SRC | TEXTURE_BINDING).
///
/// Returns `None` if:
/// - `handle` is null or dimensions are zero
/// - the `device` is not backed by the Metal API
/// - any Metal API call returns null (allocation failure)
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

    let metal_pixel_format = {
        use metal::MTLPixelFormat;
        // The sRGB variants MUST get their own arms: `texture_from_raw` is told
        // `texture_desc.format` (the sRGB format) below, so the Metal texture's
        // actual pixel format must agree, or `texture_from_raw`'s safety
        // invariant is violated.
        match format {
            TextureFormat::Bgra8Unorm => MTLPixelFormat::BGRA8Unorm,
            TextureFormat::Bgra8UnormSrgb => MTLPixelFormat::BGRA8Unorm_sRGB,
            TextureFormat::Rgba8Unorm => MTLPixelFormat::RGBA8Unorm,
            TextureFormat::Rgba8UnormSrgb => MTLPixelFormat::RGBA8Unorm_sRGB,
            _ => return None,
        }
    };

    let hal_tex = objc::rc::autoreleasepool(|| unsafe {
        use metal::foreign_types::ForeignType;
        use metal::{MTLStorageMode, MTLTextureType, MTLTextureUsage};

        let hal_device = device.as_hal::<wgpu::hal::api::Metal>()?;
        let device_guard = hal_device.raw_device().lock();

        let metal_desc = metal::TextureDescriptor::new();
        metal_desc.set_width(width as u64);
        metal_desc.set_height(height as u64);
        metal_desc.set_texture_type(MTLTextureType::D2);
        metal_desc.set_pixel_format(metal_pixel_format);
        metal_desc.set_usage(MTLTextureUsage::ShaderRead);
        // Storage mode must match the device's memory architecture: Shared
        // textures exist only on unified-memory (Apple-silicon) devices — on
        // Intel/AMD Macs `newTextureWithDescriptor:iosurface:plane:` rejects
        // Shared and the import would fail every frame (permanently black
        // webviews, since macOS has no CPU fallback). Chromium and the cef
        // crate's reference importer use Managed there. On Apple silicon,
        // Shared is required: with Managed, GPU writes from CEF's process may
        // not propagate, giving black/stale content.
        metal_desc.set_storage_mode(if device_guard.has_unified_memory() {
            MTLStorageMode::Shared
        } else {
            MTLStorageMode::Managed
        });

        // `newTextureWithDescriptor:…` returns nil on failure (e.g. OOM or an
        // invalid surface); null-check before wrapping in `metal::Texture`, or
        // the blit would dereference nil later.
        let raw: *mut objc::runtime::Object = msg_send![
            &**device_guard,
            newTextureWithDescriptor: metal_desc.as_ref()
            iosurface: handle
            plane: 0usize
        ];
        if raw.is_null() {
            return None;
        }
        // The returned MTLTexture is +1-owned; `from_ptr` takes that ownership
        // without an extra retain.
        let mtl_texture = metal::Texture::from_ptr(raw.cast());

        Some(
            // Safety: mtl_texture was just created from this device; format, type,
            // and copy_size match texture_desc.
            <wgpu::hal::api::Metal as wgpu::hal::Api>::Device::texture_from_raw(
                mtl_texture,
                texture_desc.format,
                metal::MTLTextureType::D2,
                1,
                1,
                wgpu::hal::CopyExtent {
                    width,
                    height,
                    depth: 1,
                },
            ),
        )
    })?;

    // Safety: hal_tex was created from this device and respects texture_desc.
    Some(unsafe { device.create_texture_from_hal::<wgpu::hal::api::Metal>(hal_tex, &texture_desc) })
}
