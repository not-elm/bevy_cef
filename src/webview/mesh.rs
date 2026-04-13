mod webview_extend_material;
mod webview_extend_standard_material;
mod webview_material;

pub use crate::common::*;
use crate::system_param::pointer::WebviewPointer;
use crate::webview::webview_sprite::WebviewSpritePlugin;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_cef_core::prelude::*;
pub use webview_extend_material::*;
pub use webview_extend_standard_material::*;
pub use webview_material::*;

pub struct MeshWebviewPlugin;

impl Plugin for MeshWebviewPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MeshPickingPlugin>() {
            app.add_plugins(MeshPickingPlugin);
        }

        app.add_plugins((
            WebviewMaterialPlugin,
            WebviewExtendStandardMaterialPlugin,
            WebviewSpritePlugin,
        ));

        #[cfg(not(target_os = "windows"))]
        app.add_systems(
            Update,
            (
                setup_observers,
                on_mouse_wheel.run_if(on_message::<MouseWheel>),
            ),
        );

        #[cfg(target_os = "windows")]
        app.add_systems(
            Update,
            (
                setup_observers_win,
                on_mouse_wheel_win.run_if(on_message::<MouseWheel>),
            ),
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn setup_observers(
    mut commands: Commands,
    webviews: Query<Entity, (Added<WebviewSource>, Or<(With<Mesh3d>, With<Mesh2d>)>)>,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .observe(on_pointer_move)
            .observe(on_pointer_pressed)
            .observe(on_pointer_released);
    }
}

#[cfg(not(target_os = "windows"))]
fn on_pointer_move(
    trigger: On<Pointer<Move>>,
    input: Res<ButtonInput<MouseButton>>,
    pointer: WebviewPointer,
    browsers: NonSend<Browsers>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };

    browsers.send_mouse_move(&webview, input.get_pressed(), pos, false);
}

#[cfg(not(target_os = "windows"))]
fn on_pointer_pressed(
    trigger: On<Pointer<Press>>,
    browsers: NonSend<Browsers>,
    pointer: WebviewPointer,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };
    browsers.send_mouse_click(&webview, pos, trigger.button, false);
}

#[cfg(not(target_os = "windows"))]
fn on_pointer_released(
    trigger: On<Pointer<Release>>,
    browsers: NonSend<Browsers>,
    pointer: WebviewPointer,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };
    browsers.send_mouse_click(&webview, pos, trigger.button, true);
}

#[cfg(not(target_os = "windows"))]
fn on_mouse_wheel(
    mut er: MessageReader<MouseWheel>,
    browsers: NonSend<Browsers>,
    pointer: WebviewPointer,
    windows: Query<&Window>,
    webviews: Query<Entity, (With<WebviewSource>, Or<(With<Mesh3d>, With<Mesh2d>)>)>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some(cursor_pos) = windows.iter().find_map(|window| window.cursor_position()) else {
        return;
    };
    for event in er.read() {
        for webview in webviews.iter() {
            let Some(pos) = pointer.pointer_pos(webview, cursor_pos) else {
                continue;
            };
            let delta = match event.unit {
                MouseScrollUnit::Line => {
                    // CEF expects pixel deltas; Chromium default: 3 lines × 40px = 120px per notch
                    Vec2::new(event.x * 120.0, event.y * 120.0)
                }
                MouseScrollUnit::Pixel => Vec2::new(event.x, event.y),
            };
            browsers.send_mouse_wheel(&webview, pos, delta);
        }
    }
}

#[cfg(target_os = "windows")]
fn setup_observers_win(
    mut commands: Commands,
    webviews: Query<Entity, (Added<WebviewSource>, Or<(With<Mesh3d>, With<Mesh2d>)>)>,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .observe(on_pointer_move_win)
            .observe(on_pointer_pressed_win)
            .observe(on_pointer_released_win);
    }
}

#[cfg(target_os = "windows")]
fn on_pointer_move_win(
    trigger: On<Pointer<Move>>,
    input: Res<ButtonInput<MouseButton>>,
    pointer: WebviewPointer,
    proxy: Res<BrowsersProxy>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };

    let buttons: Vec<MouseButton> = input.get_pressed().copied().collect();
    proxy.send_mouse_move(&webview, &buttons, pos, false);
}

#[cfg(target_os = "windows")]
fn on_pointer_pressed_win(
    trigger: On<Pointer<Press>>,
    proxy: Res<BrowsersProxy>,
    pointer: WebviewPointer,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };
    proxy.send_mouse_click(&webview, pos, trigger.button, false);
}

#[cfg(target_os = "windows")]
fn on_pointer_released_win(
    trigger: On<Pointer<Release>>,
    proxy: Res<BrowsersProxy>,
    pointer: WebviewPointer,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };
    proxy.send_mouse_click(&webview, pos, trigger.button, true);
}

#[cfg(target_os = "windows")]
fn on_mouse_wheel_win(
    mut er: MessageReader<MouseWheel>,
    proxy: Res<BrowsersProxy>,
    pointer: WebviewPointer,
    windows: Query<&Window>,
    webviews: Query<Entity, (With<WebviewSource>, Or<(With<Mesh3d>, With<Mesh2d>)>)>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some(cursor_pos) = windows.iter().find_map(|window| window.cursor_position()) else {
        return;
    };
    for event in er.read() {
        for webview in webviews.iter() {
            let Some(pos) = pointer.pointer_pos(webview, cursor_pos) else {
                continue;
            };
            let delta = match event.unit {
                MouseScrollUnit::Line => {
                    // CEF expects pixel deltas; Chromium default: 3 lines × 40px = 120px per notch
                    Vec2::new(event.x * 120.0, event.y * 120.0)
                }
                MouseScrollUnit::Pixel => Vec2::new(event.x, event.y),
            };
            proxy.send_mouse_wheel(&webview, pos, delta);
        }
    }
}
