//! Drag plugin: provides the async_channel plumbing that delivers
//! CEF `OnDraggableRegionsChanged` callbacks to Bevy ECS.

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_cef_core::prelude::{DraggableRegion, DraggableRegionSenderInner};

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();
        app.insert_resource(DraggableRegionSender(tx))
            .insert_resource(DraggableRegionReceiver(rx));
    }
}

#[derive(Resource, Debug, Deref)]
pub(crate) struct DraggableRegionSender(pub(crate) DraggableRegionSenderInner);

#[derive(Resource, Debug)]
pub(crate) struct DraggableRegionReceiver(pub(crate) Receiver<(Entity, Vec<DraggableRegion>)>);

/// A rectangle in webview texture-pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PixelRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl PixelRect {
    pub(crate) fn contains(&self, pos: Vec2) -> bool {
        pos.x >= self.min.x && pos.x <= self.max.x && pos.y >= self.min.y && pos.y <= self.max.y
    }
}

/// Returns `true` if `pos` is inside any drag rect but NOT inside any no-drag rect.
/// No-drag regions act as "holes" in drag regions (e.g. a button inside a toolbar).
pub(crate) fn is_draggable(
    drag_rects: &[PixelRect],
    no_drag_rects: &[PixelRect],
    pos: Vec2,
) -> bool {
    // no-drag (holes) takes priority
    if no_drag_rects.iter().any(|r| r.contains(pos)) {
        return false;
    }
    drag_rects.iter().any(|r| r.contains(pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> PixelRect {
        PixelRect {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        }
    }

    #[test]
    fn contains_inside() {
        let r = rect(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(Vec2::new(50.0, 40.0)));
    }

    #[test]
    fn contains_outside() {
        let r = rect(10.0, 20.0, 100.0, 50.0);
        assert!(!r.contains(Vec2::new(5.0, 40.0)));
        assert!(!r.contains(Vec2::new(150.0, 40.0)));
        assert!(!r.contains(Vec2::new(50.0, 10.0)));
        assert!(!r.contains(Vec2::new(50.0, 80.0)));
    }

    #[test]
    fn contains_boundary() {
        let r = rect(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(Vec2::new(10.0, 20.0))); // min corner
        assert!(r.contains(Vec2::new(110.0, 70.0))); // max corner
    }

    #[test]
    fn is_draggable_hit_drag() {
        let drags = vec![rect(0.0, 0.0, 800.0, 40.0)];
        let no_drags = vec![];
        assert!(is_draggable(&drags, &no_drags, Vec2::new(100.0, 20.0)));
    }

    #[test]
    fn is_draggable_miss_drag() {
        let drags = vec![rect(0.0, 0.0, 800.0, 40.0)];
        let no_drags = vec![];
        assert!(!is_draggable(&drags, &no_drags, Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn is_draggable_no_drag_hole_inside_drag() {
        // A toolbar with a button hole
        let drags = vec![rect(0.0, 0.0, 800.0, 40.0)];
        let no_drags = vec![rect(750.0, 5.0, 40.0, 30.0)]; // close button
        // Hit the toolbar area - draggable
        assert!(is_draggable(&drags, &no_drags, Vec2::new(100.0, 20.0)));
        // Hit the button - NOT draggable (hole)
        assert!(!is_draggable(&drags, &no_drags, Vec2::new(770.0, 20.0)));
    }

    #[test]
    fn is_draggable_empty_regions() {
        assert!(!is_draggable(&[], &[], Vec2::new(50.0, 50.0)));
    }
}
