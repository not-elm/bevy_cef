use bevy::platform::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Custom JavaScript that bevy_cef evaluates in every webview.
///
/// Each extension's code runs in every frame's V8 context, in name order, after
/// `window.cef` is defined and before `PreloadScripts` and the page's own scripts.
/// Extensions apply to all webviews. Extension code can use `window.cef.emit()`,
/// `window.cef.listen()`, and `window.cef.brp()` to communicate with Bevy.
///
/// An exception or syntax error in one extension is reported to the page console, and the
/// remaining extensions and the page keep running.
///
/// # Example
///
/// ```no_run
/// use bevy_cef_core::prelude::*;
///
/// let extensions = CefExtensions::new()
///     .add("myGame", r#"
///         var myGame = {
///             sendScore: function(score) {
///                 window.cef.emit({ score: score });
///             }
///         };
///     "#);
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CefExtensions(pub HashMap<String, String>);

impl CefExtensions {
    /// Creates a new empty extensions collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a JavaScript extension, replacing any extension with the same name.
    ///
    /// # Arguments
    /// * `name` - Extension name. It sets the evaluation order and the script URL
    ///   `cef://localhost/__extensions__/{name}.js` that console errors and DevTools show.
    ///   Names are ordered by Unicode code point (byte-wise), so uppercase names run before
    ///   lowercase ones.
    /// * `code` - JavaScript code defining the extension's API. Empty code is ignored.
    pub fn add(mut self, name: impl Into<String>, code: impl Into<String>) -> Self {
        self.0.insert(name.into(), code.into());
        self
    }

    /// Returns true if no extensions are registered.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
