//! Derive pipeline: WebviewSize = DisplaySize × BaseRenderScale × QualityMultiplier × DPR

use bevy::prelude::*;

use super::components::*;
use super::derive_webview_size;
use crate::common::{WebviewDpr, WebviewSize};

/// Derives `WebviewSize` from pipeline components whenever any input changes.
pub(crate) fn derive_pipeline_system(
    mut webviews: Query<
        (
            &DisplaySize,
            &BaseRenderScale,
            &QualityMultiplier,
            &WebviewDpr,
            &WebviewResizable,
            &mut WebviewSize,
        ),
        (
            Without<PendingBasisInit>,
            Or<(
                Changed<DisplaySize>,
                Changed<BaseRenderScale>,
                Changed<QualityMultiplier>,
                Changed<WebviewDpr>,
                Changed<WebviewResizable>,
            )>,
        ),
    >,
) {
    for (display, base, quality, dpr, resizable, mut size) in webviews.iter_mut() {
        if let Some(new_size) = derive_webview_size(
            display.0,
            base.0,
            quality.0,
            dpr.0,
            resizable.min_size,
            resizable.max_size,
            // WebviewSize wraps Vec2, convert to UVec2 for comparison
            UVec2::new(size.0.x as u32, size.0.y as u32),
        ) {
            // WebviewSize wraps Vec2, so convert UVec2 back
            size.0 = new_size.as_vec2();
        }
    }
}

/// For 3D meshes: sync `Transform.scale.xy` from `DisplaySize / WebviewBasis2d.local_size`.
pub(crate) fn apply_display_to_mesh_system(
    mut webviews: Query<
        (&DisplaySize, &WebviewBasis2d, &mut Transform),
        (With<WebviewResizable>, Changed<DisplaySize>, With<Mesh3d>),
    >,
) {
    for (display, basis, mut transform) in webviews.iter_mut() {
        transform.scale.x = display.0.x / basis.local_size.x;
        transform.scale.y = display.0.y / basis.local_size.y;
    }
}

/// For 2D sprites: sync `Sprite.custom_size` from `DisplaySize`.
pub(crate) fn apply_display_to_sprite_system(
    mut webviews: Query<
        (&DisplaySize, &mut Sprite),
        (
            With<WebviewResizable>,
            Changed<DisplaySize>,
            Without<Mesh3d>,
        ),
    >,
) {
    for (display, mut sprite) in webviews.iter_mut() {
        sprite.custom_size = Some(display.0);
    }
}
