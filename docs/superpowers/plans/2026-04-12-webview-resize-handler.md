# Webview Drag-to-Resize Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add native-OS-window-style drag-to-resize to bevy_cef webviews, with a unified derivation pipeline for Phase 2 quality profile compatibility.

**Architecture:** Opt-in `WebviewResizable` component activates a derive pipeline (`WebviewSize = DisplaySize × BaseRenderScale × QualityMultiplier × DPR`). Resize interaction writes `DisplaySize`; the pipeline derives `WebviewSize`. Edge hit-test, anchor math, and cursor feedback reuse the existing drag infrastructure patterns. System ordering via `WebviewSet` ensures the pipeline runs before browser creation/resize commit.

**Tech Stack:** Bevy 0.18 ECS (components, system sets, observers, change detection), CEF offscreen rendering, Rust edition 2024

**Design Spec:** `docs/superpowers/specs/2026-04-12-webview-resize-handler-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src/resize.rs` | **Create** | `ResizeZone`, `ResizeFrame`, `classify_zone`, `HitResult`, `classify_hit`, `apply_resize`, `derive_webview_size` pure functions + unit tests |
| `src/resize/plugin.rs` | **Create** | `ResizePlugin`, `ResizeState`, `ResizeInteractionPlugin` systems: `resize_tracking_system`, `cursor_hover_system`, `on_webview_press` observer, `WebviewResizable` on_add hook |
| `src/resize/components.rs` | **Create** | `WebviewResizable`, `AspectLockMode`, `DisplaySize`, `BaseRenderScale`, `QualityMultiplier`, `WebviewDpr`, `WebviewBasis2d`, `PendingBasisInit` |
| `src/resize/pipeline.rs` | **Create** | `derive_pipeline_system`, `pending_basis_init_system`, `apply_display_to_mesh_system`, `apply_display_to_sprite_system` |
| `src/resize/cursor.rs` | **Create** | `SystemCursorOverride`, `cursor_hover_system`, `cursor_for_zone` |
| `src/drag.rs` | **Modify** | Rename `DragEndPending` → `InteractionEndPending`, export `is_draggable`, replace `attach_drag_observers` → unified observer, add `ResizeState` gate |
| `src/webview.rs` | **Modify** | Move systems into `WebviewSet` sets, add `ResizeState` guard to CEF input systems |
| `src/webview/mesh.rs` | **Modify** | Add `ResizeState` guard to `on_pointer_move/pressed/released/wheel` |
| `src/webview/webview_sprite.rs` | **Modify** | Add `ResizeState` guard to `apply_on_pointer_move/pressed/released` + `on_mouse_wheel`, export `obtain_relative_pos` |
| `src/cursor_icon.rs` | **Modify** | Add `SystemCursorOverride` priority check in `update_cursor_icon` |
| `src/lib.rs` | **Modify** | Add `ResizePlugin`, export new components in prelude |
| `src/common/components.rs` | **Modify** | Change `WebviewSize` from `Vec2` to `UVec2` if needed (verify current type) |
| `examples/resize.rs` | **Create** | Integration example with 3D mesh + 2D sprite resizable webviews |
| `assets/resize_demo.html` | **Create** | Demo HTML with live dimension display + drag title bar |

---

## Task 1: Pure Functions — Zone Classification

**Files:**
- Create: `src/resize.rs`

This task implements the zone classification logic and its tests. All pure functions, no ECS.

- [ ] **Step 1: Create `src/resize.rs` with `ResizeZone` and `ResizeFrame`**

```rust
// src/resize.rs
//! Resize feature: drag-to-resize webviews with automatic edge detection.

use bevy::prelude::*;

/// One of the 8 resize zones around a webview's edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeZone {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

/// Describes the resize-sensitive frame around a webview in texture-pixel space.
#[derive(Debug, Clone, Copy)]
pub struct ResizeFrame {
    /// Webview texture width in pixels.
    pub width: u32,
    /// Webview texture height in pixels.
    pub height: u32,
    /// Edge thickness in texture pixels.
    pub edge_thickness: u32,
}

impl ResizeFrame {
    /// Classify a texture-pixel position into a resize zone, or `None` if interior.
    /// Corners win over edges when both axes are in the border.
    pub fn classify(&self, pos: Vec2) -> Option<ResizeZone> {
        let w = self.width as f32;
        let h = self.height as f32;
        let t = self.edge_thickness as f32;

        // Out of bounds — not in the webview at all.
        if pos.x < 0.0 || pos.y < 0.0 || pos.x > w || pos.y > h {
            return None;
        }

        let in_left = pos.x < t;
        let in_right = pos.x > w - t;
        let in_top = pos.y < t;
        let in_bottom = pos.y > h - t;

        match (in_left, in_right, in_top, in_bottom) {
            (true, _, true, _) => Some(ResizeZone::NW),
            (_, true, true, _) => Some(ResizeZone::NE),
            (true, _, _, true) => Some(ResizeZone::SW),
            (_, true, _, true) => Some(ResizeZone::SE),
            (true, _, _, _) => Some(ResizeZone::W),
            (_, true, _, _) => Some(ResizeZone::E),
            (_, _, true, _) => Some(ResizeZone::N),
            (_, _, _, true) => Some(ResizeZone::S),
            _ => None, // interior
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: u32, h: u32, t: u32) -> ResizeFrame {
        ResizeFrame { width: w, height: h, edge_thickness: t }
    }

    #[test]
    fn classify_zone_interior_returns_none() {
        let f = frame(800, 600, 16);
        assert_eq!(f.classify(Vec2::new(400.0, 300.0)), None);
        assert_eq!(f.classify(Vec2::new(16.0, 16.0)), None); // exactly at interior boundary
    }

    #[test]
    fn classify_zone_north_edge() {
        let f = frame(800, 600, 16);
        assert_eq!(f.classify(Vec2::new(400.0, 8.0)), Some(ResizeZone::N));
    }

    #[test]
    fn classify_zone_nw_corner_wins_over_n_and_w() {
        let f = frame(800, 600, 16);
        // (3, 3) is in both left edge and top edge — corner wins
        assert_eq!(f.classify(Vec2::new(3.0, 3.0)), Some(ResizeZone::NW));
    }

    #[test]
    fn classify_zone_outside_webview_returns_none() {
        let f = frame(800, 600, 16);
        assert_eq!(f.classify(Vec2::new(-1.0, 300.0)), None);
        assert_eq!(f.classify(Vec2::new(900.0, 300.0)), None);
    }

    #[test]
    fn classify_zone_respects_edge_thickness_config() {
        // With thickness 32, pos (20, 300) is in the left border
        let f = frame(800, 600, 32);
        assert_eq!(f.classify(Vec2::new(20.0, 300.0)), Some(ResizeZone::W));
        // With thickness 16, same pos is interior
        let f2 = frame(800, 600, 16);
        assert_eq!(f2.classify(Vec2::new(20.0, 300.0)), None);
    }

    #[test]
    fn classify_zone_all_eight_zones() {
        let f = frame(800, 600, 16);
        assert_eq!(f.classify(Vec2::new(3.0, 3.0)), Some(ResizeZone::NW));
        assert_eq!(f.classify(Vec2::new(400.0, 3.0)), Some(ResizeZone::N));
        assert_eq!(f.classify(Vec2::new(797.0, 3.0)), Some(ResizeZone::NE));
        assert_eq!(f.classify(Vec2::new(3.0, 300.0)), Some(ResizeZone::W));
        assert_eq!(f.classify(Vec2::new(797.0, 300.0)), Some(ResizeZone::E));
        assert_eq!(f.classify(Vec2::new(3.0, 597.0)), Some(ResizeZone::SW));
        assert_eq!(f.classify(Vec2::new(400.0, 597.0)), Some(ResizeZone::S));
        assert_eq!(f.classify(Vec2::new(797.0, 597.0)), Some(ResizeZone::SE));
    }
}
```

