//! Routes pointer + wheel input from a `MaterialNode<WebviewUiMaterial>` node to
//! its CEF webview, mirroring the mesh path but sourcing the position from the
//! node's `RelativeCursorPosition` instead of a raycast.

use crate::prelude::{WebviewSize, WebviewSource, WebviewSurface};
use crate::webview::alpha::is_pixel_transparent;
use crate::webview::ui::material::WebviewUiMaterial;
use bevy::input::mouse::MouseScrollUnit;
use bevy::picking::events::Scroll;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;

/// Attaches the UI input observers (and a `RelativeCursorPosition`) to each UI
/// webview node so the user only needs to spawn
/// `MaterialNode<WebviewUiMaterial> + WebviewSource`.
///
/// Fires in whichever frame the second of the two defining components arrives,
/// so a node that gains `WebviewSource` and `MaterialNode<WebviewUiMaterial>` on
/// different frames is still wired exactly once.
#[cfg(not(target_os = "windows"))]
pub(super) fn setup_ui_observers(
    mut commands: Commands,
    webviews: Query<
        Entity,
        (
            Or<(Added<WebviewSource>, Added<MaterialNode<WebviewUiMaterial>>)>,
            With<WebviewSource>,
            With<MaterialNode<WebviewUiMaterial>>,
        ),
    >,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .insert(RelativeCursorPosition::default())
            .observe(on_ui_pointer_move)
            .observe(on_ui_pointer_pressed)
            .observe(on_ui_pointer_released)
            .observe(on_ui_pointer_scroll);
    }
}

/// Windows variant of `setup_ui_observers` attaching the proxy-based observers.
#[cfg(target_os = "windows")]
pub(super) fn setup_ui_observers_win(
    mut commands: Commands,
    webviews: Query<
        Entity,
        (
            Or<(Added<WebviewSource>, Added<MaterialNode<WebviewUiMaterial>>)>,
            With<WebviewSource>,
            With<MaterialNode<WebviewUiMaterial>>,
        ),
    >,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .insert(RelativeCursorPosition::default())
            .observe(on_ui_pointer_move_win)
            .observe(on_ui_pointer_pressed_win)
            .observe(on_ui_pointer_released_win)
            .observe(on_ui_pointer_scroll_win);
    }
}

/// Converts a center-origin `RelativeCursorPosition.normalized` (`(0,0)` center,
/// `(-0.5,-0.5)` top-left) into a top-left-origin DIP coordinate scaled to the
/// webview's logical size.
fn ui_pos_to_dip(normalized: Vec2, computed_size: Vec2, inverse_scale_factor: f32) -> Vec2 {
    (normalized + Vec2::splat(0.5)) * computed_size * inverse_scale_factor
}

/// Converts a `Pointer<Scroll>` delta into the pixel deltas CEF expects.
/// Chromium's default line height is 3 lines × 40px = 120px per notch.
fn scroll_delta(unit: MouseScrollUnit, x: f32, y: f32) -> Vec2 {
    match unit {
        MouseScrollUnit::Line => Vec2::new(x * 120.0, y * 120.0),
        MouseScrollUnit::Pixel => Vec2::new(x, y),
    }
}

/// Components every UI input handler reads off the observed webview node.
type UiNode<'a> = (
    &'a RelativeCursorPosition,
    &'a ComputedNode,
    Option<&'a WebviewSurface>,
    &'a WebviewSize,
);

/// Resolves the DIP pointer position for a UI webview node, returning `None`
/// when the position is unknown or lands on a transparent pixel (pass-through).
fn ui_pointer_pos(node: UiNode, images: &Assets<Image>) -> Option<Vec2> {
    let (rel, computed, surface, size) = node;
    let normalized = rel.normalized?;
    let pos = ui_pos_to_dip(normalized, computed.size(), computed.inverse_scale_factor());
    if let Some(surface) = surface
        && let Some(image) = images.get(surface.0.id())
        && is_pixel_transparent(image, size.0, pos)
    {
        return None;
    }
    Some(pos)
}

