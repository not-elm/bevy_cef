use async_channel::Receiver;
use bevy::ecs::event::EntityTrigger;
use bevy::prelude::*;
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;
use bevy_cef_core::prelude::{LoadHandlerMessage, LoadHandlerSenderInner};
use serde::{Deserialize, Serialize};

pub(super) struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();
        app.insert_resource(LoadHandlerSender(tx))
            .insert_resource(LoadHandlerReceiver(rx))
            .register_type::<RequestGoBack>()
            .register_type::<RequestGoForward>()
            .register_type::<RequestNavigate>()
            .register_type::<RequestReload>()
            .register_type::<LoadingStateChanged>()
            .register_type::<LoadStarted>()
            .register_type::<LoadFinished>()
            .register_type::<LoadError>()
            .add_systems(PreUpdate, drain_load_events);

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

/// Fired when the browser's loading state changes (loading started/stopped,
/// back/forward availability changed).
#[derive(Debug, EntityEvent, Clone, Reflect, Serialize, Deserialize)]
pub struct LoadingStateChanged {
    #[event_target]
    pub webview: Entity,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

/// Fired when a new page load begins (the browser transitions from idle to loading).
#[derive(Debug, EntityEvent, Copy, Clone, Reflect, Serialize, Deserialize)]
pub struct LoadStarted {
    #[event_target]
    pub webview: Entity,
}

/// Fired when the main frame finishes loading.
#[derive(Debug, EntityEvent, Copy, Clone, Reflect, Serialize, Deserialize)]
pub struct LoadFinished {
    #[event_target]
    pub webview: Entity,
    pub http_status_code: i32,
}

/// Fired when the main frame fails to load.
#[derive(Debug, EntityEvent, Clone, Reflect, Serialize, Deserialize)]
pub struct LoadError {
    #[event_target]
    pub webview: Entity,
    pub error_code: i32,
    pub url: String,
}

#[derive(Resource, Debug, Deref)]
pub(crate) struct LoadHandlerSender(pub(crate) LoadHandlerSenderInner);

#[derive(Resource, Debug)]
struct LoadHandlerReceiver(Receiver<LoadHandlerMessage>);

fn drain_load_events(mut commands: Commands, receiver: Res<LoadHandlerReceiver>) {
    while let Ok(msg) = receiver.0.try_recv() {
        match msg {
            LoadHandlerMessage::LoadingStateChanged {
                webview,
                is_loading,
                can_go_back,
                can_go_forward,
            } => {
                commands.trigger_with(
                    LoadingStateChanged {
                        webview,
                        is_loading,
                        can_go_back,
                        can_go_forward,
                    },
                    EntityTrigger,
                );
                if is_loading {
                    commands.trigger_with(LoadStarted { webview }, EntityTrigger);
                }
            }
            LoadHandlerMessage::Finished {
                webview,
                http_status_code,
            } => {
                commands.trigger_with(
                    LoadFinished {
                        webview,
                        http_status_code,
                    },
                    EntityTrigger,
                );
            }
            LoadHandlerMessage::Error {
                webview,
                error_code,
                url,
            } => {
                commands.trigger_with(
                    LoadError {
                        webview,
                        error_code,
                        url,
                    },
                    EntityTrigger,
                );
            }
        }
    }
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
