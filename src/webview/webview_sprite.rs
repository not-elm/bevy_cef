use crate::common::{CefWebviewUri, WebviewSize};
use crate::prelude::update_webview_image;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_cef_core::prelude::{Browsers, RenderTextureMessage};
use std::fmt::Debug;

pub(in crate::webview) struct WebviewSpritePlugin;

impl Plugin for WebviewSpritePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SpritePickingPlugin>() {
            app.add_plugins(SpritePickingPlugin);
        }

        app.add_systems(
            Update,
            (
                setup_observers,
                on_mouse_wheel.run_if(on_message::<MouseWheel>),
            ),
        )
        .add_systems(PostUpdate, render.run_if(on_message::<RenderTextureMessage>));
    }
}

fn render(
    mut er: MessageReader<RenderTextureMessage>,
    mut images: ResMut<Assets<bevy::prelude::Image>>,
    webviews: Query<&Sprite, With<CefWebviewUri>>,
) {
    for texture in er.read() {
        if let Ok(sprite) = webviews.get(texture.webview)
            && let Some(image) = images.get_mut(sprite.image.id())
        {
            update_webview_image(texture.clone(), image);
        }
    }
}

fn setup_observers(
    mut commands: Commands,
    webviews: Query<Entity, (Added<CefWebviewUri>, With<Sprite>)>,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .observe(apply_on_pointer_move)
            .observe(apply_on_pointer_pressed)
            .observe(apply_on_pointer_released);
    }
}

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

            browsers.send_mouse_wheel(&webview, pos, Vec2::new(event.x, event.y));
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
