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
use bevy_cef_core::prelude::CommandLineConfig;
use bevy_remote::RemotePlugin;

pub mod prelude {
    pub use crate::{CefPlugin, RunOnMainThread, common::*, navigation::*, webview::prelude::*};
}

pub struct RunOnMainThread;

pub struct CefPlugin {
    switches: Vec<String>,
    switch_values: Vec<(String, String)>,
}

impl Default for CefPlugin {
    fn default() -> Self {
        Self {
            switches: vec![
                #[cfg(all(target_os = "macos", debug_assertions))]
                "use-mock-keychain".to_string(),
            ],
            switch_values: Vec::new(),
        }
    }
}

impl CefPlugin {
    /// Add a command line switch (e.g., "disable-web-security", "disable-gpu").
    pub fn with_switch(mut self, name: impl Into<String>) -> Self {
        self.switches.push(name.into());
        self
    }

    /// Add a command line switch with a value (e.g., "remote-debugging-port", "9222").
    pub fn with_switch_value(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.switch_values.push((name.into(), value.into()));
        self
    }
}

impl Plugin for CefPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LocalHostPlugin,
            MessageLoopPlugin {
                config: CommandLineConfig {
                    switches: self.switches.clone(),
                    switch_values: self.switch_values.clone(),
                },
            },
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