- [ ] **Step 2: Add `mod resize;` to `src/lib.rs`**

Add `mod resize;` after the existing `mod drag;` line (around line 12 in `src/lib.rs`). Don't add it to the prelude yet.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test --workspace --all-features -- resize`
Expected: All 6 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/resize.rs src/lib.rs
git commit -m "feat(resize): add ResizeZone and ResizeFrame with zone classification tests"
```

---

## Task 2: Pure Functions — Unified Hit-Test

**Files:**
- Modify: `src/resize.rs` (add `HitResult`, `classify_hit`)

- [ ] **Step 1: Add `HitResult` and `classify_hit` to `src/resize.rs`**

Add after the `ResizeFrame` impl block, before `#[cfg(test)]`:

```rust
use crate::drag::{is_draggable, DraggableRegions};

/// Result of classifying a pointer position on a webview.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HitResult {
    /// Position is in a resize edge/corner zone.
    Resize(ResizeZone),
    /// Position is in a `-webkit-app-region: drag` region (but not a resize edge).
    Drag,
    /// Position is in the webview interior — normal page input.
    None,
}

/// Classify a pointer's texture-pixel position: resize edge > drag region > page input.
pub(crate) fn classify_hit(
    regions: &DraggableRegions,
    frame: Option<&ResizeFrame>,
    pos: Vec2,
) -> HitResult {
    // Resize edges take priority over drag regions (matches native OS window behavior).
    if let Some(frame) = frame {
        if let Some(zone) = frame.classify(pos) {
            return HitResult::Resize(zone);
        }
    }
    if is_draggable(&regions.drag_rects, &regions.no_drag_rects, pos) {
        return HitResult::Drag;
    }
    HitResult::None
}
```

- [ ] **Step 2: Make `is_draggable` and `DraggableRegions` visible to `resize.rs`**

In `src/drag.rs`, change `pub(crate) fn is_draggable` (line 132) and `pub(crate) struct DraggableRegions` (line 52) to remain `pub(crate)` (they already are — just verify).

- [ ] **Step 3: Add tests for `classify_hit`**

Add to the `tests` module in `src/resize.rs`:

```rust
    use crate::drag::{DraggableRegions, PixelRect};

    #[test]
    fn resize_edge_wins_over_drag_region_at_boundary() {
        let regions = DraggableRegions {
            drag_rects: vec![PixelRect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(800.0, 40.0),
            }],
            no_drag_rects: vec![],
        };
        let f = frame(800, 600, 16);
        let resize_frame = ResizeFrame { width: 800, height: 600, edge_thickness: 16 };
        // Pixel (3, 3) is in both the drag region AND the NW resize corner
        let result = classify_hit(&regions, Some(&resize_frame), Vec2::new(3.0, 3.0));
        assert_eq!(result, HitResult::Resize(ResizeZone::NW));
    }

    #[test]
    fn interior_drag_region_unchanged_by_resize_edges() {
        let regions = DraggableRegions {
            drag_rects: vec![PixelRect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(800.0, 40.0),
            }],
            no_drag_rects: vec![],
        };
        let resize_frame = ResizeFrame { width: 800, height: 600, edge_thickness: 16 };
        // Pixel (400, 25) is in the drag region but NOT in any resize edge
        let result = classify_hit(&regions, Some(&resize_frame), Vec2::new(400.0, 25.0));
        assert_eq!(result, HitResult::Drag);
    }

    #[test]
    fn classify_hit_no_resize_frame_falls_through_to_drag() {
        let regions = DraggableRegions {
            drag_rects: vec![PixelRect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(800.0, 40.0),
            }],
            no_drag_rects: vec![],
        };
        // No resize frame = non-resizable webview
        let result = classify_hit(&regions, None, Vec2::new(3.0, 3.0));
        assert_eq!(result, HitResult::Drag);
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test --workspace --all-features -- resize`
Expected: All 9 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/resize.rs src/drag.rs
git commit -m "feat(resize): add unified hit-test with resize-over-drag priority"
```

---

## Task 3: Pure Functions — Apply Resize (Anchor Math)

**Files:**
- Modify: `src/resize.rs` (add `apply_resize` function + 8 anchor tests)

- [ ] **Step 1: Add `apply_resize` function**

Add after `classify_hit`, before `#[cfg(test)]`:

```rust
/// Compute new display size and translation after a resize drag.
///
/// The pinned-corner rule: dragging one edge/corner keeps the opposite
/// edge/corner fixed in world space. Assumes centered origin (sprite
/// `Anchor::Center` / mesh centered at origin).
pub(crate) fn apply_resize(
    zone: ResizeZone,
    start_size: Vec2,
    start_translation: Vec3,
    du: f32,
    dv: f32,
    u_axis: Vec3,
    v_axis: Vec3,
    lock_aspect: bool,
    min_size: Vec2,
    max_size: Option<Vec2>,
) -> (Vec2, Vec3) {
    // 1. Raw size deltas (signed by zone).
    let (dw_raw, dh_raw) = match zone {
        ResizeZone::E => (du, 0.0),
        ResizeZone::W => (-du, 0.0),
        ResizeZone::S => (0.0, dv),
        ResizeZone::N => (0.0, -dv),
        ResizeZone::NE => (du, -dv),
        ResizeZone::NW => (-du, -dv),
        ResizeZone::SE => (du, dv),
        ResizeZone::SW => (-du, dv),
    };
    let mut new_size = Vec2::new(start_size.x + dw_raw, start_size.y + dh_raw);

    // 2. Aspect lock: dominant axis wins.
    if lock_aspect {
        let aspect = start_size.x / start_size.y;
        if dw_raw.abs() * (1.0 / aspect) > dh_raw.abs() {
            new_size.y = new_size.x / aspect;
        } else {
            new_size.x = new_size.y * aspect;
        }
    }

    // 3. Clamp.
    new_size = new_size.max(min_size);
    if let Some(max) = max_size {
        new_size = new_size.min(max);
    }

    // 4. Actual deltas after clamping.
    let actual_dw = new_size.x - start_size.x;
    let actual_dh = new_size.y - start_size.y;

    // 5. Translation delta (pinned-corner rule for centered origin).
    let sign_u = match zone {
        ResizeZone::E | ResizeZone::NE | ResizeZone::SE => 0.5,
        ResizeZone::W | ResizeZone::NW | ResizeZone::SW => -0.5,
        _ => 0.0,
    };
    let sign_v = match zone {
        ResizeZone::S | ResizeZone::SE | ResizeZone::SW => 0.5,
        ResizeZone::N | ResizeZone::NE | ResizeZone::NW => -0.5,
        _ => 0.0,
    };
    let new_translation =
        start_translation + u_axis * (actual_dw * sign_u) + v_axis * (actual_dh * sign_v);

    (new_size, new_translation)
}
```

- [ ] **Step 2: Add anchor math tests**

