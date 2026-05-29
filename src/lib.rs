#![allow(clippy::type_complexity)]

mod common;
mod cursor_icon;
mod drag;
mod keyboard;
mod mute;
mod navigation;
mod resize;
mod system_param;
mod webview;
mod zoom;

use crate::common::{
    LocalHostPlugin, MessageLoopPlugin, WebviewCoreComponentsPlugin, WebviewDpiPlugin,
};
use crate::cursor_icon::SystemCursorIconPlugin;
use crate::drag::DragPlugin;
use crate::keyboard::KeyboardPlugin;
use crate::mute::AudioMutePlugin;
use crate::prelude::{IpcPlugin, NavigationPlugin, WebviewPlugin};
use crate::resize::plugin::ResizePlugin;
use crate::zoom::ZoomPlugin;
use bevy::prelude::*;
use bevy_cef_core::prelude::{CefCustomScheme, CefExtensions, CommandLineConfig};
use bevy_remote::RemotePlugin;

pub mod prelude {
    pub use crate::resize::components::{AspectLockMode, WebviewResizable};
    pub use crate::{CefPlugin, RunOnMainThread, common::*, navigation::*, webview::prelude::*};
    pub use bevy_cef_core::prelude::{
        CefCustomScheme, CefExtensions, CefSchemeBody, CefSchemeHandler, CefSchemeOptions,
        CefSchemeRequest, CefSchemeResponse, CommandLineConfig,
    };
}

pub struct RunOnMainThread;

#[derive(Default)]
pub struct CefPlugin {
    pub command_line_config: CommandLineConfig,
    pub extensions: CefExtensions,
    /// Root directory for CEF runtime data (cache, profiles, etc.).
    /// If empty, defaults to the executable's directory.
    /// Should be set to a user-writable path (e.g. `~/.myapp/cef_data`).
    pub root_cache_path: Option<String>,
    /// Custom URL schemes to register in addition to the built-in
    /// `cef://localhost/`. Each carries a handler that services requests.
    pub custom_schemes: Vec<CefCustomScheme>,
}

impl std::fmt::Debug for CefPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CefPlugin")
            .field("command_line_config", &self.command_line_config)
            .field("extensions", &self.extensions)
            .field("root_cache_path", &self.root_cache_path)
            .field(
                "custom_schemes",
                &self.custom_schemes.iter().map(|s| &s.name).collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl Plugin for CefPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: Must run before MessageLoopPlugin::build, which calls cef_initialize.
        // CEF's OnRegisterCustomSchemes fires during initialize; schemes registered
        // afterward are silently ignored.
        bevy_cef_core::prelude::init_registered_schemes(self.custom_schemes.clone());
        app.add_plugins((
            LocalHostPlugin,
            MessageLoopPlugin {
                config: self.command_line_config.clone(),
                extensions: self.extensions.clone(),
                root_cache_path: self.root_cache_path.clone(),
            },
            WebviewCoreComponentsPlugin,
            WebviewDpiPlugin,
            WebviewPlugin,
            IpcPlugin,
            KeyboardPlugin,
            SystemCursorIconPlugin,
            DragPlugin,
            ResizePlugin,
            NavigationPlugin,
            ZoomPlugin,
            AudioMutePlugin,
        ));
        if !app.is_plugin_added::<RemotePlugin>() {
            app.add_plugins(RemotePlugin::default());
        }
    }
}
