//! Process-wide webview focus.
//!
//! Tracks the active webview in [`FocusedWebview`] (set on pointer press by any
//! display path) and pushes it into CEF as single-browser focus, so keyboard and
//! IME reach only the focused webview.

use crate::common::WebviewSource;
use crate::system_param::pointer::find_webview_entity;
use bevy::prelude::*;
use bevy_cef_core::prelude::Browsers;

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
            .add_systems(Update, (setup_focus_observers, apply_webview_focus));
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

fn apply_webview_focus(
    focused: Res<FocusedWebview>,
    browsers: NonSend<Browsers>,
    webviews: Query<Entity, With<WebviewSource>>,
    mut prev: Local<Option<Entity>>,
) {
    if !focused.is_changed() {
        return;
    }
    match focused.0 {
        Some(target) => {
            for webview in webviews.iter() {
                browsers.set_focus(&webview, webview == target);
            }
            *prev = Some(target);
        }
        // NOTE: blur ONLY the previously-focused webview, never blur-all.
        // `init_resource` marks this changed with `None` on the first frame, but
        // `prev` is still `None` then, so a webview CEF auto-focused at startup
        // is left alone (blurring it would leave the app keyboard-dead until the
        // first click). A genuine Some→None transition — focus moved to a
        // non-webview (e.g. a terminal pane in an embedder) — releases CEF focus
        // on the webview that held it so its DOM element/caret is dropped.
        None => {
            if let Some(p) = prev.take() {
                browsers.set_focus(&p, false);
            }
        }
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
        let parents = state.get(&world).unwrap();

        assert_eq!(find_webview_entity(parent, &parents), Some(parent));
        assert_eq!(find_webview_entity(child, &parents), Some(parent));
    }

    #[test]
    fn non_webview_entity_resolves_to_none() {
        let mut world = World::new();
        let orphan = world.spawn_empty().id();

        let mut state: SystemState<Query<(Option<&ChildOf>, Has<WebviewSource>)>> =
            SystemState::new(&mut world);
        let parents = state.get(&world).unwrap();

        assert_eq!(find_webview_entity(orphan, &parents), None);
    }
}
