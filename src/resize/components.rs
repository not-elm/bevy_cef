//! Components for the resize pipeline.

use bevy::prelude::*;

/// Makes a webview user-resizable by dragging its edges/corners.
///
/// Opt-in — presence of this component activates the derive pipeline
/// and edge hit-testing. Auto-requires pipeline components on insertion.
///
/// On `WebviewResizable` meshes, `Transform.scale.xy` is owned by the
/// pipeline. Use a parent entity if you need animation-driven scale.
#[derive(Component, Debug, Clone, Copy)]
pub struct WebviewResizable {
    /// Width of the invisible resize border, in **logical pixels (DIP)**.
    /// Default: 16.
    pub edge_thickness: u32,
    /// Minimum size in **logical pixels (DIP)**. Default: (100, 100).
    pub min_size: UVec2,
    /// Maximum size in **logical pixels (DIP)**. `None` = no cap.
    pub max_size: Option<UVec2>,
    /// Aspect-lock behavior during resize drag.
    pub aspect_lock: AspectLockMode,
}

impl Default for WebviewResizable {
    fn default() -> Self {
        Self {
            edge_thickness: 16,
            min_size: UVec2::new(100, 100),
            max_size: None,
            aspect_lock: AspectLockMode::default(),
        }
    }
}

/// How aspect ratio is preserved during resize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AspectLockMode {
    /// Free resize normally; Shift-drag locks to initial aspect ratio.
    #[default]
    LockOnShift,
    /// Always lock aspect ratio.
    Always,
    /// Never lock aspect ratio, even with Shift.
    Never,
}

/// Logical visual size of the webview.
///
/// - For 2D sprites: DIP (= `sprite.custom_size`).
/// - For 3D meshes: world units (= mesh local bounds × Transform.scale.xy).
///
/// Written by the resize interaction; read by the derive pipeline.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct DisplaySize(pub Vec2);

/// Snapshotted pixels-per-DisplaySize-unit ratio at spawn. Per-axis.
///
/// **Never overwritten** after initialization. Phase 2 quality profiles
/// compose via [`QualityMultiplier`] on top of this base value.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct BaseRenderScale(pub Vec2);

/// Quality profile multiplier. Phase 1: always `1.0`.
/// Phase 2 will drive this from quality profile settings.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct QualityMultiplier(pub f32);

impl Default for QualityMultiplier {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Planar basis for a 3D mesh webview. Describes the webview's local
/// width/height directions in world space and its pre-scale local size.
///
/// Phase 1 requires Z-normal planes (`Plane3d::new(Vec3::Z, ...)`).
#[derive(Component, Debug, Clone, Copy)]
pub struct WebviewBasis2d {
    /// Extent of the webview in local 2D units (width, height), pre-scale.
    pub local_size: Vec2,
}

/// Marker for entities awaiting AABB availability for basis initialization.
/// The derive pipeline skips entities with this marker.
#[derive(Component, Debug, Clone, Copy)]
pub struct PendingBasisInit;
