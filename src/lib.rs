#![allow(clippy::type_complexity)]

mod common;
mod cursor_icon;
mod keyboard;
mod mute;
mod navigation;
mod system_param;
mod webview;
mod zoom;

use crate::common::{LocalHostPlugin, MessageLoopPlugin, WebviewCoreComponentsPlugin};
use crate::cursor_icon::SystemCursorIconPlugin;
use crate::keyboard::KeyboardPlugin;
use crate::mute::AudioMutePlugin;
use crate::prelude::{IpcPlugin, NavigationPlugin, WebviewPlugin};
use crate::zoom::ZoomPlugin;
use bevy::prelude::*;
use bevy_remote::RemotePlugin;

pub mod prelude {
    pub use crate::{CefPlugin, RunOnMainThread, common::*, navigation::*, webview::prelude::*};
}

pub struct RunOnMainThread;

pub struct CefPlugin;

impl Plugin for CefPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LocalHostPlugin,
            MessageLoopPlugin,
            WebviewCoreComponentsPlugin,
            WebviewPlugin,
            IpcPlugin,
            KeyboardPlugin,
            SystemCursorIconPlugin,
            NavigationPlugin,
            ZoomPlugin,
            AudioMutePlugin,
        ));
        if !app.is_plugin_added::<RemotePlugin>() {
            app.add_plugins(RemotePlugin::default());
        }
    }
}