Add to the `tests` module. Each test verifies the pinned point stays fixed:

```rust
    // Helper: compute world position of a point at fractional (fx, fy) of a centered rect.
    // (0, 0) = bottom-left, (1, 1) = top-right, (0.5, 0.5) = center.
    fn world_pos_of(
        translation: Vec3,
        size: Vec2,
        u_axis: Vec3,
        v_axis: Vec3,
        fx: f32,
        fy: f32,
    ) -> Vec3 {
        translation + u_axis * size.x * (fx - 0.5) + v_axis * size.y * (fy - 0.5)
    }

    fn assert_pinned(before: Vec3, after: Vec3) {
        assert!(
            (before - after).length() < 1e-4,
            "Pinned point moved: {before:?} → {after:?}"
        );
    }

    const U: Vec3 = Vec3::X;
    const V: Vec3 = Vec3::Y;
    const START_SIZE: Vec2 = Vec2::new(2.0, 2.0);
    const START_TR: Vec3 = Vec3::ZERO;
    const MIN: Vec2 = Vec2::new(0.1, 0.1);

    #[test]
    fn apply_resize_east_pins_west_edge() {
        let west_before = world_pos_of(START_TR, START_SIZE, U, V, 0.0, 0.5);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::E, START_SIZE, START_TR, 1.5, 0.0, U, V, false, MIN, None);
        let west_after = world_pos_of(new_tr, new_size, U, V, 0.0, 0.5);
        assert_pinned(west_before, west_after);
    }

    #[test]
    fn apply_resize_west_pins_east_edge() {
        let east_before = world_pos_of(START_TR, START_SIZE, U, V, 1.0, 0.5);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::W, START_SIZE, START_TR, 0.8, 0.0, U, V, false, MIN, None);
        let east_after = world_pos_of(new_tr, new_size, U, V, 1.0, 0.5);
        assert_pinned(east_before, east_after);
    }

    #[test]
    fn apply_resize_north_pins_south_edge() {
        let south_before = world_pos_of(START_TR, START_SIZE, U, V, 0.5, 0.0);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::N, START_SIZE, START_TR, 0.0, -1.0, U, V, false, MIN, None);
        let south_after = world_pos_of(new_tr, new_size, U, V, 0.5, 0.0);
        assert_pinned(south_before, south_after);
    }

    #[test]
    fn apply_resize_south_pins_north_edge() {
        let north_before = world_pos_of(START_TR, START_SIZE, U, V, 0.5, 1.0);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::S, START_SIZE, START_TR, 0.0, 1.0, U, V, false, MIN, None);
        let north_after = world_pos_of(new_tr, new_size, U, V, 0.5, 1.0);
        assert_pinned(north_before, north_after);
    }

    #[test]
    fn apply_resize_ne_pins_sw_corner() {
        let sw_before = world_pos_of(START_TR, START_SIZE, U, V, 0.0, 0.0);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::NE, START_SIZE, START_TR, 0.5, -0.5, U, V, false, MIN, None);
        let sw_after = world_pos_of(new_tr, new_size, U, V, 0.0, 0.0);
        assert_pinned(sw_before, sw_after);
    }

    #[test]
    fn apply_resize_nw_pins_se_corner() {
        let se_before = world_pos_of(START_TR, START_SIZE, U, V, 1.0, 0.0);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::NW, START_SIZE, START_TR, -0.5, -0.5, U, V, false, MIN, None);
        let se_after = world_pos_of(new_tr, new_size, U, V, 1.0, 0.0);
        assert_pinned(se_before, se_after);
    }

    #[test]
    fn apply_resize_se_pins_nw_corner() {
        let nw_before = world_pos_of(START_TR, START_SIZE, U, V, 0.0, 1.0);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::SE, START_SIZE, START_TR, 0.5, 0.5, U, V, false, MIN, None);
        let nw_after = world_pos_of(new_tr, new_size, U, V, 0.0, 1.0);
        assert_pinned(nw_before, nw_after);
    }

    #[test]
    fn apply_resize_sw_pins_ne_corner() {
        let ne_before = world_pos_of(START_TR, START_SIZE, U, V, 1.0, 1.0);
        let (new_size, new_tr) =
            apply_resize(ResizeZone::SW, START_SIZE, START_TR, -0.5, 0.5, U, V, false, MIN, None);
        let ne_after = world_pos_of(new_tr, new_size, U, V, 1.0, 1.0);
        assert_pinned(ne_before, ne_after);
    }
```

- [ ] **Step 3: Run tests**

Run: `cargo test --workspace --all-features -- apply_resize`
Expected: All 8 anchor tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/resize.rs
git commit -m "feat(resize): add apply_resize anchor math with pinned-corner tests"
```

---

## Task 4: Pure Functions — Derive Pipeline + Aspect Lock

**Files:**
- Modify: `src/resize.rs` (add `derive_webview_size`, aspect lock tests, pipeline tests)

- [ ] **Step 1: Add `derive_webview_size` function**

Add after `apply_resize`, before `#[cfg(test)]`:

```rust
/// Derive the target `WebviewSize` from the pipeline inputs.
/// Returns `None` if the computed size equals the current size (no-op).
pub(crate) fn derive_webview_size(
    display_size: Vec2,
    base_render_scale: Vec2,
    quality_multiplier: f32,
    dpr: f32,
    min_size: UVec2,
    max_size: Option<UVec2>,
    current_size: UVec2,
) -> Option<UVec2> {
    let raw = display_size * base_render_scale * quality_multiplier * dpr;
    let mut target = UVec2::new(raw.x.round().max(1.0) as u32, raw.y.round().max(1.0) as u32);

    // Clamp.
    target = target.max(min_size);
    if let Some(max) = max_size {
        target = target.min(max);
    }

    // Dedupe on integer change.
    if target == current_size {
        None
    } else {
        Some(target)
    }
}
```

- [ ] **Step 2: Add pipeline derivation and aspect-lock tests**

Add to the `tests` module:

