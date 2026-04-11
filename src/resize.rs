//! Resize feature: drag-to-resize webviews with automatic edge detection.

use bevy::prelude::*;

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
}
