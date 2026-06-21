#![allow(clippy::type_complexity)]

mod common;
mod cursor_icon;
mod drag;
mod focus;
mod keyboard;
mod mute;
mod navigation;
mod resize;
mod system_param;
mod title;
mod webview;
mod zoom;

use crate::common::{
    LocalHostPlugin, MessageLoopPlugin, SandboxMode, WebviewCoreComponentsPlugin, WebviewDpiPlugin,
    resolve_no_sandbox,
};
use crate::cursor_icon::SystemCursorIconPlugin;
use crate::drag::DragPlugin;
use crate::focus::FocusPlugin;
use crate::keyboard::KeyboardPlugin;
use crate::mute::AudioMutePlugin;
use crate::prelude::{IpcPlugin, NavigationPlugin, WebviewPlugin};
use crate::resize::plugin::ResizePlugin;
use crate::title::TitlePlugin;
use crate::zoom::ZoomPlugin;
use bevy::prelude::*;
use bevy_cef_core::prelude::{
    CefCustomScheme, CefExtensions, CommandLineConfig, effective_command_line_config, switches,
};
use bevy_remote::RemotePlugin;

pub mod prelude {
    pub use crate::focus::FocusedWebview;
    pub use crate::keyboard::{CefKeyboardFilter, KeyboardDeliverSet, ModifiersState};
    pub use crate::resize::components::{AspectLockMode, WebviewResizable};
    pub use crate::{
        CefPlugin, RunOnMainThread, common::*, navigation::*, title::*, webview::prelude::*,
    };
    pub use bevy_cef_core::prelude::{
        CefCustomScheme, CefExtensions, CefSchemeBody, CefSchemeHandler, CefSchemeOptions,
        CefSchemeRequest, CefSchemeResponse, CommandLineConfig, switches,
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
    /// Controls Chromium's OS-level sandbox. Defaults to the current per-platform
    /// behavior; see [`SandboxMode`].
    pub sandbox: SandboxMode,
}

impl Plugin for CefPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: Must run before MessageLoopPlugin::build, which calls cef_initialize.
        // CEF's OnRegisterCustomSchemes fires during initialize; schemes registered
        // afterward are silently ignored.
        bevy_cef_core::prelude::init_registered_schemes(self.custom_schemes.clone());

        // Resolve the sandbox decision and compute the effective command line once.
        let no_sandbox = resolve_no_sandbox(self.sandbox);
        let strip_no_zygote = cfg!(target_os = "linux") && !no_sandbox;
        let effective_config =
            effective_command_line_config(&self.command_line_config, strip_no_zygote);

        // Warn when any security-relaxing switch is active (based on the EFFECTIVE set).
        let risky = switches::risky_present(&effective_config.switches);
        if !risky.is_empty() {
            warn!(
                "bevy_cef: web-security relaxations active: {risky:?}. \
                 Only enable these when loading fully trusted content."
            );
        }

        // Warn when the sandbox is requested but its prerequisites are absent on macOS
        // (the render process is not linked against cef_sandbox).
        #[cfg(target_os = "macos")]
        if self.sandbox == SandboxMode::Enabled {
            warn!(
                "bevy_cef: SandboxMode::Enabled requested on macOS, but the render \
                 process is not linked against cef_sandbox / does not call \
                 cef_sandbox_initialize(); the sandbox will not function and may abort. \
                 See the plugin-configuration docs."
            );
        }

        // Warn when the sandbox is requested on Linux, which requires a SUID-root
        // `chrome-sandbox` helper installed alongside the render process.
        #[cfg(target_os = "linux")]
        if self.sandbox == SandboxMode::Enabled {
            warn!(
                "bevy_cef: SandboxMode::Enabled requested on Linux; the sandbox requires a \
                 correctly-installed SUID-root chrome-sandbox helper alongside the render \
                 process. Without it, Chromium aborts the renderer with no bevy_cef-side \
                 diagnostic. See the plugin-configuration docs."
            );
        }

        app.add_plugins((
            LocalHostPlugin,
            MessageLoopPlugin {
                config: effective_config,
                extensions: self.extensions.clone(),
                root_cache_path: self.root_cache_path.clone(),
                no_sandbox,
            },
            WebviewCoreComponentsPlugin,
            WebviewDpiPlugin,
            WebviewPlugin,
            IpcPlugin,
            KeyboardPlugin,
            FocusPlugin,
            SystemCursorIconPlugin,
            DragPlugin,
            ResizePlugin,
            NavigationPlugin,
            TitlePlugin,
            ZoomPlugin,
            AudioMutePlugin,
        ));
        if !app.is_plugin_added::<RemotePlugin>() {
            app.add_plugins(RemotePlugin::default());
        }
    }
}