```rust
    #[test]
    fn derive_sprite_pixel_size_from_display_and_dpr() {
        // Sprite: custom_size = 400×300, BaseRenderScale = (2, 2) (initial 800×600 / 400×300 / DPR 1),
        // QualityMul = 1.0, DPR = 2.0 → target = (400 × 2 × 1 × 2) = 1600×1200
        let result = derive_webview_size(
            Vec2::new(400.0, 300.0),
            Vec2::new(2.0, 2.0),
            1.0,
            2.0,
            UVec2::new(100, 100),
            None,
            UVec2::ZERO,
        );
        assert_eq!(result, Some(UVec2::new(1600, 1200)));
    }

    #[test]
    fn derive_mesh_pixel_size_preserves_initial_ratio() {
        // Mesh: world size 2×2, WebviewSize 800×800, DPR 1.0
        // BaseRenderScale = 800 / (2 × 1) = (400, 400)
        // After resize to DisplaySize 3×2: target = (3×400×1×1, 2×400×1×1) = (1200, 800)
        let result = derive_webview_size(
            Vec2::new(3.0, 2.0),
            Vec2::new(400.0, 400.0),
            1.0,
            1.0,
            UVec2::new(100, 100),
            None,
            UVec2::ZERO,
        );
        assert_eq!(result, Some(UVec2::new(1200, 800)));
    }

    #[test]
    fn derive_clamps_to_min_size() {
        let result = derive_webview_size(
            Vec2::new(0.1, 0.1),
            Vec2::new(100.0, 100.0),
            1.0,
            1.0,
            UVec2::new(100, 100),
            None,
            UVec2::ZERO,
        );
        assert_eq!(result, Some(UVec2::new(100, 100)));
    }

    #[test]
    fn derive_clamps_to_max_size() {
        let result = derive_webview_size(
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 100.0),
            1.0,
            1.0,
            UVec2::new(100, 100),
            Some(UVec2::new(2000, 2000)),
            UVec2::ZERO,
        );
        assert_eq!(result, Some(UVec2::new(2000, 2000)));
    }

    #[test]
    fn derive_rounds_fractional_to_int() {
        // 3.0 × 133.33 = 399.99 → rounds to 400
        let result = derive_webview_size(
            Vec2::new(3.0, 3.0),
            Vec2::new(133.33, 133.33),
            1.0,
            1.0,
            UVec2::new(1, 1),
            None,
            UVec2::ZERO,
        );
        assert_eq!(result, Some(UVec2::new(400, 400)));
    }

    #[test]
    fn derive_dedupes_on_integer_change() {
        // Same as current → returns None
        let result = derive_webview_size(
            Vec2::new(2.0, 2.0),
            Vec2::new(400.0, 400.0),
            1.0,
            1.0,
            UVec2::new(100, 100),
            None,
            UVec2::new(800, 800),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn aspect_lock_dominant_axis_drives_other() {
        // Drag mostly east (+2.0 du, +0.1 dv), aspect = 1:1
        // Width dominant → height follows
        let (new_size, _) = apply_resize(
            ResizeZone::SE,
            Vec2::new(2.0, 2.0),
            Vec3::ZERO,
            2.0,
            0.1,
            Vec3::X,
            Vec3::Y,
            true,  // locked
            Vec2::new(0.1, 0.1),
            None,
        );
        assert!((new_size.x - new_size.y).abs() < 1e-4, "Aspect not locked: {new_size:?}");
    }

    #[test]
    fn aspect_lock_preserves_start_ratio() {
        // Start 4:3, drag east +1.0
        let (new_size, _) = apply_resize(
            ResizeZone::E,
            Vec2::new(4.0, 3.0),
            Vec3::ZERO,
            1.0,
            0.0,
            Vec3::X,
            Vec3::Y,
            true,
            Vec2::new(0.1, 0.1),
            None,
        );
        let expected_aspect = 4.0 / 3.0;
        let actual_aspect = new_size.x / new_size.y;
        assert!(
            (expected_aspect - actual_aspect).abs() < 1e-4,
            "Aspect ratio not preserved: expected {expected_aspect}, got {actual_aspect}"
        );
    }

    #[test]
    fn aspect_lock_respects_min_max_clamp() {
        // Locked aspect, but min clamp kicks in
        let (new_size, _) = apply_resize(
            ResizeZone::W,
            Vec2::new(2.0, 2.0),
            Vec3::ZERO,
            10.0, // shrink a LOT
            0.0,
            Vec3::X,
            Vec3::Y,
            true,
            Vec2::new(1.0, 1.0),
            None,
        );
        assert!(new_size.x >= 1.0 && new_size.y >= 1.0, "Below min: {new_size:?}");
    }
```

- [ ] **Step 3: Run all tests**

Run: `cargo test --workspace --all-features -- resize`
Expected: All tests pass (zone classification + hit-test + anchor math + pipeline + aspect lock).

- [ ] **Step 4: Commit**

```bash
git add src/resize.rs
git commit -m "feat(resize): add derive_webview_size and aspect-lock tests"
```

---

## Task 5: Components — WebviewResizable + Pipeline Components

**Files:**
- Create: `src/resize/components.rs`
- Modify: `src/resize.rs` → convert to `src/resize/mod.rs` (module directory)

- [ ] **Step 1: Convert `src/resize.rs` to module directory**

Move `src/resize.rs` → `src/resize/mod.rs`:

```bash
mkdir -p src/resize
mv src/resize.rs src/resize/mod.rs
```

Add `pub(crate) mod components;` at the top of `src/resize/mod.rs` (after the `use` statements).

- [ ] **Step 2: Create `src/resize/components.rs`**

```rust
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
    /// Width of the invisible resize border, in **texture pixels**.
    /// Default: 16.
    pub edge_thickness: u32,
    /// Minimum texture size in pixels. Default: (100, 100).
    pub min_size: UVec2,
    /// Maximum texture size in pixels. `None` = no cap.
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

/// Window DPR snapshot for this webview. Phase 1: set once at spawn
/// from the `HostWindow` (or `PrimaryWindow` fallback).
/// Phase 2 will update this on monitor transitions.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct WebviewDpr(pub f32);

impl Default for WebviewDpr {
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
    /// World-space direction of local +U (webview width).
    pub u_axis: Vec3,
    /// World-space direction of local +V (webview height).
    pub v_axis: Vec3,
    /// Extent of the webview in local 2D units (width, height), pre-scale.
    pub local_size: Vec2,
}

/// Marker for entities awaiting AABB availability for basis initialization.
/// The derive pipeline skips entities with this marker.
#[derive(Component, Debug, Clone, Copy)]
pub struct PendingBasisInit;
```

- [ ] **Step 3: Run lint check**

Run: `cargo clippy --workspace --all-features`
Expected: No errors (warnings OK at this stage).

- [ ] **Step 4: Commit**

```bash
git add src/resize/
git commit -m "feat(resize): add pipeline components (WebviewResizable, DisplaySize, BaseRenderScale, etc.)"
```

---

## Task 6: Derive Pipeline System

**Files:**
- Create: `src/resize/pipeline.rs`
- Modify: `src/resize/mod.rs` (add `mod pipeline;`)

- [ ] **Step 1: Create `src/resize/pipeline.rs`**

```rust
//! Derive pipeline: WebviewSize = DisplaySize × BaseRenderScale × QualityMultiplier × DPR

use bevy::prelude::*;

use super::components::*;
use super::derive_webview_size;
use crate::common::WebviewSize;

/// Derives `WebviewSize` from pipeline components whenever any input changes.
/// Runs in `WebviewSet::DerivePipeline`.
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
            size.0,
        ) {
            size.0 = new_size;
        }
    }
}

/// For 3D meshes: sync `Transform.scale.xy` from `DisplaySize / WebviewBasis2d.local_size`.
/// Runs in `WebviewSet::DerivePipeline`, after `derive_pipeline_system`.
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
/// Runs in `WebviewSet::DerivePipeline`, after `derive_pipeline_system`.
pub(crate) fn apply_display_to_sprite_system(
    mut webviews: Query<
        (&DisplaySize, &mut Sprite),
        (With<WebviewResizable>, Changed<DisplaySize>, Without<Mesh3d>),
    >,
) {
    for (display, mut sprite) in webviews.iter_mut() {
        sprite.custom_size = Some(display.0);
    }
}
```

- [ ] **Step 2: Add `pub(crate) mod pipeline;` to `src/resize/mod.rs`**

Add after `pub(crate) mod components;`.

- [ ] **Step 3: Run lint check**

Run: `cargo clippy --workspace --all-features`
Expected: No errors. The `WebviewSize` type reference needs to match the actual type in `src/common/components.rs`. Check: if `WebviewSize` wraps `Vec2`, the pipeline needs to convert `UVec2 → Vec2`. Adjust the `derive_pipeline_system` accordingly — if `WebviewSize(Vec2)`, then `size.0 = new_size.as_vec2()`.

