use bevy::prelude::*;
use bevy_cef_core::prelude::Browsers;
use serde::{Deserialize, Serialize};

pub(super) struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RequestGoBack>()
            .register_type::<RequestGoForward>()
            .add_observer(apply_request_go_back)
            .add_observer(apply_request_go_forward);
    }
}

/// A trigger event to navigate backwards.
#[derive(Default, Debug, Event, Copy, Clone, Reflect, Serialize, Deserialize)]
pub struct RequestGoBack;

/// A trigger event to navigate forwards.
#[derive(Default, Debug, Event, Copy, Clone, Reflect, Serialize, Deserialize)]
pub struct RequestGoForward;

fn apply_request_go_back(trigger: Trigger<RequestGoBack>, browsers: NonSend<Browsers>) {
    browsers.go_back(&trigger.target());
}

fn apply_request_go_forward(trigger: Trigger<RequestGoForward>, browsers: NonSend<Browsers>) {
    browsers.go_forward(&trigger.target());
}
