//! Shared transparent-pixel hit-testing for webview surfaces.
//!
//! Both the mesh pointer path (`WebviewPointer`) and the `bevy_ui` input path
//! use this to decide whether a pointer event lands on a fully transparent
//! pixel (and should pass through instead of reaching CEF).

use bevy::prelude::*;
#[cfg(target_os = "macos")]
use bevy_cef_core::prelude::RetainedIoSurface;

/// Returns `true` when the surface pixel under `pos_dip` is fully transparent
/// (alpha == 0), i.e. the pointer event should pass through rather than reach
/// CEF.
///
/// Returns `false` (opaque) for a zero-area image/viewport or missing pixel
/// data, so a not-yet-allocated surface forwards events instead of swallowing
/// them.
pub(crate) fn is_pixel_transparent(image: &Image, webview_size: Vec2, pos_dip: Vec2) -> bool {
    let img_size = UVec2::new(image.width(), image.height());
    if img_size.x == 0 || img_size.y == 0 || webview_size.x <= 0.0 || webview_size.y <= 0.0 {
        return false;
    }
    let px = dip_to_pixel(pos_dip, img_size, webview_size);
    let offset = ((px.y * img_size.x + px.x) * 4 + 3) as usize;
    let Some(data) = image.data.as_ref() else {
        return false;
    };
    data.len() > offset && data[offset] == 0
}

/// Returns `true` when the surface pixel under `pos_dip` is fully transparent
/// (alpha == 0) according to a pre-extracted CPU alpha buffer.
///
/// `alpha` is a flat row-major buffer with 1 byte per pixel (alpha channel only),
/// of logical size `buf_size`. `webview_size` is the DIP (logical) size of the
/// webview viewport.
///
/// Used on macOS GPU path where `Image.data` is a black placeholder; the real
/// alpha values are extracted from the IOSurface into `WebviewAlpha` each frame.
///
/// Returns `false` (opaque) when the buffer is empty, the webview has zero area,
/// or the computed pixel is out of range — so a not-yet-populated surface
/// forwards events instead of swallowing them.
///
/// Only the macOS GPU path consumes this; gated to `macos`/`test` so the
/// non-macOS lib build doesn't flag it as dead code.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn is_pixel_transparent_buf(
    alpha: &[u8],
    buf_size: UVec2,
    webview_size: Vec2,
    pos_dip: Vec2,
) -> bool {
    if buf_size.x == 0 || buf_size.y == 0 || webview_size.x <= 0.0 || webview_size.y <= 0.0 {
        return false;
    }
    if alpha.is_empty() {
        return false;
    }
    let px = dip_to_pixel(pos_dip, buf_size, webview_size);
    let offset = (px.y * buf_size.x + px.x) as usize;
    alpha.get(offset) == Some(&0)
}

/// Returns `true` when the pixel under `pos_dip` is fully transparent (alpha == 0),
/// reading a single byte on demand from the webview's retained IOSurface.
///
/// `webview_size` is the DIP (logical) viewport size; physical pixel coordinates
/// are derived via `dip_to_pixel` using the surface's physical dimensions. Returns
/// `false` (opaque) for a zero-area viewport, out-of-range pixel, or lock failure —
/// so a not-yet-painted surface forwards events instead of swallowing them.
///
/// macOS GPU path only: `Image.data` is a black placeholder there, so the real
/// alpha lives in the IOSurface behind `WebviewIoSurface`.
#[cfg(target_os = "macos")]
pub(crate) fn is_pixel_transparent_surface(
    surface: &RetainedIoSurface,
    webview_size: Vec2,
    pos_dip: Vec2,
) -> bool {
    let buf_size = UVec2::new(surface.width, surface.height);
    if buf_size.x == 0 || buf_size.y == 0 || webview_size.x <= 0.0 || webview_size.y <= 0.0 {
        return false;
    }
    let px = dip_to_pixel(pos_dip, buf_size, webview_size);
    surface.read_alpha_at(px.x, px.y) == Some(0)
}

