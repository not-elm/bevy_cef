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

pub struct CefPlugin {
    switches: Vec<String>,
    switch_values: Vec<(String, String)>,
    include_default_switches: bool,
}

impl Default for CefPlugin {
    fn default() -> Self {
        Self {
            switches: Vec::new(),
            switch_values: Vec::new(),
            include_default_switches: true,
        }
    }
}

impl CefPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a command line switch (e.g., "disable-web-security", "disable-gpu").
    pub fn with_switch(mut self, name: impl Into<String>) -> Self {
        self.switches.push(name.into());
        self
    }

    /// Add a command line switch with a value (e.g., "remote-debugging-port", "9222").
    pub fn with_switch_value(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.switch_values.push((name.into(), value.into()));
        self
    }

    /// Disable default switches. By default, `use-mock-keychain` is included.
    pub fn without_default_switches(mut self) -> Self {
        self.include_default_switches = false;
        self
    }

    fn build_command_line_config(&self) -> bevy_cef_core::prelude::CommandLineConfig {
        let mut switches = self.switches.clone();
        if self.include_default_switches {
            switches.insert(0, "use-mock-keychain".to_string());
        }
        bevy_cef_core::prelude::CommandLineConfig {
            switches,
            switch_values: self.switch_values.clone(),
        }
    }
}

impl Plugin for CefPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LocalHostPlugin,
            MessageLoopPlugin {
                config: self.build_command_line_config(),
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