- [ ] **Step 4: Commit**

```bash
git add src/resize/pipeline.rs src/resize/mod.rs
git commit -m "feat(resize): add derive pipeline system (WebviewSize from DisplaySize × Scale × DPR)"
```

---

## Task 7: Cursor Override System

**Files:**
- Create: `src/resize/cursor.rs`
- Modify: `src/cursor_icon.rs` (add override priority)
- Modify: `src/resize/mod.rs` (add `mod cursor;`)

- [ ] **Step 1: Create `src/resize/cursor.rs`**

```rust
//! Cursor override for resize edge hover feedback.

use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

use super::ResizeZone;

/// When set, overrides CEF's page cursor (e.g., during resize edge hover).
#[derive(Resource, Default, Debug)]
pub struct SystemCursorOverride(Option<SystemCursorIcon>);

impl SystemCursorOverride {
    pub fn set(&mut self, icon: SystemCursorIcon) {
        self.0 = Some(icon);
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn get(&self) -> Option<SystemCursorIcon> {
        self.0
    }
}

/// Map a resize zone to the appropriate directional cursor.
pub(crate) fn cursor_for_zone(zone: ResizeZone) -> SystemCursorIcon {
    match zone {
        ResizeZone::N | ResizeZone::S => SystemCursorIcon::NsResize,
        ResizeZone::E | ResizeZone::W => SystemCursorIcon::EwResize,
        ResizeZone::NE | ResizeZone::SW => SystemCursorIcon::NeswResize,
        ResizeZone::NW | ResizeZone::SE => SystemCursorIcon::NwseResize,
    }
}
```

- [ ] **Step 2: Modify `src/cursor_icon.rs` to check `SystemCursorOverride`**

Read the current `update_cursor_icon` system (lines 23–35). Add a `SystemCursorOverride` parameter and check it first:

```rust
fn update_cursor_icon(
    receiver: Res<SystemCursorIconReceiver>,
    cursor_override: Res<crate::resize::cursor::SystemCursorOverride>,
    mut commands: Commands,
    windows: Query<Entity, With<Window>>,
) {
    // Override takes priority over CEF cursor.
    if let Some(override_icon) = cursor_override.get() {
        for entity in windows.iter() {
            commands
                .entity(entity)
                .insert(CursorIcon::System(override_icon));
        }
        // Drain the receiver so it doesn't pile up.
        while receiver.0.try_recv().is_ok() {}
        return;
    }

    // Original CEF cursor path.
    while let Ok(cursor_icon) = receiver.0.try_recv() {
        for entity in windows.iter() {
            commands
                .entity(entity)
                .insert(CursorIcon::System(cursor_icon));
        }
    }
}
```

- [ ] **Step 3: Add `pub(crate) mod cursor;` to `src/resize/mod.rs`**

- [ ] **Step 4: Run lint check**

Run: `cargo clippy --workspace --all-features`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src/resize/cursor.rs src/cursor_icon.rs src/resize/mod.rs
git commit -m "feat(resize): add SystemCursorOverride and cursor priority in cursor_icon"
```

---

## Task 8: System Ordering — WebviewSet

**Files:**
- Modify: `src/webview.rs` (introduce `WebviewSet`, move existing systems into sets)
- Modify: `src/resize/mod.rs` (re-export `WebviewSet`)

- [ ] **Step 1: Define `WebviewSet` in `src/webview.rs`**

Add at the top of the file (after use statements):

```rust
/// System ordering for the webview lifecycle.
/// Ensures: ResizeInteraction → DerivePipeline → CreateBrowser → CommitResize
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub enum WebviewSet {
    /// Resize drag tracking writes DisplaySize.
    ResizeInteraction,
    /// Derives WebviewSize from pipeline components.
    DerivePipeline,
    /// Creates CEF browser instances.
    CreateBrowser,
    /// Commits WebviewSize changes to CEF via browsers.resize().
    CommitResize,
}
```

- [ ] **Step 2: Configure set ordering in `WebviewPlugin::build()`**

In the `build()` method, add set configuration before the system registrations:

```rust
app.configure_sets(
    Update,
    (
        WebviewSet::ResizeInteraction,
        WebviewSet::DerivePipeline,
        WebviewSet::CreateBrowser,
        WebviewSet::CommitResize,
    )
        .chain(),
);
```

- [ ] **Step 3: Move existing systems into their sets**

For the non-Windows path (lines ~113–120 in current `build()`):
- `create_webview` → `.in_set(WebviewSet::CreateBrowser)`
- `resize` → `.in_set(WebviewSet::CommitResize)`

For the Windows path (lines ~155–162):
- `create_webview_win` → `.in_set(WebviewSet::CreateBrowser)`
- `resize_win` → `.in_set(WebviewSet::CommitResize)`

- [ ] **Step 4: Run existing examples to verify no regression**

Run: `cargo run --example simple --features debug`
Expected: Webview loads and renders normally. No panic or visible difference.

- [ ] **Step 5: Commit**

```bash
git add src/webview.rs
git commit -m "refactor(webview): introduce WebviewSet system ordering for resize pipeline"
```

---

## Task 9: ResizeState Guard on Existing CEF Input Systems

**Files:**
- Modify: `src/webview/mesh.rs` (add `ResizeState` guard to 4 observer/system functions)
- Modify: `src/webview/webview_sprite.rs` (add `ResizeState` guard to 4 observer/system functions)
- Modify: `src/drag.rs` (rename `DragEndPending` → `InteractionEndPending`)

- [ ] **Step 1: Add `ResizeState` resource**

In `src/resize/mod.rs`, add the `ResizeState` resource (before `#[cfg(test)]`):

```rust
pub(crate) mod components;
pub(crate) mod cursor;
pub(crate) mod pipeline;

use bevy::prelude::*;
use components::AspectLockMode;

/// Global resize routing state.
#[derive(Resource, Debug, Default)]
pub(crate) enum ResizeState {
    #[default]
    Idle,
    Hovering {
        webview: Entity,
        zone: ResizeZone,
    },
    Resizing {
        webview: Entity,
        zone: ResizeZone,
        start_display_size: Vec2,
        start_translation: Vec3,
        start_hit_world: Vec3,
        plane_origin: Vec3,
        plane_normal: Dir3,
        camera: Entity,
        u_axis: Vec3,
        v_axis: Vec3,
        aspect_lock_mode: AspectLockMode,
    },
}

impl ResizeState {
    pub(crate) fn is_resizing(&self) -> bool {
        matches!(self, ResizeState::Resizing { .. })
    }
}
```

- [ ] **Step 2: Add `ResizeState` guard to mesh pointer systems**

In `src/webview/mesh.rs`, for each of `on_pointer_move` (line 71), `on_pointer_pressed` (line 88), `on_pointer_released` (line 104), `on_mouse_wheel` (line 122) — and their `_win` variants — add:

```rust
// After the existing drag_state.is_dragging() check:
if resize_state.is_resizing() {
    return;
}
```

Add `resize_state: Res<crate::resize::ResizeState>` to each function signature.

Repeat for `_win` variants.

- [ ] **Step 3: Add `ResizeState` guard to sprite pointer systems**

