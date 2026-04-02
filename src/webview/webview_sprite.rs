use crate::common::{WebviewSize, WebviewSource};
use crate::prelude::update_webview_image;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;
use bevy_cef_core::prelude::RenderTextureMessage;
use std::fmt::Debug;

pub(in crate::webview) struct WebviewSpritePlugin;

impl Plugin for WebviewSpritePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SpritePickingPlugin>() {
            app.add_plugins(SpritePickingPlugin);
        }

        app.add_systems(
            PostUpdate,
            render.run_if(on_message::<RenderTextureMessage>),
        );

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

fn render(
    mut er: MessageReader<RenderTextureMessage>,
    mut images: ResMut<Assets<bevy::prelude::Image>>,
    webviews: Query<&Sprite, With<WebviewSource>>,
) {
    for texture in er.read() {
        if let Ok(sprite) = webviews.get(texture.webview)
            && let Some(image) = images.get_mut(sprite.image.id())
        {
            update_webview_image(texture, image);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn setup_observers(
    mut commands: Commands,
    webviews: Query<Entity, (Added<WebviewSource>, With<Sprite>)>,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .observe(apply_on_pointer_move)
            .observe(apply_on_pointer_pressed)
            .observe(apply_on_pointer_released);
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_on_pointer_move(
    trigger: On<Pointer<Move>>,
    input: Res<ButtonInput<MouseButton>>,
    browsers: NonSend<Browsers>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
) {
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    browsers.send_mouse_move(&trigger.entity, input.get_pressed(), pos, false);
}

#[cfg(not(target_os = "windows"))]
fn apply_on_pointer_pressed(
    trigger: On<Pointer<Press>>,
    browsers: NonSend<Browsers>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
) {
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    browsers.send_mouse_click(&trigger.entity, pos, trigger.button, false);
}

#[cfg(not(target_os = "windows"))]
fn apply_on_pointer_released(
    trigger: On<Pointer<Release>>,
    browsers: NonSend<Browsers>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
) {
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    browsers.send_mouse_click(&trigger.entity, pos, trigger.button, true);
}

#[cfg(not(target_os = "windows"))]
fn on_mouse_wheel(
    mut er: MessageReader<MouseWheel>,
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &Sprite, &WebviewSize, &GlobalTransform)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    let Some(cursor_pos) = windows.iter().find_map(|window| window.cursor_position()) else {
        return;
    };
    for event in er.read() {
        for (webview, sprite, webview_size, gtf) in webviews.iter() {
            let Some(pos) = obtain_relative_pos(sprite, webview_size, gtf, &cameras, cursor_pos)
            else {
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

// ---------------------------------------------------------------------------
// Windows: BrowsersProxy variants
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn setup_observers_win(
    mut commands: Commands,
    webviews: Query<Entity, (Added<WebviewSource>, With<Sprite>)>,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .observe(apply_on_pointer_move_win)
            .observe(apply_on_pointer_pressed_win)
            .observe(apply_on_pointer_released_win);
    }
}

#[cfg(target_os = "windows")]
fn apply_on_pointer_move_win(
    trigger: On<Pointer<Move>>,
    input: Res<ButtonInput<MouseButton>>,
    proxy: Res<BrowsersProxy>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
) {
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    let buttons: Vec<MouseButton> = input.get_pressed().copied().collect();
    proxy.send_mouse_move(&trigger.entity, &buttons, pos, false);
}

#[cfg(target_os = "windows")]
fn apply_on_pointer_pressed_win(
    trigger: On<Pointer<Press>>,
    proxy: Res<BrowsersProxy>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
) {
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    proxy.send_mouse_click(&trigger.entity, pos, trigger.button, false);
}

#[cfg(target_os = "windows")]
fn apply_on_pointer_released_win(
    trigger: On<Pointer<Release>>,
    proxy: Res<BrowsersProxy>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
) {
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    proxy.send_mouse_click(&trigger.entity, pos, trigger.button, true);
}

#[cfg(target_os = "windows")]
fn on_mouse_wheel_win(
    mut er: MessageReader<MouseWheel>,
    proxy: Res<BrowsersProxy>,
    webviews: Query<(Entity, &Sprite, &WebviewSize, &GlobalTransform)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    let Some(cursor_pos) = windows.iter().find_map(|window| window.cursor_position()) else {
        return;
    };
    for event in er.read() {
        for (webview, sprite, webview_size, gtf) in webviews.iter() {
            let Some(pos) = obtain_relative_pos(sprite, webview_size, gtf, &cameras, cursor_pos)
            else {
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

fn obtain_relative_pos_from_trigger<E: Debug + Clone + Reflect>(
    trigger: &On<Pointer<E>>,
    webviews: &Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (sprite, webview_size, gtf) = webviews.get(trigger.entity).ok()?;
    obtain_relative_pos(
        sprite,
        webview_size,
        gtf,
        cameras,
        trigger.pointer_location.position,
    )
}

fn obtain_relative_pos(
    sprite: &Sprite,
    webview_size: &WebviewSize,
    transform: &GlobalTransform,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    cursor_pos: Vec2,
) -> Option<Vec2> {
    let size = sprite.custom_size?;
    let viewport_pos = cameras.iter().find_map(|(camera, camera_gtf)| {
        camera
            .world_to_viewport(camera_gtf, transform.translation())
            .ok()
    })?;
    let relative_pos = (cursor_pos - viewport_pos + size / 2.0) / size;
    Some(relative_pos * webview_size.0)
}
