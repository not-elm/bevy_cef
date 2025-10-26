use bevy::prelude::*;
use bevy_cef_core::prelude::*;
use serde::{Deserialize, Serialize};

/// A trigger event to emit an event from the host to the webview.
///
/// You need to subscribe to this event on the webview side by calling `window.cef.listen("event-id", (e) => {})` beforehand.
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, EntityEvent)]
#[reflect(Serialize, Deserialize)]
pub struct HostEmitEvent {
    #[event_target]
    pub webview: Entity,
    pub id: String,
    pub payload: String,
}

impl HostEmitEvent {
    /// Creates a new `HostEmitEvent` with the given id and payload.
    pub fn new(webview: Entity, id: impl Into<String>, payload: &impl Serialize) -> Self {
        Self {
            webview,
            id: id.into(),
            payload: serde_json::to_string(payload).unwrap_or_default(),
        }
    }
}

pub(super) struct HostEmitPlugin;

impl Plugin for HostEmitPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HostEmitEvent>().add_observer(host_emit);
    }
}

fn host_emit(trigger: On<HostEmitEvent>, browsers: NonSend<Browsers>) {
    if let Ok(v) = serde_json::to_value(&trigger.payload) {
        browsers.emit_event(&trigger.webview, trigger.id.clone(), &v);
    }
}