Same pattern in `src/webview/webview_sprite.rs`: `apply_on_pointer_move` (line 82), `apply_on_pointer_pressed` (line 99), `apply_on_pointer_released` (line 116), `on_mouse_wheel` (line 134) — and their `_win` variants.

- [ ] **Step 4: Rename `DragEndPending` → `InteractionEndPending`**

In `src/drag.rs`:
- Rename `DragEndPending` (line 60) → `InteractionEndPending`
- Update all references: `DragPlugin::build()` (line 18), `drag_tracking_system` (line 250), `restore_hover_after_drag` (line 280)

- [ ] **Step 5: Run lint + tests**

Run: `cargo clippy --workspace --all-features && cargo test --workspace --all-features`
Expected: All pass. Existing drag tests still pass.

- [ ] **Step 6: Commit**

```bash
git add src/webview/mesh.rs src/webview/webview_sprite.rs src/drag.rs src/resize/mod.rs
git commit -m "feat(resize): add ResizeState guard to all CEF input paths, rename DragEndPending"
```

---

## Task 10: Resize Plugin + On-Add Hook + Unified Observer

**Files:**
- Create: `src/resize/plugin.rs`
- Modify: `src/resize/mod.rs` (add `mod plugin;`)
- Modify: `src/lib.rs` (add `ResizePlugin`)
- Modify: `src/drag.rs` (merge `attach_drag_observers` into unified observer)

This is the largest task. It wires everything together.

- [ ] **Step 1: Create `src/resize/plugin.rs`**

```rust
//! Resize plugin: wires the derive pipeline, resize state machine, cursor override,
//! and unified pointer observer into the Bevy app.

use bevy::prelude::*;

use super::components::*;
use super::cursor::SystemCursorOverride;
use super::pipeline::*;
use super::*;
use crate::common::{WebviewSize, WebviewSource};
use crate::drag::{DragState, DraggableRegions, InteractionEndPending};
use crate::system_param::pointer::WebviewPointer;
use crate::webview::WebviewSet;

pub struct ResizePlugin;

impl Plugin for ResizePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResizeState>()
            .init_resource::<SystemCursorOverride>()
            .add_systems(
                Update,
                (resize_tracking_system, cursor_hover_system)
                    .in_set(WebviewSet::ResizeInteraction),
            )
            .add_systems(
                Update,
                (
                    pending_basis_init_system,
                    derive_pipeline_system,
                    (apply_display_to_mesh_system, apply_display_to_sprite_system),
                )
                    .chain()
                    .in_set(WebviewSet::DerivePipeline),
            )
            .add_systems(Update, attach_webview_observers);
    }
}

/// Attach unified pointer-press observer to newly created webviews.
/// Replaces the old `attach_drag_observers` for webviews that have
/// `WebviewResizable`; non-resizable webviews still get the drag-only observer.
fn attach_webview_observers(
    mut commands: Commands,
    webviews: Query<
        (Entity, Has<WebviewResizable>),
        (
            Added<WebviewSource>,
            With<Transform>,
            Or<(With<Mesh3d>, With<Mesh2d>, With<Sprite>)>,
        ),
    >,
) {
    for (entity, has_resizable) in webviews.iter() {
        if has_resizable {
            commands.entity(entity).observe(on_webview_press);
        }
        // Non-resizable webviews keep the existing drag-only observer
        // (attached by the drag plugin's attach_drag_observers system).
    }
}

/// Unified pointer-press observer for resizable webviews.
/// Routes to resize, drag, or pass-through based on hit classification.
#[allow(clippy::too_many_arguments)]
fn on_webview_press(
    trigger: On<Pointer<Press>>,
    mut drag_state: ResMut<DragState>,
    mut resize_state: ResMut<ResizeState>,
    mut commands: Commands,
    pointer: WebviewPointer,
    regions_q: Query<&DraggableRegions>,
    resizable_q: Query<(&WebviewResizable, &WebviewSize, &DisplaySize)>,
    transforms_q: Query<(&GlobalTransform, &Transform), With<WebviewSource>>,
    cameras_q: Query<(&Camera, &GlobalTransform)>,
    basis_q: Query<&WebviewBasis2d>,
    #[cfg(not(target_os = "windows"))] browsers: NonSend<bevy_cef_core::prelude::Browsers>,
    #[cfg(target_os = "windows")] browsers: Res<bevy_cef_core::prelude::BrowsersProxy>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }

    let Some((webview, pixel_pos, camera_entity)) = pointer.pos_from_trigger_raw(&trigger) else {
        return;
    };

    let regions = regions_q.get(webview).ok();
    let resizable = resizable_q.get(webview).ok();

    let resize_frame = resizable.map(|(r, size, _)| ResizeFrame {
        width: size.0.x as u32,
        height: size.0.y as u32,
        edge_thickness: r.edge_thickness,
    });

    let empty_regions = DraggableRegions::default();
    let hit = classify_hit(
        regions.unwrap_or(&empty_regions),
        resize_frame.as_ref(),
        pixel_pos,
    );

    match hit {
        HitResult::Resize(zone) => {
            let Ok((webview_gtf, webview_tf)) = transforms_q.get(webview) else {
                return;
            };
            let Ok((cam, cam_gtf)) = cameras_q.get(camera_entity) else {
                return;
            };
            let Ok((resizable, _, display)) = resizable_q.get(webview) else {
                return;
            };

            let viewport_pos = trigger.pointer_location.position;
            let Ok(ray) = cam.viewport_to_world(cam_gtf, viewport_pos) else {
                return;
            };
            let plane_origin = webview_gtf.translation();
            let plane_normal = webview_gtf.forward();
            let Some(t) =
                ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))
            else {
                return;
            };
            let start_hit = ray.origin + ray.direction * t;

            // Determine u/v axes.
            let (u_axis, v_axis) = if let Ok(basis) = basis_q.get(webview) {
                (basis.u_axis, basis.v_axis)
            } else {
                // Sprite fallback.
                (Vec3::X, Vec3::Y)
            };

            *resize_state = ResizeState::Resizing {
                webview,
                zone,
                start_display_size: display.0,
                start_translation: webview_tf.translation,
                start_hit_world: start_hit,
                plane_origin,
                plane_normal,
                camera: camera_entity,
                u_axis,
                v_axis,
                aspect_lock_mode: resizable.aspect_lock,
            };

            // Clear CEF hover.
            #[cfg(not(target_os = "windows"))]
            browsers.send_mouse_move(
                &webview,
                std::iter::empty::<&MouseButton>(),
                pixel_pos,
                true,
            );
            #[cfg(target_os = "windows")]
            browsers.send_mouse_move(&webview, &[], pixel_pos, true);
        }
        HitResult::Drag => {
            // Delegate to existing drag logic. The drag plugin's observer
            // handles the actual DraggingState insertion. Since this observer
            // fires on the same event, we need to replicate the drag start
            // logic here for resizable webviews (whose drag observer was
            // NOT attached).
            crate::drag::start_drag_from_press(
                &trigger,
                webview,
                pixel_pos,
                camera_entity,
                &mut drag_state,
                &mut commands,
                &transforms_q,
                &cameras_q,
                &browsers,
            );
        }
        HitResult::None => {
            // Pass through to CEF page input — handled by existing observers.
        }
    }
}

/// Resize tracking: updates DisplaySize + Transform each frame during drag.
#[allow(clippy::too_many_arguments)]
fn resize_tracking_system(
    mut resize_state: ResMut<ResizeState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut webviews: Query<(
        &mut Transform,
        &mut DisplaySize,
        &WebviewResizable,
        &BaseRenderScale,
        &QualityMultiplier,
        &WebviewDpr,
    )>,
    mut pending: ResMut<InteractionEndPending>,
) {
    let ResizeState::Resizing {
        webview,
        zone,
        start_display_size,
        start_translation,
        start_hit_world,
        plane_origin,
        plane_normal,
        camera,
        u_axis,
        v_axis,
        aspect_lock_mode,
    } = *resize_state
    else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        *resize_state = ResizeState::Idle;
        pending.webview = Some(webview);
        return;
    }

    let Some(cursor) = windows.iter().find_map(|w| w.cursor_position()) else {
        return;
    };
    let Ok((cam, cam_gtf)) = cameras.get(camera) else {
        return;
    };
    let Ok(ray) = cam.viewport_to_world(cam_gtf, cursor) else {
        return;
    };
    let Some(t) = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal)) else {
        return;
    };
    let current_hit = ray.origin + ray.direction * t;
    let delta_world = current_hit - start_hit_world;

    let du = delta_world.dot(u_axis);
    let dv = delta_world.dot(v_axis);

    let Ok((_, _, resizable, base, quality, dpr)) = webviews.get(webview) else {
        return;
    };

    let lock = match aspect_lock_mode {
        AspectLockMode::Always => true,
        AspectLockMode::Never => false,
        AspectLockMode::LockOnShift => {
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
        }
    };

    // Convert min/max from texture pixels to display-size units.
    let scale_factor = base.0 * quality.0 * dpr.0;
    let min_display = Vec2::new(
        resizable.min_size.x as f32 / scale_factor.x,
        resizable.min_size.y as f32 / scale_factor.y,
    );
    let max_display = resizable.max_size.map(|m| {
        Vec2::new(m.x as f32 / scale_factor.x, m.y as f32 / scale_factor.y)
    });

    let (new_size, new_tr) = apply_resize(
        zone,
        start_display_size,
        start_translation,
        du,
        dv,
        u_axis,
        v_axis,
        lock,
        min_display,
        max_display,
    );

    let Ok((mut tf, mut display, ..)) = webviews.get_mut(webview) else {
        return;
    };
    display.0 = new_size;
    tf.translation = new_tr;
}

/// Cursor hover: classifies the pointer's texture-pixel position
/// and sets the appropriate resize cursor.
fn cursor_hover_system(
    resize_state: Res<ResizeState>,
    pointer: WebviewPointer,
    windows: Query<&Window>,
    resizables: Query<(Entity, &WebviewResizable, &WebviewSize, &Sprite, &crate::common::WebviewSize, &GlobalTransform), With<WebviewResizable>>,
    // Note: The actual implementation will need separate mesh/sprite hover detection.
    // This is a placeholder signature — the implementer should use the mesh path
    // (WebviewPointer + pointer_pos_raw) and sprite path (obtain_relative_pos)
    // as described in spec §4.4, then feed both into classify_zone.
    mut cursor_override: ResMut<SystemCursorOverride>,
) {
    // Implementation follows spec §4.4 — two paths:
    // 1. Mesh: WebviewPointer<Camera3d> + pointer_pos_raw for each resizable mesh
    // 2. Sprite: obtain_relative_pos with window cursor position
    // Both produce Option<(Entity, Vec2)> → classify_zone → set cursor override.
    //
    // The implementer should refer to the spec for the full logic.
    // This function body is intentionally left for the implementer to complete
    // based on the available pointer APIs, as the exact API shape depends on
    // what's accessible from the system parameters.
    todo!("Implement cursor hover per spec §4.4 — mesh + sprite dual-path detection")
}

/// Retry basis initialization for entities whose mesh AABB wasn't ready at on_add.
fn pending_basis_init_system(
    mut commands: Commands,
    pending: Query<(Entity, &WebviewResizable), With<PendingBasisInit>>,
    meshes_3d: Query<(&Mesh3d, &GlobalTransform, &WebviewSize)>,
    mesh_assets: Res<Assets<Mesh>>,
    windows: Query<&Window>,
) {
    for (entity, _) in pending.iter() {
        if let Ok((mesh3d, gtf, webview_size)) = meshes_3d.get(entity) {
            if let Some(mesh) = mesh_assets.get(&mesh3d.0) {
                if let Some(aabb) = mesh.compute_aabb() {
                    let local_size = Vec2::new(
                        aabb.half_extents.x * 2.0,
                        aabb.half_extents.y * 2.0,
                    );
                    let dpr = windows
                        .iter()
                        .next()
                        .map(|w| w.scale_factor())
                        .unwrap_or(1.0);

                    let world_size = local_size * gtf.compute_transform().scale.truncate();
                    let base = Vec2::new(
                        webview_size.0.x as f32 / (world_size.x * dpr),
                        webview_size.0.y as f32 / (world_size.y * dpr),
                    );

                    commands.entity(entity).insert((
                        DisplaySize(world_size),
                        BaseRenderScale(base),
                        QualityMultiplier::default(),
                        WebviewDpr(dpr),
                        WebviewBasis2d {
                            u_axis: Vec3::X,
                            v_axis: Vec3::Y,
                            local_size,
                        },
                    ));
                    commands.entity(entity).remove::<PendingBasisInit>();
                }
            }
        }
    }
}
```

