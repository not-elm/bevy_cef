//! You can create a webview as a sprite in your scene.

use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, (spawn_camera_2d, spawn_sprite_webview))
        .run();
}

fn spawn_camera_2d(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_sprite_webview(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        CefWebviewUri::new("https://github.com/not-elm/bevy_cef"),
        Pickable::default(),
        Sprite {
            image: images.add(Image::default()),
            custom_size: Some(Vec2::splat(500.0)),
            ..default()
        },
    ));
}
