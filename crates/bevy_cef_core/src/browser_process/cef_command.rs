//! CefCommand enum and BrowsersProxy resource for the Windows multi-threaded
//! message loop architecture.
//!
//! On Windows with `multi_threaded_message_loop`, CEF owns its own UI thread.
//! Bevy systems cannot call CEF APIs directly from arbitrary threads. Instead,
//! they enqueue [`CefCommand`] variants through [`BrowsersProxy`], and a
//! dedicated drain task on the CEF thread processes them.

use async_channel::Sender;
use bevy::prelude::*;
use bevy_remote::BrpMessage;
use raw_window_handle::RawWindowHandle;

use crate::browser_process::client_handler::IpcEventRaw;
use crate::browser_process::display_handler::SystemCursorIconSenderInner;
use crate::browser_process::localhost::Requester;

/// A `Send`-safe wrapper around [`RawWindowHandle`].
///
/// `RawWindowHandle` is not `Send` because some variants contain raw pointers.
/// On Windows the handle is an `HWND`, which is safe to send across threads.
#[allow(deprecated)]
pub struct SendRawWindowHandle(pub RawWindowHandle);

// SAFETY: On Windows, `RawWindowHandle` contains a Win32 `HWND` which is safe
// to send and share across threads.
unsafe impl Send for SendRawWindowHandle {}
unsafe impl Sync for SendRawWindowHandle {}

/// Every operation Bevy can request from the CEF UI thread.
///
/// Each variant corresponds to a public method on [`super::Browsers`]. Fields
/// use owned types so the command is `Send`.
#[allow(dead_code)]
pub enum CefCommand {
    /// Create a new browser instance for the given webview entity.
    CreateBrowser {
        webview: Entity,
        uri: String,
        webview_size: Vec2,
        requester: Requester,
        ipc_event_sender: Sender<IpcEventRaw>,
        brp_sender: Sender<BrpMessage>,
        system_cursor_icon_sender: SystemCursorIconSenderInner,
        initialize_scripts: Vec<String>,
        window_handle: Option<SendRawWindowHandle>,
    },

    /// Close and remove the browser for the given entity.
    Close { entity: Entity },

    /// Navigate a webview to a new URL.
    Navigate { entity: Entity, url: String },

    /// Navigate backwards.
    GoBack { entity: Entity },

    /// Navigate forwards.
    GoForward { entity: Entity },

    /// Reload the current page.
    ReloadWebview { entity: Entity },

    /// Resize the webview texture.
    Resize { entity: Entity, size: Vec2 },

    /// Forward a mouse-move event.
    SendMouseMove {
        webview: Entity,
        buttons: Vec<MouseButton>,
        position: Vec2,
        mouse_leave: bool,
    },

    /// Forward a mouse-click event.
    SendMouseClick {
        webview: Entity,
        position: Vec2,
        button: PointerButton,
        mouse_up: bool,
    },

    /// Forward a mouse-wheel event.
    SendMouseWheel {
        webview: Entity,
        position: Vec2,
        delta: Vec2,
    },

    /// Forward a keyboard event.
    SendKey {
        webview: Entity,
        event: cef::KeyEvent,
    },

    /// Emit a host event to the webview's JS context.
    EmitEvent {
        webview: Entity,
        id: String,
        event: serde_json::Value,
    },

    /// Show DevTools for the given webview.
    ShowDevTool { webview: Entity },

    /// Close DevTools for the given webview.
    CloseDevTools { webview: Entity },

    /// Set the zoom level for a webview.
    SetZoomLevel { webview: Entity, zoom_level: f64 },

    /// Set audio muted state for a webview.
    SetAudioMuted { webview: Entity, muted: bool },

    /// Reload all browsers.
    Reload,

    /// Set IME composition text.
    SetImeComposition {
        text: String,
        cursor_utf16: Option<u32>,
    },

    /// Cancel IME composition.
    ImeCancelComposition,

    /// Finish IME composition.
    ImeFinishComposition { keep_selection: bool },

    /// Commit IME text.
    SetImeCommitText { text: String },

    /// Send external begin frame to all browsers.
    SendExternalBeginFrame,
}

/// A `Send + Sync` Bevy [`Resource`] that enqueues [`CefCommand`]s for the CEF
/// UI thread.
///
/// Every convenience method mirrors the corresponding [`super::Browsers`]
/// method so that call-site migration is minimal.
#[derive(Resource, Clone)]
pub struct BrowsersProxy {
    tx: Sender<CefCommand>,
}

impl BrowsersProxy {
    /// Create a new proxy wrapping the given sender.
    pub fn new(tx: Sender<CefCommand>) -> Self {
        Self { tx }
    }

    /// Returns `true` when no commands are pending.
    pub fn is_empty(&self) -> bool {
        self.tx.is_empty()
    }

    /// Direct access to the underlying sender.
    pub fn sender(&self) -> &Sender<CefCommand> {
        &self.tx
    }

    // -- Browser lifecycle ----------------------------------------------------

