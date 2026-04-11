//! Drag plugin: provides the async_channel plumbing that delivers
//! CEF `OnDraggableRegionsChanged` callbacks to Bevy ECS.

use crate::prelude::WebviewSource;
use crate::system_param::pointer::WebviewPointer;
use async_channel::Receiver;
use bevy::prelude::*;
use bevy_cef_core::prelude::{DraggableRegion, DraggableRegionSenderInner};

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();
        app.insert_resource(DraggableRegionSender(tx))
            .insert_resource(DraggableRegionReceiver(rx))
            .init_resource::<DragState>()
            .init_resource::<InteractionEndPending>()
            .add_systems(PreUpdate, receive_drag_regions)
            .add_systems(
                Update,
                (
                    attach_drag_observers,
                    drag_tracking_system,
                    restore_hover_after_drag,
                )
                    .chain(),
            );
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

/// Per-entity cached drag regions, parsed from CEF OnDraggableRegionsChanged.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct DraggableRegions {
    pub(crate) drag_rects: Vec<PixelRect>, // -webkit-app-region: drag
    pub(crate) no_drag_rects: Vec<PixelRect>, // -webkit-app-region: no-drag (holes)
}

/// Tracks a webview that just finished a drag, waiting one frame to send
/// mouse_leave=false with the correct (post-drag) GlobalTransform.
#[derive(Resource, Default)]
pub(crate) struct InteractionEndPending {
    pub(crate) webview: Option<Entity>,
}

/// Global drag routing state — single source of truth for "is drag active?"
#[derive(Resource, Debug, Default)]
pub(crate) enum DragState {
    #[default]
    Idle,
    Dragging {
        webview: Entity,
    },
}

impl DragState {
    pub(crate) fn is_dragging(&self) -> bool {
        matches!(self, DragState::Dragging { .. })
    }

    pub(crate) fn dragging_entity(&self) -> Option<Entity> {
        match self {
            DragState::Dragging { webview } => Some(*webview),
            DragState::Idle => None,
        }
    }
}

/// Per-drag coordinate computation state, inserted on drag start, removed on drag end.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct DraggingState {
    pub(crate) camera: Entity,
    pub(crate) start_hit: Vec3,
    pub(crate) start_translation: Vec3,
    pub(crate) plane_normal: Dir3,
    pub(crate) plane_origin: Vec3,
}

/// Convert a slice of CEF `DraggableRegion`s into drag_rects + no_drag_rects split by the `draggable` flag.
/// This matches the CEF behavior where both drag (=1) and no-drag (=0) regions are reported separately.
pub(crate) fn convert_draggable_regions(regions: &[DraggableRegion]) -> DraggableRegions {
    let mut drag_rects = Vec::new();
    let mut no_drag_rects = Vec::new();
    for r in regions {
        let rect = PixelRect {
            min: Vec2::new(r.bounds.x as f32, r.bounds.y as f32),
            max: Vec2::new(
                (r.bounds.x + r.bounds.width) as f32,
                (r.bounds.y + r.bounds.height) as f32,
            ),
        };
        if r.draggable != 0 {
            drag_rects.push(rect);
        } else {
            no_drag_rects.push(rect);
        }
    }
    DraggableRegions {
        drag_rects,
        no_drag_rects,
    }
}