/// Converts a DIP (logical-pixel) coordinate to a physical pixel index inside an
/// image of size `img_size`, given a logical viewport of `dip_size`.
///
/// Clamps to `img_size - 1` on each axis so the result is safe to use as a byte
/// index. Returns `UVec2::ZERO` when `dip_size` has a zero component.
fn dip_to_pixel(pos: Vec2, img_size: UVec2, dip_size: Vec2) -> UVec2 {
    if dip_size.x <= 0.0 || dip_size.y <= 0.0 || img_size.x == 0 || img_size.y == 0 {
        return UVec2::ZERO;
    }
    let sx = img_size.x as f32 / dip_size.x;
    let sy = img_size.y as f32 / dip_size.y;
    let x = ((pos.x * sx).floor() as u32).min(img_size.x - 1);
    let y = ((pos.y * sy).floor() as u32).min(img_size.y - 1);
    UVec2::new(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::Extent3d;

    fn image_with_alpha(width: u32, height: u32, alpha_per_pixel: &[u8]) -> Image {
        let mut data = vec![0u8; (width * height * 4) as usize];
        for (i, &a) in alpha_per_pixel.iter().enumerate() {
            data[i * 4 + 3] = a;
        }
        let mut image = Image::default();
        image.texture_descriptor.size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        image.data = Some(data);
        image
    }

    #[test]
    fn dip_to_pixel_identity_at_dpr_1() {
        let result = dip_to_pixel(
            Vec2::new(100.0, 200.0),
            UVec2::new(800, 800),
            Vec2::new(800.0, 800.0),
        );
        assert_eq!(result, UVec2::new(100, 200));
    }

    #[test]
    fn dip_to_pixel_scales_by_dpr_2() {
        let result = dip_to_pixel(
            Vec2::new(100.0, 200.0),
            UVec2::new(1600, 1600),
            Vec2::new(800.0, 800.0),
        );
        assert_eq!(result, UVec2::new(200, 400));
    }

    #[test]
    fn dip_to_pixel_scales_by_dpr_1_5() {
        let result = dip_to_pixel(
            Vec2::new(100.0, 100.0),
            UVec2::new(1200, 900),
            Vec2::new(800.0, 600.0),
        );
        assert_eq!(result, UVec2::new(150, 150));
    }

    #[test]
    fn dip_to_pixel_clamps_to_image_bounds() {
        let result = dip_to_pixel(
            Vec2::new(1000.0, 1000.0),
            UVec2::new(800, 800),
            Vec2::new(800.0, 800.0),
        );
        assert_eq!(result, UVec2::new(799, 799));
    }

    #[test]
    fn dip_to_pixel_zero_position_is_origin() {
        let result = dip_to_pixel(Vec2::ZERO, UVec2::new(1600, 1600), Vec2::new(800.0, 800.0));
        assert_eq!(result, UVec2::ZERO);
    }

    #[test]
    fn transparent_pixel_is_transparent() {
        let image = image_with_alpha(2, 1, &[255, 0]);
        let size = Vec2::new(2.0, 1.0);
        assert!(!is_pixel_transparent(&image, size, Vec2::new(0.0, 0.0)));
        assert!(is_pixel_transparent(&image, size, Vec2::new(1.0, 0.0)));
    }

    // ── is_pixel_transparent_buf ──────────────────────────────────────────────

    #[test]
    fn buf_left_opaque_right_transparent() {
        // 2×1 buffer: pixel 0 = opaque (255), pixel 1 = transparent (0).
        let alpha = [255u8, 0u8];
        let buf_size = UVec2::new(2, 1);
        let view_size = Vec2::new(2.0, 1.0);
        assert!(!is_pixel_transparent_buf(
            &alpha,
            buf_size,
            view_size,
            Vec2::new(0.0, 0.0)
        ));
        assert!(is_pixel_transparent_buf(
            &alpha,
            buf_size,
            view_size,
            Vec2::new(1.0, 0.0)
        ));
    }

    #[test]
    fn buf_zero_area_returns_opaque() {
        let alpha = [0u8; 4];
        // Zero webview size → opaque
        assert!(!is_pixel_transparent_buf(
            &alpha,
            UVec2::new(2, 2),
            Vec2::ZERO,
            Vec2::new(0.0, 0.0)
        ));
        // Zero buffer size → opaque
        assert!(!is_pixel_transparent_buf(
            &alpha,
            UVec2::ZERO,
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 0.0)
        ));
    }

    #[test]
    fn buf_empty_slice_returns_opaque() {
        assert!(!is_pixel_transparent_buf(
            &[],
            UVec2::new(2, 1),
            Vec2::new(2.0, 1.0),
            Vec2::new(0.0, 0.0)
        ));
    }

    #[test]
    fn buf_scales_by_dpr_2() {
        // 4×2 buffer (DPR 2×) mapped to a 2×1 DIP viewport.
        // Pixels: row 0 = [255, 255, 0, 0], row 1 = [255, 255, 0, 0]
        // DIP (0,0) → physical (0,0) = 255 → opaque
        // DIP (1,0) → physical (2,0) = 0   → transparent
        let alpha = [255u8, 255, 0, 0, 255, 255, 0, 0];
        let buf_size = UVec2::new(4, 2);
        let view_size = Vec2::new(2.0, 1.0);
        assert!(!is_pixel_transparent_buf(
            &alpha,
            buf_size,
            view_size,
            Vec2::new(0.0, 0.0)
        ));
        assert!(is_pixel_transparent_buf(
            &alpha,
            buf_size,
            view_size,
            Vec2::new(1.0, 0.0)
        ));
    }

    #[test]
    fn zero_area_or_missing_data_is_opaque() {
        let image = image_with_alpha(2, 1, &[0, 0]);
        assert!(!is_pixel_transparent(
            &image,
            Vec2::ZERO,
            Vec2::new(1.0, 0.0)
        ));

        let mut empty = Image::default();
        empty.texture_descriptor.size = Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        };
        empty.data = None;
        assert!(!is_pixel_transparent(
            &empty,
            Vec2::new(2.0, 1.0),
            Vec2::new(1.0, 0.0)
        ));
    }
}
