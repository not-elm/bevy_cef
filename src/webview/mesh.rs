mod webview_extend_material;
mod webview_extend_standard_material;
mod webview_material;

pub use crate::common::*;
use crate::system_param::pointer::WebviewPointer;
use crate::webview::webview_sprite::WebviewSpritePlugin;
use bevy::input::mouse::MouseWheel;
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
        ))
        .add_systems(
            Update,
            (
                setup_observers,
                on_mouse_wheel.run_if(on_message::<MouseWheel>),
            ),
        );
    }
}

fn setup_observers(
    mut commands: Commands,
    webviews: Query<Entity, (Added<CefWebviewUri>, Or<(With<Mesh3d>, With<Mesh2d>)>)>,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .observe(on_pointer_move)
            .observe(on_pointer_pressed)
            .observe(on_pointer_released);
    }
}

fn on_pointer_move(
    trigger: On<Pointer<Move>>,
    input: Res<ButtonInput<MouseButton>>,
    pointer: WebviewPointer,
    browsers: NonSend<Browsers>,
) {
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };

    browsers.send_mouse_move(&webview, input.get_pressed(), pos, false);
}

fn on_pointer_pressed(
    trigger: On<Pointer<Press>>,
    browsers: NonSend<Browsers>,
    pointer: WebviewPointer,
) {
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };
    browsers.send_mouse_click(&webview, pos, trigger.button, false);
}

fn on_pointer_released(
    trigger: On<Pointer<Release>>,
    browsers: NonSend<Browsers>,
    pointer: WebviewPointer,
) {
    let Some((webview, pos)) = pointer.pos_from_trigger(&trigger) else {
        return;
    };
    browsers.send_mouse_click(&webview, pos, trigger.button, true);
}

fn on_mouse_wheel(
    mut er: MessageReader<MouseWheel>,
    browsers: NonSend<Browsers>,
    pointer: WebviewPointer,
    windows: Query<&Window>,
    webviews: Query<Entity, With<CefWebviewUri>>,
) {
    let Some(cursor_pos) = windows.iter().find_map(|window| window.cursor_position()) else {
        return;
    };
    for event in er.read() {
        for webview in webviews.iter() {
            let Some(pos) = pointer.pointer_pos(webview, cursor_pos) else {
                continue;
            };
            browsers.send_mouse_wheel(&webview, pos, Vec2::new(event.x, event.y));
        }
    }
}