**Important note for implementer:** The `cursor_hover_system` has a `todo!()` because the exact system parameter signatures depend on the runtime pointer API. The implementer should:
1. For mesh hover: iterate resizable mesh entities, use `WebviewPointer::pointer_pos_raw()` with the window cursor.
2. For sprite hover: use `obtain_relative_pos()` from `webview_sprite.rs`.
3. Feed both into `classify_zone()` → `cursor_for_zone()` → `cursor_override.set()`.

- [ ] **Step 2: Extract `start_drag_from_press` in `src/drag.rs`**

The `on_webview_press` observer in the resize plugin needs to start a drag when the hit is in a drag region. Extract the drag-start logic from `on_drag_press` (lines 174–211) into a public `start_drag_from_press()` function that both observers can call.

- [ ] **Step 3: Modify `src/drag.rs` `attach_drag_observers`**

Change the query in `attach_drag_observers` (line 217) to exclude entities that have `WebviewResizable`:

```rust
fn attach_drag_observers(
    mut commands: Commands,
    webviews: Query<
        Entity,
        (
            Added<WebviewSource>,
            With<Transform>,
            Or<(With<Mesh3d>, With<Mesh2d>)>,
            Without<crate::resize::components::WebviewResizable>,
        ),
    >,
) {
    for entity in webviews.iter() {
        commands.entity(entity).observe(on_drag_press);
    }
}
```

- [ ] **Step 4: Add `ResizePlugin` to `CefPlugin::build()` in `src/lib.rs`**

Add after `DragPlugin` (line 55):

```rust
app.add_plugins(crate::resize::plugin::ResizePlugin);
```

Also add to the prelude (line 26):

```rust
pub use crate::resize::components::{
    AspectLockMode, BaseRenderScale, DisplaySize, QualityMultiplier, WebviewBasis2d,
    WebviewDpr, WebviewResizable,
};
```

- [ ] **Step 5: Run lint**

Run: `cargo clippy --workspace --all-features`
Expected: Warnings about the `todo!()` in cursor_hover_system, but no errors.

- [ ] **Step 6: Commit**