    #[allow(clippy::too_many_arguments, deprecated)]
    pub fn create_browser(
        &self,
        webview: Entity,
        uri: &str,
        webview_size: Vec2,
        requester: Requester,
        ipc_event_sender: Sender<IpcEventRaw>,
        brp_sender: Sender<BrpMessage>,
        system_cursor_icon_sender: SystemCursorIconSenderInner,
        initialize_scripts: &[String],
        window_handle: Option<RawWindowHandle>,
    ) {
        let _ = self.tx.send_blocking(CefCommand::CreateBrowser {
            webview,
            uri: uri.to_owned(),
            webview_size,
            requester,
            ipc_event_sender,
            brp_sender,
            system_cursor_icon_sender,
            initialize_scripts: initialize_scripts.to_vec(),
            window_handle: window_handle.map(SendRawWindowHandle),
        });
    }

    pub fn close(&self, entity: &Entity) {
        let _ = self.tx.send_blocking(CefCommand::Close { entity: *entity });
    }

    // -- Navigation -----------------------------------------------------------

    pub fn navigate(&self, entity: &Entity, url: &str) {
        let _ = self.tx.send_blocking(CefCommand::Navigate {
            entity: *entity,
            url: url.to_owned(),
        });
    }

    pub fn go_back(&self, entity: &Entity) {
        let _ = self
            .tx
            .send_blocking(CefCommand::GoBack { entity: *entity });
    }

    pub fn go_forward(&self, entity: &Entity) {
        let _ = self
            .tx
            .send_blocking(CefCommand::GoForward { entity: *entity });
    }

    pub fn reload_webview(&self, entity: &Entity) {
        let _ = self
            .tx
            .send_blocking(CefCommand::ReloadWebview { entity: *entity });
    }

    // -- Resize ---------------------------------------------------------------

    pub fn resize(&self, entity: &Entity, size: Vec2) {
        let _ = self.tx.send_blocking(CefCommand::Resize {
            entity: *entity,
            size,
        });
    }

    // -- Input forwarding -----------------------------------------------------

    pub fn send_mouse_move(
        &self,
        webview: &Entity,
        buttons: &[MouseButton],
        position: Vec2,
        mouse_leave: bool,
    ) {
        let _ = self.tx.send_blocking(CefCommand::SendMouseMove {
            webview: *webview,
            buttons: buttons.to_vec(),
            position,
            mouse_leave,
        });
    }

    pub fn send_mouse_click(
        &self,
        webview: &Entity,
        position: Vec2,
        button: PointerButton,
        mouse_up: bool,
    ) {
        let _ = self.tx.send_blocking(CefCommand::SendMouseClick {
            webview: *webview,
            position,
            button,
            mouse_up,
        });
    }

    pub fn send_mouse_wheel(&self, webview: &Entity, position: Vec2, delta: Vec2) {
        let _ = self.tx.send_blocking(CefCommand::SendMouseWheel {
            webview: *webview,
            position,
            delta,
        });
    }

    pub fn send_key(&self, webview: &Entity, event: cef::KeyEvent) {
        let _ = self.tx.send_blocking(CefCommand::SendKey {
            webview: *webview,
            event,
        });
    }

    // -- IPC ------------------------------------------------------------------

    pub fn emit_event(&self, webview: &Entity, id: impl Into<String>, event: &serde_json::Value) {
        let _ = self.tx.send_blocking(CefCommand::EmitEvent {
            webview: *webview,
            id: id.into(),
            event: event.clone(),
        });
    }

    // -- DevTools -------------------------------------------------------------

    pub fn show_devtool(&self, webview: &Entity) {
        let _ = self
            .tx
            .send_blocking(CefCommand::ShowDevTool { webview: *webview });
    }

    pub fn close_devtools(&self, webview: &Entity) {
        let _ = self
            .tx
            .send_blocking(CefCommand::CloseDevTools { webview: *webview });
    }

    // -- Settings -------------------------------------------------------------

    pub fn set_zoom_level(&self, webview: &Entity, zoom_level: f64) {
        let _ = self.tx.send_blocking(CefCommand::SetZoomLevel {
            webview: *webview,
            zoom_level,
        });
    }

    pub fn set_audio_muted(&self, webview: &Entity, muted: bool) {
        let _ = self.tx.send_blocking(CefCommand::SetAudioMuted {
            webview: *webview,
            muted,
        });
    }

    // -- Reload all -----------------------------------------------------------

    pub fn reload(&self) {
        let _ = self.tx.send_blocking(CefCommand::Reload);
    }

    // -- IME ------------------------------------------------------------------

    pub fn set_ime_composition(&self, text: &str, cursor_utf16: Option<u32>) {
        let _ = self.tx.send_blocking(CefCommand::SetImeComposition {
            text: text.to_owned(),
            cursor_utf16,
        });
    }

    pub fn ime_cancel_composition(&self) {
        let _ = self.tx.send_blocking(CefCommand::ImeCancelComposition);
    }

    pub fn ime_finish_composition(&self, keep_selection: bool) {
        let _ = self
            .tx
            .send_blocking(CefCommand::ImeFinishComposition { keep_selection });
    }

    pub fn set_ime_commit_text(&self, text: &str) {
        let _ = self.tx.send_blocking(CefCommand::SetImeCommitText {
            text: text.to_owned(),
        });
    }

    // -- Frame ----------------------------------------------------------------

    pub fn send_external_begin_frame(&self) {
        let _ = self.tx.send_blocking(CefCommand::SendExternalBeginFrame);
    }
}
