//! Process-wide webview focus.
//!
//! Tracks the active webview in [`FocusedWebview`] (set on pointer press by any
//! display path) and pushes it into CEF as single-browser focus, so keyboard and
//! IME reach only the focused webview.

use crate::common::WebviewSource;
use crate::system_param::pointer::find_webview_entity;
use bevy::prelude::*;
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;

/// The webview that currently holds input focus, if any.
///
/// Set on pointer press by `set_focus_on_press` for every display path, and
/// pushed into CEF as single-browser focus by `apply_webview_focus`.
#[derive(Resource, Default, Debug)]
pub struct FocusedWebview(pub Option<Entity>);

/// Wires the process-wide focus model: the [`FocusedWebview`] resource, a press
/// observer on every `WebviewSource`, and the system that drives CEF focus.
pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusedWebview>()
            .add_systems(Update, setup_focus_observers);

        #[cfg(not(target_os = "windows"))]
        app.add_systems(Update, apply_webview_focus);

        #[cfg(target_os = "windows")]
        app.add_systems(Update, apply_webview_focus_win);
    }
}

fn setup_focus_observers(mut commands: Commands, webviews: Query<Entity, Added<WebviewSource>>) {
    for entity in webviews.iter() {
        commands.entity(entity).observe(set_focus_on_press);
    }
}

fn set_focus_on_press(
    trigger: On<Pointer<Press>>,
    parents: Query<(Option<&ChildOf>, Has<WebviewSource>)>,
    mut focused: ResMut<FocusedWebview>,
) {
    if let Some(webview) = find_webview_entity(trigger.entity, &parents) {
        focused.0 = Some(webview);
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_webview_focus(
    focused: Res<FocusedWebview>,
    browsers: NonSend<Browsers>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    if !focused.is_changed() {
        return;
    }
    for webview in webviews.iter() {
        browsers.set_focus(&webview, focused.0 == Some(webview));
    }
}

#[cfg(target_os = "windows")]
fn apply_webview_focus_win(
    focused: Res<FocusedWebview>,
    proxy: Res<BrowsersProxy>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    if !focused.is_changed() {
        return;
    }
    for webview in webviews.iter() {
        proxy.set_focus(&webview, focused.0 == Some(webview));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn resolves_webview_ancestor_from_child() {
        let mut world = World::new();
        let parent = world.spawn(WebviewSource::inline("x")).id();
        let child = world.spawn(ChildOf(parent)).id();

        let mut state: SystemState<Query<(Option<&ChildOf>, Has<WebviewSource>)>> =
            SystemState::new(&mut world);
        let parents = state.get(&world);

        assert_eq!(find_webview_entity(parent, &parents), Some(parent));
        assert_eq!(find_webview_entity(child, &parents), Some(parent));
    }

    #[test]
    fn non_webview_entity_resolves_to_none() {
        let mut world = World::new();
        let orphan = world.spawn_empty().id();

        let mut state: SystemState<Query<(Option<&ChildOf>, Has<WebviewSource>)>> =
            SystemState::new(&mut world);
        let parents = state.get(&world);

        assert_eq!(find_webview_entity(orphan, &parents), None);
    }
}
