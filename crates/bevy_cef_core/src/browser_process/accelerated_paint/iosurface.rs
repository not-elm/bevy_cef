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
#![allow(unexpected_cfgs)]

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

    // Confirm the device uses the Metal backend before touching any Metal APIs.
    // Safety: we are only inspecting the backend type, not retaining any handles.
    let is_metal = unsafe { device.as_hal::<wgpu::hal::api::Metal>().is_some() };
    if !is_metal {
        return None;
    }

    let texture_desc = TextureDescriptor {
        label: Some("cef-iosurface-imported"),
        size: Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    };

    let metal_pixel_format = {
        use metal::MTLPixelFormat;
        match format {
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => MTLPixelFormat::BGRA8Unorm,
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => MTLPixelFormat::RGBA8Unorm,
            _ => return None,
        }
    };

    let hal_tex = objc::rc::autoreleasepool(|| unsafe {
        use metal::{MTLStorageMode, MTLTextureType, MTLTextureUsage};

        // Build the MTLTextureDescriptor for the IOSurface-backed texture.
        let metal_desc = metal::TextureDescriptor::new();
        metal_desc.set_width(width as u64);
        metal_desc.set_height(height as u64);
        metal_desc.set_texture_type(MTLTextureType::D2);
        metal_desc.set_pixel_format(metal_pixel_format);
        metal_desc.set_usage(MTLTextureUsage::ShaderRead);
        // Apple Silicon: an IOSurface-backed Metal texture must use Shared storage.
        // With Managed, GPU writes from CEF's process may not propagate, giving
        // black/stale content.
        metal_desc.set_storage_mode(MTLStorageMode::Shared);

        // Acquire a scoped reference to the wgpu-hal Metal device.
        // `as_hal` returns `Option<impl Deref<Target = hal::metal::Device>>`.
        let hal_device = device.as_hal::<wgpu::hal::api::Metal>()?;

        // wgpu-hal 27: raw_device() → &parking_lot::Mutex<metal::Device>
        // Lock it; the guard derefs to `metal::Device`, then to `metal::DeviceRef`.
        let device_guard = hal_device.raw_device().lock();

        // Call the ObjC selector  -newTextureWithDescriptor:iosurface:plane:
        // Receiver: &DeviceRef (via double-deref: MutexGuard<Device> → Device → DeviceRef).
        // `metal::DeviceRef` implements `objc::Message`; `metal::Device` does not.
        // `handle` is `*mut c_void` which implements `objc::Encode` ("^v").
        let mtl_texture: metal::Texture = msg_send![
            &**device_guard,
            newTextureWithDescriptor: metal_desc.as_ref()
            iosurface: handle
            plane: 0usize
        ];

        Some(
            // Safety: mtl_texture was just created from this device; format, type,
            // and copy_size match texture_desc.
            <wgpu::hal::api::Metal as wgpu::hal::Api>::Device::texture_from_raw(
                mtl_texture,
                texture_desc.format,
                metal::MTLTextureType::D2,
                1,
                1,
                wgpu::hal::CopyExtent { width, height, depth: 1 },
            ),
        )
    })?;

    // Safety: hal_tex was created from this device and respects texture_desc.
    Some(unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Metal>(hal_tex, &texture_desc)
    })
}
