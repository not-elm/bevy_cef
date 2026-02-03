use bevy::platform::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Custom JavaScript extensions to register via CEF's `register_extension`.
///
/// Extensions are global to all webviews and loaded before any page scripts run.
/// Use existing `window.cef.emit()`, `window.cef.listen()`, and `window.cef.brp()`
/// APIs within your extension code for Bevy communication.
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
///                 window.cef.emit('score_update', { score: score });
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

    /// Adds a JavaScript extension.
    ///
    /// # Arguments
    /// * `name` - Extension name (will be prefixed with `v8/` internally)
    /// * `code` - JavaScript code defining the extension's API
    pub fn add(mut self, name: impl Into<String>, code: impl Into<String>) -> Self {
        self.0.insert(name.into(), code.into());
        self
    }

    /// Returns true if no extensions are registered.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