fn receive_drag_regions(mut commands: Commands, receiver: Res<DraggableRegionReceiver>) {
    while let Ok((entity, regions)) = receiver.0.try_recv() {
        let regions_component = convert_draggable_regions(&regions);
        commands.entity(entity).try_insert(regions_component);
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

/// Observer that fires on Pointer<Press> and starts a drag if the press is in a drag region.
/// Attached to mesh webview entities via `attach_drag_observers`.
#[allow(clippy::too_many_arguments)]
fn on_drag_press(
    trigger: On<Pointer<Press>>,
    mut drag_state: ResMut<DragState>,
    mut commands: Commands,
    pointer: WebviewPointer,
    regions_q: Query<&DraggableRegions>,
    transforms_q: Query<(&GlobalTransform, &Transform), With<WebviewSource>>,
    cameras_q: Query<(&Camera, &GlobalTransform)>,
    #[cfg(not(target_os = "windows"))] browsers: NonSend<bevy_cef_core::prelude::Browsers>,
    #[cfg(target_os = "windows")] browsers: Res<bevy_cef_core::prelude::BrowsersProxy>,
) {
    // Ignore if already dragging.
    if drag_state.is_dragging() {
        return;
    }

    let Some((webview, pixel_pos, camera_entity)) = pointer.pos_from_trigger_raw(&trigger) else {
        return;
    };
    let Ok(regions) = regions_q.get(webview) else {
        return;
    };

    if !is_draggable(&regions.drag_rects, &regions.no_drag_rects, pixel_pos) {
        return;
    }

    // Hit — start drag. Use the camera that produced the hit for the plane snapshot.
    let Ok((webview_gtf, webview_tf)) = transforms_q.get(webview) else {
        return;
    };
    let Ok((cam, cam_gtf)) = cameras_q.get(camera_entity) else {
        return;
    };

    let viewport_pos = trigger.pointer_location.position;
    let Ok(ray) = cam.viewport_to_world(cam_gtf, viewport_pos) else {
        return;
    };
    let plane_origin = webview_gtf.translation();
    let plane_normal = webview_gtf.forward();
    let Some(t) = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal)) else {
        return;
    };
    let start_hit = ray.origin + ray.direction * t;

    *drag_state = DragState::Dragging { webview };
    commands.entity(webview).insert(DraggingState {
        camera: camera_entity,
        start_hit,
        start_translation: webview_tf.translation,
        plane_normal,
        plane_origin,
    });

    // Clear CEF hover state — the webview is being dragged, not hovered.
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

/// Attach drag-press observer to newly-created mesh webviews with a Transform.
fn attach_drag_observers(
    mut commands: Commands,
    webviews: Query<
        Entity,
        (
            Added<WebviewSource>,
            With<Transform>,
            Or<(With<Mesh3d>, With<Mesh2d>)>,
        ),
    >,
) {
    for entity in webviews.iter() {
        commands.entity(entity).observe(on_drag_press);
    }
}

/// Updates the Transform of the dragged webview every frame by raycasting the current
/// cursor position to the snapshotted plane, then applying start_translation + delta.
/// Also detects mouse button release and ends the drag.
fn drag_tracking_system(
    mut drag_state: ResMut<DragState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    windows: Query<&Window>,
    mut webviews: Query<(&mut Transform, &DraggingState)>,
    cameras_q: Query<(&Camera, &GlobalTransform)>,
) {
    let Some(webview) = drag_state.dragging_entity() else {
        return;
    };

    // Release detection: if mouse button is no longer pressed, end the drag.
    if !mouse_button.pressed(MouseButton::Left) {
        *drag_state = DragState::Idle;
        commands.entity(webview).remove::<DraggingState>();
        commands.insert_resource(InteractionEndPending {
            webview: Some(webview),
        });
        return;
    }

    // Position update via raycast to snapshotted plane.
    let Some(cursor) = windows.iter().find_map(|w| w.cursor_position()) else {
        return;
    };
    let Ok((mut tf, ds)) = webviews.get_mut(webview) else {
        return;
    };
    let Ok((cam, cam_gtf)) = cameras_q.get(ds.camera) else {
        return;
    };
    let Ok(ray) = cam.viewport_to_world(cam_gtf, cursor) else {
        return;
    };
    let Some(t) = ray.intersect_plane(ds.plane_origin, InfinitePlane3d::new(ds.plane_normal))
    else {
        return;
    };
    let current_hit = ray.origin + ray.direction * t;
    tf.translation = ds.start_translation + (current_hit - ds.start_hit);
}

/// Runs one frame after drag end to send mouse_leave=false to CEF with the
/// restored (post-drag) cursor-to-texture mapping.
fn restore_hover_after_drag(
    mut pending: ResMut<InteractionEndPending>,
    windows: Query<&Window>,
    pointer: WebviewPointer,
    #[cfg(not(target_os = "windows"))] browsers: NonSend<bevy_cef_core::prelude::Browsers>,
    #[cfg(target_os = "windows")] browsers: Res<bevy_cef_core::prelude::BrowsersProxy>,
) {
    let Some(entity) = pending.webview.take() else {
        return;
    };
    let Some(cursor) = windows.iter().find_map(|w| w.cursor_position()) else {
        return;
    };
    let Some((pos, _cam)) = pointer.pointer_pos_raw(entity, cursor) else {
        return;
    };

    #[cfg(not(target_os = "windows"))]
    browsers.send_mouse_move(&entity, std::iter::empty::<&MouseButton>(), pos, false);
    #[cfg(target_os = "windows")]
    browsers.send_mouse_move(&entity, &[], pos, false);
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

    #[test]
    fn converts_drag_region() {
        use bevy_cef_core::prelude::{DraggableRegion, Rect};
        let input = vec![DraggableRegion {
            bounds: Rect {
                x: 10,
                y: 20,
                width: 100,
                height: 50,
            },
            draggable: 1,
        }];
        let result = convert_draggable_regions(&input);
        assert_eq!(result.drag_rects.len(), 1);
        assert_eq!(result.no_drag_rects.len(), 0);
        assert_eq!(result.drag_rects[0].min, Vec2::new(10.0, 20.0));
        assert_eq!(result.drag_rects[0].max, Vec2::new(110.0, 70.0));
    }

    #[test]
    fn converts_no_drag_region() {
        use bevy_cef_core::prelude::{DraggableRegion, Rect};
        let input = vec![DraggableRegion {
            bounds: Rect {
                x: 0,
                y: 0,
                width: 50,
                height: 50,
            },
            draggable: 0,
        }];
        let result = convert_draggable_regions(&input);
        assert_eq!(result.drag_rects.len(), 0);
        assert_eq!(result.no_drag_rects.len(), 1);
    }

    #[test]
    fn converts_mixed() {
        use bevy_cef_core::prelude::{DraggableRegion, Rect};
        let input = vec![
            DraggableRegion {
                bounds: Rect {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 40,
                },
                draggable: 1,
            },
            DraggableRegion {
                bounds: Rect {
                    x: 750,
                    y: 5,
                    width: 40,
                    height: 30,
                },
                draggable: 0,
            },
        ];
        let result = convert_draggable_regions(&input);
        assert_eq!(result.drag_rects.len(), 1);
        assert_eq!(result.no_drag_rects.len(), 1);
    }

    #[test]
    fn converts_empty() {
        use bevy_cef_core::prelude::DraggableRegion;
        let input: Vec<DraggableRegion> = vec![];
        let result = convert_draggable_regions(&input);
        assert_eq!(result.drag_rects.len(), 0);
        assert_eq!(result.no_drag_rects.len(), 0);
    }
}