```bash
git add src/resize/ src/drag.rs src/lib.rs
git commit -m "feat(resize): add ResizePlugin with state machine, tracking, and unified observer"
```

---

## Task 11: Example + Demo HTML

**Files:**
- Create: `examples/resize.rs`
- Create: `assets/resize_demo.html`

- [ ] **Step 1: Create `assets/resize_demo.html`**

```html
<!doctype html>
<html>
<head>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    background: linear-gradient(135deg, #1a5276, #2ecc71);
    color: white;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    display: grid;
    place-items: center;
    height: 100vh;
    overflow: hidden;
  }
  .title-bar {
    -webkit-app-region: drag;
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 32px;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    padding-left: 12px;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.7);
  }
  .content {
    text-align: center;
  }
  .size {
    font-size: 2.5em;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .hint {
    margin-top: 8px;
    font-size: 0.9em;
    opacity: 0.7;
  }
</style>
</head>
<body>
  <div class="title-bar">Drag to move</div>
  <div class="content">
    <div class="size" id="s">&mdash;</div>
    <div class="hint">Drag edges to resize &middot; Shift locks aspect</div>
  </div>
  <script>
    const s = document.getElementById('s');
    function update() {
      s.textContent = window.innerWidth + ' \u00d7 ' + window.innerHeight;
    }
    window.addEventListener('resize', update);
    update();
  </script>
</body>
</html>
```

- [ ] **Step 2: Create `examples/resize.rs`**

```rust
//! Resize example: drag edges/corners to resize webviews.
//! Shows a 3D mesh webview and a reference plane.

use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, (spawn_scene, spawn_webview))
        .run();
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 0., 4.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(1., 1., 1.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // Reference plane so the user can see the resize motion.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(6.0, 6.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.25, 0.35))),
        Transform::from_translation(Vec3::new(0.0, -1.5, -1.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::local("resize_demo.html"),
        WebviewSize(UVec2::new(800, 600)),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
        Transform::from_translation(Vec3::ZERO),
        WebviewResizable::default(),
    ));
}
```

**Note for implementer:** If `WebviewSize` wraps `Vec2` (not `UVec2`), adjust the example to use `Vec2::new(800.0, 600.0)`. Check the actual type in `src/common/components.rs`.

- [ ] **Step 3: Run the example**

Run: `cargo run --example resize --features debug`
Expected: A green gradient webview renders on a 3D plane. The title bar says "Drag to move". The center shows the live dimensions. Resize edges should work if the full resize plugin is wired.

- [ ] **Step 4: Commit**

```bash
git add examples/resize.rs assets/resize_demo.html
git commit -m "feat(resize): add resize example with 3D mesh webview"
```

---

## Task 12: Implement cursor_hover_system

**Files:**
- Modify: `src/resize/plugin.rs` (replace `todo!()` in `cursor_hover_system`)

This task fills in the cursor hover detection that was left as `todo!()`. The implementer needs to:

- [ ] **Step 1: Implement mesh hover path**

For each resizable mesh entity, use `WebviewPointer<Camera3d>::pointer_pos_raw(entity, cursor_pos)` to get the texture-pixel position under the current cursor.

- [ ] **Step 2: Implement sprite hover path**

For each resizable sprite entity, use the sprite pointer mapping (adapted from `obtain_relative_pos` in `src/webview/webview_sprite.rs`) to get the texture-pixel position.

- [ ] **Step 3: Feed into `classify_zone` + `cursor_for_zone`**

Whichever path finds a resizable webview under the cursor: classify the zone, set the cursor override. If no webview is under the cursor, clear the override.

- [ ] **Step 4: Test manually**

Run: `cargo run --example resize --features debug`
Expected: Cursor changes to directional resize cursor when hovering edges/corners.

- [ ] **Step 5: Commit**

```bash
git add src/resize/plugin.rs
git commit -m "feat(resize): implement cursor hover system for edge detection"
```

---

## Task 13: On-Add Hook for WebviewResizable

**Files:**
- Modify: `src/resize/plugin.rs` or `src/webview.rs` (add component hook)

When `WebviewResizable` is added to an entity, the pipeline components must be initialized from the current mesh/sprite state.

- [ ] **Step 1: Register `on_add` hook for `WebviewResizable`**

In `ResizePlugin::build()`, register a component hook:

```rust
app.register_component_hooks::<WebviewResizable>(ComponentHooks::new().on_add(
    |mut world: DeferredWorld, entity: Entity, _component_id: ComponentId| {
        // Snapshot pipeline components from current state.
        // Read mesh AABB, sprite custom_size, window DPR, current WebviewSize.
        // Insert DisplaySize, BaseRenderScale, QualityMultiplier, WebviewDpr,
        // WebviewBasis2d (mesh only), or PendingBasisInit if AABB not ready.
    },
));
```

- [ ] **Step 2: Implement the hook body**

The hook must:
1. Read the entity's `WebviewSize`, `Transform`, `GlobalTransform`.
2. If it has `Mesh3d`: read the mesh asset, compute AABB, derive `local_size`. If AABB unavailable, insert `PendingBasisInit`.
3. If it has `Sprite`: read `custom_size` (or set it from WebviewSize if missing).
4. Read `Window.scale_factor()` from the `HostWindow` or `PrimaryWindow`.
5. Compute `BaseRenderScale = WebviewSize / (display_size × dpr)`.
6. Debug-assert: centered origin, Z-normal (mesh only), `min_size > 2 × edge_thickness`.

- [ ] **Step 3: Test with the resize example**

Run: `cargo run --example resize --features debug`
Expected: No panic at startup. `DisplaySize` and `BaseRenderScale` are correctly initialized (verify with `bevy-inspector-egui` or debug logging).

- [ ] **Step 4: Commit**

```bash
git add src/resize/
git commit -m "feat(resize): add WebviewResizable on_add hook for pipeline initialization"
```

---

## Task 14: Final Integration — Lint, Tests, Manual Verification

**Files:**
- All modified files

- [ ] **Step 1: Run full lint**

Run: `make fix-lint`
Expected: No warnings.

- [ ] **Step 2: Run all unit tests**

Run: `cargo test --workspace --all-features`
Expected: All tests pass, including:
- Zone classification tests (6)
- Hit-test priority tests (3)
- Anchor math tests (8)
- Pipeline derivation tests (6)
- Aspect lock tests (3)
- Existing drag tests (all pass unchanged)

- [ ] **Step 3: Run existing examples for regression**

Run each of these and confirm no visible change:
```bash
cargo run --example simple --features debug
cargo run --example sprite --features debug
cargo run --example toolbar_drag --features debug
```

- [ ] **Step 4: Run the resize example for full manual test**

Run: `cargo run --example resize --features debug`

Walk through the manual test checklist from spec §6.3:
- [ ] Drag each of the 8 zones; opposite edge/corner stays pinned
- [ ] Cursor changes to correct direction on each edge hover
- [ ] Page's `window.innerWidth`/`innerHeight` updates live during drag
- [ ] Text stays readable after resize (not pixelated)
- [ ] Drag title bar still moves the webview
- [ ] Shift + drag locks aspect ratio
- [ ] `min_size`/`max_size` stop the resize at the limits
- [ ] Existing examples unchanged

- [ ] **Step 5: Commit any final fixes**

```bash
git add -u
git commit -m "chore: final lint and integration fixes for resize feature"
```
