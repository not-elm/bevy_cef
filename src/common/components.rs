use bevy::prelude::*;
use bevy_cef_core::prelude::{HOST_CEF, SCHEME_CEF};
use serde::{Deserialize, Serialize};

pub(crate) struct WebviewCoreComponentsPlugin;

impl Plugin for WebviewCoreComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WebviewSize>()
            .register_type::<CefWebviewUri>()
            .register_type::<HostWindow>()
            .register_type::<ZoomLevel>()
            .register_type::<AudioMuted>()
            .register_type::<PreloadScripts>();
    }
}

/// A component that specifies the URI of the webview.
///
/// When opening a remote web page, specify the URI with the http(s) schema.
///
/// When opening a local file, use the custom schema `cef://localhost/` to specify the file path.
/// Alternatively, you can also use [`CefWebviewUri::local`].
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component, Debug)]
#[require(WebviewSize, ZoomLevel, AudioMuted, PreloadScripts)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub struct CefWebviewUri(pub String);

impl CefWebviewUri {
    /// Creates a new `CefWebviewUri` with the given URI.
    ///
    /// If you want to specify a local file path, use [`CefWebviewUri::local`] instead.
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    /// Creates a new `CefWebviewUri` with the given file path.
    ///
    /// It interprets the given path as a file path in the format `cef://localhost/<file_path>`.
    pub fn local(uri: impl Into<String>) -> Self {
        Self(format!("{SCHEME_CEF}://{HOST_CEF}/{}", uri.into()))
    }
}

/// Specifies the view size of the webview.
///
/// This does not affect the actual object size.
#[derive(Reflect, Component, Debug, Copy, Clone, PartialEq)]
#[reflect(Component, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub struct WebviewSize(pub Vec2);

impl Default for WebviewSize {
    fn default() -> Self {
        Self(Vec2::splat(800.0))
    }
}

/// An optional component to specify the parent window of the webview.
/// The window handle of [Window] specified by this component is passed to `parent_view` of [`WindowInfo`](cef::WindowInfo).
///
/// If this component is not inserted, the handle of [PrimaryWindow](bevy::window::PrimaryWindow) is passed instead.
#[derive(Reflect, Component, Debug, Copy, Clone, PartialEq)]
#[reflect(Component, Debug)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub struct HostWindow(pub Entity);

/// This component is used to specify the zoom level of the webview.
///
/// Specify 0.0 to reset the zoom level to the default.
#[derive(Reflect, Component, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Default)]
#[reflect(Component, Debug, Serialize, Deserialize, Default)]
pub struct ZoomLevel(pub f64);

/// This component is used to specify whether the audio of the webview is muted or not.
#[derive(Reflect, Component, Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
#[reflect(Component, Debug, Default, Serialize, Deserialize)]
pub struct AudioMuted(pub bool);

/// This component is used to preload scripts in the webview.
///
/// Scripts specified in this component are executed before the scripts in the HTML.
#[derive(Reflect, Component, Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[reflect(Component, Debug, Default, Serialize, Deserialize)]
pub struct PreloadScripts(pub Vec<String>);

impl<L, S> From<L> for PreloadScripts
where
    L: IntoIterator<Item = S>,
    S: Into<String>,
{
    fn from(scripts: L) -> Self {
        Self(scripts.into_iter().map(Into::into).collect())
    }
}
