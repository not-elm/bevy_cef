//! Resize feature: drag-to-resize webviews with automatic edge detection.

use bevy::prelude::*;
use crate::drag::{is_draggable, DraggableRegions};

/// One of the 8 resize zones around a webview's edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeZone {
    N, S, E, W, NE, NW, SE, SW,
}

/// Describes the resize-sensitive frame around a webview in texture-pixel space.
#[derive(Debug, Clone, Copy)]
pub struct ResizeFrame {
    pub width: u32,
    pub height: u32,
    pub edge_thickness: u32,
}

impl ResizeFrame {
    /// Classify a texture-pixel position into a resize zone, or `None` if interior.
    /// Corners win over edges when both axes are in the border.
    pub fn classify(&self, pos: Vec2) -> Option<ResizeZone> {
        let w = self.width as f32;
        let h = self.height as f32;
        let t = self.edge_thickness as f32;

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
            _ => None,
        }
    }
}

/// Result of classifying a pointer position on a webview.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HitResult {
    Resize(ResizeZone),
    Drag,
    None,
}

/// Compute new display size and translation after a resize drag.
///
/// The pinned-corner rule: dragging one edge/corner keeps the opposite
/// edge/corner fixed in world space. Assumes centered origin.
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

    if lock_aspect {
        let aspect = start_size.x / start_size.y;
        if dw_raw.abs() * (1.0 / aspect) > dh_raw.abs() {
            new_size.y = new_size.x / aspect;
        } else {
            new_size.x = new_size.y * aspect;
        }
    }

    new_size = new_size.max(min_size);
    if let Some(max) = max_size {
        new_size = new_size.min(max);
    }

    let actual_dw = new_size.x - start_size.x;
    let actual_dh = new_size.y - start_size.y;

    let sign_u = match zone {
        ResizeZone::E | ResizeZone::NE | ResizeZone::SE => 0.5,
        ResizeZone::W | ResizeZone::NW | ResizeZone::SW => -0.5,
        _ => 0.0,
    };
    let sign_v = match zone {
        ResizeZone::N | ResizeZone::NE | ResizeZone::NW => 0.5,
        ResizeZone::S | ResizeZone::SE | ResizeZone::SW => -0.5,
        _ => 0.0,
    };
    let new_translation =
        start_translation + u_axis * (actual_dw * sign_u) + v_axis * (actual_dh * sign_v);

    (new_size, new_translation)
}

/// Classify a pointer's texture-pixel position: resize edge > drag region > page input.
pub(crate) fn classify_hit(
    regions: &DraggableRegions,
    frame: Option<&ResizeFrame>,
    pos: Vec2,
) -> HitResult {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drag::{DraggableRegions, PixelRect};

    fn frame(w: u32, h: u32, t: u32) -> ResizeFrame {
        ResizeFrame { width: w, height: h, edge_thickness: t }
    }

    #[test]
    fn classify_zone_interior_returns_none() {
        let f = frame(800, 600, 16);
        assert_eq!(f.classify(Vec2::new(400.0, 300.0)), None);
        assert_eq!(f.classify(Vec2::new(16.0, 16.0)), None);
    }

    #[test]
    fn classify_zone_north_edge() {
        let f = frame(800, 600, 16);
        assert_eq!(f.classify(Vec2::new(400.0, 8.0)), Some(ResizeZone::N));
    }

    #[test]
    fn classify_zone_nw_corner_wins_over_n_and_w() {
        let f = frame(800, 600, 16);
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
        let f = frame(800, 600, 32);
        assert_eq!(f.classify(Vec2::new(20.0, 300.0)), Some(ResizeZone::W));
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

    #[test]
    fn resize_edge_wins_over_drag_region_at_boundary() {
        let regions = DraggableRegions {
            drag_rects: vec![PixelRect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(800.0, 40.0),
            }],
            no_drag_rects: vec![],
        };
        let resize_frame = ResizeFrame { width: 800, height: 600, edge_thickness: 16 };
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
        let result = classify_hit(&regions, None, Vec2::new(3.0, 3.0));
        assert_eq!(result, HitResult::Drag);
    }

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
}
