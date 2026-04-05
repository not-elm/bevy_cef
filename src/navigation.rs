use bevy::prelude::*;
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;
use serde::{Deserialize, Serialize};

pub(super) struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RequestGoBack>()
            .register_type::<RequestGoForward>()
            .register_type::<RequestNavigate>()
            .register_type::<RequestReload>();

        #[cfg(not(target_os = "windows"))]
        app.add_observer(apply_request_go_back)
            .add_observer(apply_request_go_forward)
            .add_observer(apply_request_navigate)
            .add_observer(apply_request_reload);

        #[cfg(target_os = "windows")]
        app.add_observer(apply_request_go_back_win)
            .add_observer(apply_request_go_forward_win)
            .add_observer(apply_request_navigate_win)
            .add_observer(apply_request_reload_win);
    }
}

/// A trigger event to navigate backwards.
#[derive(Debug, EntityEvent, Copy, Clone, Reflect, Serialize, Deserialize)]
pub struct RequestGoBack {
    #[event_target]
    pub webview: Entity,
}

/// A trigger event to navigate forwards.
#[derive(Debug, EntityEvent, Copy, Clone, Reflect, Serialize, Deserialize)]
pub struct RequestGoForward {
    #[event_target]
    pub webview: Entity,
}

/// A trigger event to navigate to a new URL.
#[derive(Debug, EntityEvent, Clone, Reflect, Serialize, Deserialize)]
pub struct RequestNavigate {
    #[event_target]
    pub webview: Entity,
    pub url: String,
}

/// A trigger event to reload the current page.
#[derive(Debug, EntityEvent, Copy, Clone, Reflect, Serialize, Deserialize)]
pub struct RequestReload {
    #[event_target]
    pub webview: Entity,
}

#[cfg(not(target_os = "windows"))]
fn apply_request_go_back(trigger: On<RequestGoBack>, browsers: NonSend<Browsers>) {
    browsers.go_back(&trigger.webview);
}

#[cfg(not(target_os = "windows"))]
fn apply_request_go_forward(trigger: On<RequestGoForward>, browsers: NonSend<Browsers>) {
    browsers.go_forward(&trigger.webview);
}

#[cfg(not(target_os = "windows"))]
fn apply_request_navigate(trigger: On<RequestNavigate>, browsers: NonSend<Browsers>) {
    browsers.navigate(&trigger.webview, &trigger.url);
}

#[cfg(not(target_os = "windows"))]
fn apply_request_reload(trigger: On<RequestReload>, browsers: NonSend<Browsers>) {
    browsers.reload_webview(&trigger.webview);
}

#[cfg(target_os = "windows")]
fn apply_request_go_back_win(trigger: On<RequestGoBack>, proxy: Res<BrowsersProxy>) {
    proxy.go_back(&trigger.webview);
}

#[cfg(target_os = "windows")]
fn apply_request_go_forward_win(trigger: On<RequestGoForward>, proxy: Res<BrowsersProxy>) {
    proxy.go_forward(&trigger.webview);
}

#[cfg(target_os = "windows")]
fn apply_request_navigate_win(trigger: On<RequestNavigate>, proxy: Res<BrowsersProxy>) {
    proxy.navigate(&trigger.webview, &trigger.url);
}

#[cfg(target_os = "windows")]
fn apply_request_reload_win(trigger: On<RequestReload>, proxy: Res<BrowsersProxy>) {
    proxy.reload_webview(&trigger.webview);
}
