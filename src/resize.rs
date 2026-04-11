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
}