#[cfg(not(target_os = "windows"))]
fn on_ui_pointer_move(
    trigger: On<Pointer<Move>>,
    input: Res<ButtonInput<MouseButton>>,
    browsers: NonSend<Browsers>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    browsers.send_mouse_move(&trigger.entity, input.get_pressed(), pos, false);
}

#[cfg(not(target_os = "windows"))]
fn on_ui_pointer_pressed(
    trigger: On<Pointer<Press>>,
    browsers: NonSend<Browsers>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    browsers.send_mouse_click(&trigger.entity, pos, trigger.button, false);
}

#[cfg(not(target_os = "windows"))]
fn on_ui_pointer_released(
    trigger: On<Pointer<Release>>,
    browsers: NonSend<Browsers>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    browsers.send_mouse_click(&trigger.entity, pos, trigger.button, true);
}

#[cfg(not(target_os = "windows"))]
fn on_ui_pointer_scroll(
    trigger: On<Pointer<Scroll>>,
    browsers: NonSend<Browsers>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    let delta = scroll_delta(trigger.unit, trigger.x, trigger.y);
    browsers.send_mouse_wheel(&trigger.entity, pos, delta);
}

#[cfg(target_os = "windows")]
fn on_ui_pointer_move_win(
    trigger: On<Pointer<Move>>,
    input: Res<ButtonInput<MouseButton>>,
    proxy: Res<BrowsersProxy>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    let buttons: Vec<MouseButton> = input.get_pressed().copied().collect();
    proxy.send_mouse_move(&trigger.entity, &buttons, pos, false);
}

#[cfg(target_os = "windows")]
fn on_ui_pointer_pressed_win(
    trigger: On<Pointer<Press>>,
    proxy: Res<BrowsersProxy>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    proxy.send_mouse_click(&trigger.entity, pos, trigger.button, false);
}

#[cfg(target_os = "windows")]
fn on_ui_pointer_released_win(
    trigger: On<Pointer<Release>>,
    proxy: Res<BrowsersProxy>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    proxy.send_mouse_click(&trigger.entity, pos, trigger.button, true);
}

#[cfg(target_os = "windows")]
fn on_ui_pointer_scroll_win(
    trigger: On<Pointer<Scroll>>,
    proxy: Res<BrowsersProxy>,
    nodes: Query<UiNode, With<MaterialNode<WebviewUiMaterial>>>,
    images: Res<Assets<Image>>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() || resize_state.is_resizing() {
        return;
    }
    let Ok(node) = nodes.get(trigger.entity) else {
        return;
    };
    let Some(pos) = ui_pointer_pos(node, &images) else {
        return;
    };
    let delta = scroll_delta(trigger.unit, trigger.x, trigger.y);
    proxy.send_mouse_wheel(&trigger.entity, pos, delta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_maps_to_logical_center() {
        let out = ui_pos_to_dip(Vec2::new(0.0, 0.0), Vec2::new(800.0, 600.0), 1.0);
        assert_eq!(out, Vec2::new(400.0, 300.0));
    }

    #[test]
    fn top_left_maps_to_origin() {
        let out = ui_pos_to_dip(Vec2::new(-0.5, -0.5), Vec2::new(800.0, 600.0), 1.0);
        assert_eq!(out, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn bottom_right_maps_to_full_size() {
        let out = ui_pos_to_dip(Vec2::new(0.5, 0.5), Vec2::new(800.0, 600.0), 1.0);
        assert_eq!(out, Vec2::new(800.0, 600.0));
    }

    #[test]
    fn output_is_logical_dip_at_dpr_2() {
        let out = ui_pos_to_dip(Vec2::new(0.5, 0.5), Vec2::new(1600.0, 1200.0), 0.5);
        assert_eq!(out, Vec2::new(800.0, 600.0));
    }
}
