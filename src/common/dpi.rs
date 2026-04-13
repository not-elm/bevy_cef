//! DPI / device-scale-factor tracking for webviews.
//!
//! `WebviewDpiPlugin` maintains each webview's `WebviewDpr` component,
//! seeding it from the host window at spawn and refreshing it when the
//! host window's `scale_factor` changes (monitor move, OS DPI setting).
//! The change is then committed to CEF via `notify_screen_info_changed`.

use crate::common::{HostWindow, WebviewDpr, WebviewSource};
use crate::webview::WebviewSet;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowScaleFactorChanged};

#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;

pub struct WebviewDpiPlugin;

impl Plugin for WebviewDpiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                seed_webview_dpr_system,
                refresh_on_scale_factor_changed_system,
            )
                .in_set(WebviewSet::DpiSeed),
        );

        #[cfg(not(target_os = "windows"))]
        app.add_systems(
            Update,
            commit_webview_dpr_system.in_set(WebviewSet::CommitResize),
        );

        #[cfg(target_os = "windows")]
        app.add_systems(
            Update,
            commit_webview_dpr_system_win.in_set(WebviewSet::CommitResize),
        );
    }
}

fn seed_webview_dpr_system(
    mut webviews: Query<(&mut WebviewDpr, Option<&HostWindow>), Added<WebviewSource>>,
    windows: Query<&Window>,
    primary: Query<&Window, With<PrimaryWindow>>,
) {
    for (mut dpr, host) in webviews.iter_mut() {
        let resolved = host
            .and_then(|hw| windows.get(hw.0).ok())
            .or_else(|| primary.single().ok())
            .map(|w| w.scale_factor())
            .unwrap_or_else(|| {
                warn!("No window found when seeding WebviewDpr; falling back to 1.0");
                1.0
            });
        dpr.0 = resolved;
    }
}

// Stubs — implemented in Task 10 and Task 11.
fn refresh_on_scale_factor_changed_system() {}

#[cfg(not(target_os = "windows"))]
fn commit_webview_dpr_system() {}

#[cfg(target_os = "windows")]
fn commit_webview_dpr_system_win() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{WebviewDpr, WebviewSource};

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::window::WindowPlugin {
                primary_window: None,
                ..default()
            })
            .add_systems(Update, seed_webview_dpr_system);
        app
    }

    #[test]
    fn seed_falls_back_to_1_0_when_no_windows_exist() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((WebviewSource::new("https://example.com"), WebviewDpr(1.0)))
            .id();
        app.update();
        let dpr = app.world().get::<WebviewDpr>(entity).unwrap();
        assert_eq!(dpr.0, 1.0);
    }

    #[test]
    fn seed_uses_primary_window_scale_factor_when_no_host_window() {
        let mut app = make_app();
        // Spawn a fake PrimaryWindow with scale_factor_override = 2.0
        let window = Window {
            resolution: bevy::window::WindowResolution::new(800, 600)
                .with_scale_factor_override(2.0),
            ..default()
        };
        app.world_mut().spawn((window, PrimaryWindow));

        let entity = app
            .world_mut()
            .spawn((WebviewSource::new("https://example.com"), WebviewDpr(1.0)))
            .id();
        app.update();
        let dpr = app.world().get::<WebviewDpr>(entity).unwrap();
        assert_eq!(dpr.0, 2.0);
    }
}
