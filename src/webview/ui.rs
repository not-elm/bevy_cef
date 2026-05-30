//! `bevy_ui` webview display path: renders a webview into a `MaterialNode<WebviewUiMaterial>`.

use bevy::math::Vec2;

mod material;

/// Converts a node's physical-pixel `ComputedNode` size to the logical DIP size
/// `WebviewSize` expects. Returns `None` for a pre-layout / sub-pixel size so a
/// 0-area surface is never requested.
pub(crate) fn webview_size_from_computed(physical_size: Vec2, inverse_scale_factor: f32) -> Option<Vec2> {
    let logical = physical_size * inverse_scale_factor;
    if logical.x < 1.0 || logical.y < 1.0 {
        None
    } else {
        Some(logical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    #[test]
    fn converts_physical_to_logical() {
        // 1600x1200 physical at 2x DPI (inverse 0.5) -> 800x600 logical.
        let out = webview_size_from_computed(Vec2::new(1600.0, 1200.0), 0.5);
        assert_eq!(out, Some(Vec2::new(800.0, 600.0)));
    }

    #[test]
    fn identity_at_1x() {
        let out = webview_size_from_computed(Vec2::new(640.0, 480.0), 1.0);
        assert_eq!(out, Some(Vec2::new(640.0, 480.0)));
    }

    #[test]
    fn zero_and_subpixel_return_none() {
        assert_eq!(webview_size_from_computed(Vec2::ZERO, 1.0), None);
        assert_eq!(webview_size_from_computed(Vec2::new(0.0, 600.0), 1.0), None);
        assert_eq!(webview_size_from_computed(Vec2::new(0.5, 0.5), 1.0), None);
    }
}
