//! CEF-thread–resident browser state for Windows `multi_threaded_message_loop`.
//!
//! [`BrowsersCefSide`] holds the actual `!Send` CEF browser objects and lives
//! exclusively on the CEF UI thread.  Bevy systems communicate with it by
//! sending [`CefCommand`]s through an `async_channel`, which are drained each
//! tick via [`drain_commands`].

// Module is already gated by #[cfg(target_os = "windows")] in browser_process.rs

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use async_channel::{Receiver, Sender};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_remote::BrpMessage;
use cef::{
    BrowserSettings, CefString, Client, DictionaryValue, ImplBrowser, ImplBrowserHost,
    ImplDictionaryValue, ImplFrame, ImplListValue, ImplProcessMessage, ImplRequestContext,
    MouseButtonType, ProcessId, Range, RequestContext, RequestContextSettings, WindowInfo,
    browser_host_create_browser_sync, dictionary_value_create, process_message_create,
};
use cef_dll_sys::{cef_event_flags_t, cef_mouse_button_type_t};
#[allow(deprecated)]
use raw_window_handle::RawWindowHandle;

use crate::browser_process::ClientHandlerBuilder;
use crate::browser_process::browsers::devtool_render_handler::DevToolRenderHandlerBuilder;
use crate::browser_process::browsers::{
    WebviewBrowser, make_underlines_for, modifiers_from_mouse_buttons,
};
use crate::browser_process::cef_command::CefCommand;
use crate::browser_process::client_handler::{BrpHandler, IpcEventRaw, JsEmitEventHandler};
use crate::browser_process::display_handler::{DisplayHandlerBuilder, SystemCursorIconSenderInner};
use crate::browser_process::drag_handler::{DragHandlerBuilder, DraggableRegionSenderInner};
use crate::browser_process::localhost::{LocalSchemaHandlerBuilder, Requester};
use crate::browser_process::renderer_handler::{
    RenderHandlerBuilder, RenderTextureMessage, SharedDpr, SharedViewSize, TextureSender,
};
use crate::browser_process::request_context_handler::RequestContextHandlerBuilder;
use crate::prelude::{INIT_SCRIPT_KEY, IntoString, PROCESS_MESSAGE_HOST_EMIT};
use crate::util::{HOST_CEF, SCHEME_CEF};

/// CEF-thread counterpart of `Browsers`.
///
/// This struct is intentionally `!Send` because it holds raw CEF browser
/// objects that must only be touched from the CEF UI thread.
pub struct BrowsersCefSide {
    browsers: HashMap<Entity, WebviewBrowser>,
    texture_sender: TextureSender,
}

impl BrowsersCefSide {
    /// Create a new, empty instance with the given texture delivery channel.
    pub fn new(texture_sender: TextureSender) -> Self {
        Self {
            browsers: HashMap::default(),
            texture_sender,
        }
    }

    /// Drain all pending commands from `rx` and execute them.
    pub fn drain(&mut self, rx: &Receiver<CefCommand>) {
        loop {
            match rx.try_recv() {
                Ok(cmd) => self.execute(cmd),
                Err(async_channel::TryRecvError::Empty) => break,
                Err(async_channel::TryRecvError::Closed) => break,
            }
        }
    }

    fn execute(&mut self, cmd: CefCommand) {
        match cmd {
            CefCommand::CreateBrowser {
                webview,
                uri,
                webview_size,
                initial_dpr,
                requester,
                ipc_event_sender,
                brp_sender,
                system_cursor_icon_sender,
                drag_regions_sender,
                initialize_scripts,
                window_handle,
            } => {
                #[allow(deprecated)]
                let raw_handle = window_handle.map(|h| h.0);
                self.create_browser(
                    webview,
                    &uri,
                    webview_size,
                    initial_dpr,
                    requester,
                    ipc_event_sender,
                    brp_sender,
                    system_cursor_icon_sender,
                    drag_regions_sender,
                    &initialize_scripts,
                    raw_handle,
                );
            }
            CefCommand::Close { entity } => self.close(&entity),
            CefCommand::Navigate { entity, url } => self.navigate(&entity, &url),
            CefCommand::GoBack { entity } => self.go_back(&entity),
            CefCommand::GoForward { entity } => self.go_forward(&entity),
            CefCommand::ReloadWebview { entity } => self.reload_webview(&entity),
            CefCommand::Resize { entity, size } => self.resize(&entity, size),
            CefCommand::SetDpr { entity, dpr } => self.set_dpr(&entity, dpr),
            CefCommand::NotifyScreenInfoChanged { entity } => {
                self.notify_screen_info_changed(&entity)
            }
            CefCommand::SendMouseMove {
                webview,
                buttons,
                position,
                mouse_leave,
            } => self.send_mouse_move(&webview, &buttons, position, mouse_leave),
            CefCommand::SendMouseClick {
                webview,
                position,
                button,
                mouse_up,
            } => self.send_mouse_click(&webview, position, button, mouse_up),
            CefCommand::SendMouseWheel {
                webview,
                position,
                delta,
            } => self.send_mouse_wheel(&webview, position, delta),
            CefCommand::SendKey { webview, event } => self.send_key(&webview, event),
            CefCommand::EmitEvent { webview, id, event } => {
                self.emit_event(&webview, id, &event);
            }
            CefCommand::ShowDevTool { webview } => self.show_devtool(&webview),
            CefCommand::CloseDevTools { webview } => self.close_devtools(&webview),
            CefCommand::SetZoomLevel {
                webview,
                zoom_level,
            } => self.set_zoom_level(&webview, zoom_level),
            CefCommand::SetAudioMuted { webview, muted } => {
                self.set_audio_muted(&webview, muted);
            }
            CefCommand::Reload => self.reload(),
            CefCommand::SetImeComposition { text, cursor_utf16 } => {
                self.set_ime_composition(&text, cursor_utf16)
            }
            CefCommand::ImeCancelComposition => self.ime_cancel_composition(),
            CefCommand::ImeFinishComposition { keep_selection } => {
                self.ime_finish_composition(keep_selection);
            }
            CefCommand::SetImeCommitText { text } => self.set_ime_commit_text(&text),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_browser(
        &mut self,
        webview: Entity,
        uri: &str,
        webview_size: Vec2,
        initial_dpr: f32,
        requester: Requester,
        ipc_event_sender: Sender<IpcEventRaw>,
        brp_sender: Sender<BrpMessage>,
        system_cursor_icon_sender: SystemCursorIconSenderInner,
        drag_regions_sender: DraggableRegionSenderInner,
        initialize_scripts: &[String],
        #[allow(deprecated)] _window_handle: Option<RawWindowHandle>,
    ) {
        let mut context = Self::request_context(requester);
        let size: SharedViewSize = Arc::new(Mutex::new(webview_size));
        let dpr: SharedDpr = Arc::new(Mutex::new(initial_dpr));
        let browser = browser_host_create_browser_sync(
            Some(&WindowInfo {
                windowless_rendering_enabled: true as _,
                external_begin_frame_enabled: false as _,
                #[allow(deprecated)]
                parent_window: match _window_handle {
                    Some(RawWindowHandle::Win32(handle)) => {
                        cef_dll_sys::HWND(handle.hwnd.get() as _)
                    }
                    _ => cef_dll_sys::HWND(std::ptr::null_mut()),
                },
                ..Default::default()
            }),
            Some(&mut self.client_handler(
                webview,
                size.clone(),
                dpr.clone(),
                ipc_event_sender,
                brp_sender,
                system_cursor_icon_sender,
                drag_regions_sender,
            )),
            Some(&uri.into()),
            Some(&BrowserSettings {
                windowless_frame_rate: 60,
                ..Default::default()
            }),
            Self::create_extra_info(initialize_scripts).as_mut(),
            context.as_mut(),
        )
        .expect("Failed to create browser");
        let host = browser.host().expect("Failed to get browser host");
        let webview_browser = WebviewBrowser {
            host,
            client: browser,
            size,
            dpr,
        };
        self.browsers.insert(webview, webview_browser);
    }

    fn close(&mut self, entity: &Entity) {
        if let Some(browser) = self.browsers.remove(entity) {
            browser.host.close_browser(true as _);
            debug!("Closed browser with webview: {:?}", entity);
        }
    }

    fn navigate(&self, entity: &Entity, url: &str) {
        if let Some(browser) = self.browsers.get(entity)
            && let Some(frame) = browser.client.main_frame()
        {
            frame.load_url(Some(&url.into()));
        }
    }

    fn go_back(&self, entity: &Entity) {
        if let Some(browser) = self.browsers.get(entity)
            && browser.client.can_go_back() == 1
        {
            browser.client.go_back();
        }
    }

    fn go_forward(&self, entity: &Entity) {
        if let Some(browser) = self.browsers.get(entity)
            && browser.client.can_go_forward() == 1
        {
            browser.client.go_forward();
        }
    }

    fn reload_webview(&self, entity: &Entity) {
        if let Some(browser) = self.browsers.get(entity)
            && let Some(frame) = browser.client.main_frame()
        {
            let url = frame.url().into_string();
            frame.load_url(Some(&url.as_str().into()));
        }
    }

    fn resize(&self, entity: &Entity, size: Vec2) {
        if let Some(browser) = self.browsers.get(entity) {
            *browser.size.lock().unwrap() = size;
            browser.host.was_resized();
        }
    }

    /// Updates the `SharedDpr` slot that `screen_info` reads.
    ///
    /// Call this *before* [`Self::notify_screen_info_changed`] — the latter causes
    /// CEF to immediately re-query `GetScreenInfo`, which reads from this slot.
    fn set_dpr(&self, webview: &Entity, dpr: f32) {
        if let Some(browser) = self.browsers.get(webview) {
            *browser.dpr.lock().unwrap() = dpr;
        }
    }

    /// Tell CEF to re-query screen info and force Blink to reflow at the new DPR.
    ///
    /// `notify_screen_info_changed` alone updates Chromium's cached screen
    /// metrics but does not run `ResizeRootLayer` / `SynchronizeVisualProperties`.
    /// Only `was_resized()` pushes new `VisualProperties` (including the new
    /// `device_scale_factor`) to Blink. Without the pair, the CSS viewport
    /// ends up laid out as `view_rect × DSF` DIP wide and on-screen text
    /// shrinks by exactly `1/DSF`. Matches the cefclient OSR convention
    /// (`tests/cefclient/browser/osr_window_win.cc::SetDeviceScaleFactor`).
    fn notify_screen_info_changed(&self, webview: &Entity) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.host.notify_screen_info_changed();
            browser.host.was_resized();
        }
    }

    fn send_mouse_move(
        &self,
        webview: &Entity,
        buttons: &[MouseButton],
        position: Vec2,
        mouse_leave: bool,
    ) {
        if let Some(browser) = self.get_focused_browser(webview) {
            let mouse_event = cef::MouseEvent {
                x: position.x as i32,
                y: position.y as i32,
                modifiers: modifiers_from_mouse_buttons(buttons.iter()),
            };
            browser
                .host
                .send_mouse_move_event(Some(&mouse_event), mouse_leave as _);
        }
    }

    fn send_mouse_click(
        &self,
        webview: &Entity,
        position: Vec2,
        button: PointerButton,
        mouse_up: bool,
    ) {
        if let Some(browser) = self.get_focused_browser(webview) {
            let mouse_event = cef::MouseEvent {
                x: position.x as i32,
                y: position.y as i32,
                modifiers: match button {
                    PointerButton::Primary => cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON.0,
                    PointerButton::Secondary => cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON.0,
                    PointerButton::Middle => cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON.0,
                } as _,
            };
            let mouse_button = match button {
                PointerButton::Secondary => cef_mouse_button_type_t::MBT_RIGHT,
                PointerButton::Middle => cef_mouse_button_type_t::MBT_MIDDLE,
                _ => cef_mouse_button_type_t::MBT_LEFT,
            };
            browser.host.set_focus(true as _);
            browser.host.send_mouse_click_event(
                Some(&mouse_event),
                MouseButtonType::from(mouse_button),
                mouse_up as _,
                1,
            );
        }
    }

    fn send_mouse_wheel(&self, webview: &Entity, position: Vec2, delta: Vec2) {
        if let Some(browser) = self.get_focused_browser(webview) {
            let mouse_event = cef::MouseEvent {
                x: position.x as i32,
                y: position.y as i32,
                modifiers: 0,
            };
            browser
                .host
                .send_mouse_wheel_event(Some(&mouse_event), delta.x as _, delta.y as _);
        }
    }

    fn send_key(&self, webview: &Entity, event: cef::KeyEvent) {
        if let Some(browser) = self.get_focused_browser(webview) {
            browser.host.send_key_event(Some(&event));
        }
    }

    fn emit_event(&self, webview: &Entity, id: impl Into<String>, event: &serde_json::Value) {
        if let Some(mut process_message) =
            process_message_create(Some(&PROCESS_MESSAGE_HOST_EMIT.into()))
            && let Some(argument_list) = process_message.argument_list()
            && let Some(browser) = self.browsers.get(webview)
            && let Some(frame) = browser.client.main_frame()
        {
            argument_list.set_string(0, Some(&id.into().as_str().into()));
            argument_list.set_string(1, Some(&event.to_string().as_str().into()));
            frame.send_process_message(
                ProcessId::from(cef_dll_sys::cef_process_id_t::PID_RENDERER),
                Some(&mut process_message),
            );
        };
    }

    fn show_devtool(&self, webview: &Entity) {
        let Some(browser) = self.browsers.get(webview) else {
            return;
        };
        browser.host.show_dev_tools(
            Some(&WindowInfo::default()),
            Some(&mut ClientHandlerBuilder::new(DevToolRenderHandlerBuilder::build()).build()),
            Some(&BrowserSettings::default()),
            None,
        );
    }

    fn close_devtools(&self, webview: &Entity) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.host.close_dev_tools();
        }
    }

    fn set_zoom_level(&self, webview: &Entity, zoom_level: f64) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.host.set_zoom_level(zoom_level);
        }
    }

    fn set_audio_muted(&self, webview: &Entity, muted: bool) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.host.set_audio_muted(muted as _);
        }
    }

    fn reload(&self) {
        for browser in self.browsers.values() {
            if let Some(frame) = browser.client.main_frame() {
                let url = frame.url().into_string();
                info!("Reloading browser with URL: {}", url);
                frame.load_url(Some(&url.as_str().into()));
            }
        }
    }

    fn set_ime_composition(&self, text: &str, cursor_utf16: Option<u32>) {
        let underlines = make_underlines_for(text, cursor_utf16.map(|i| (i, i)));
        let i = text.encode_utf16().count();
        let selection_range = Range {
            from: i as _,
            to: i as _,
        };
        for browser in self
            .browsers
            .values()
            .filter(|b| b.client.focused_frame().is_some())
        {
            let replacement_range = Self::ime_caret_range();
            browser.host.ime_set_composition(
                Some(&text.into()),
                Some(&underlines),
                Some(&replacement_range),
                Some(&selection_range),
            );
        }
    }

    fn ime_cancel_composition(&self) {
        for browser in self
            .browsers
            .values()
            .filter(|b| b.client.focused_frame().is_some())
        {
            browser.host.ime_cancel_composition();
        }
    }

    fn ime_finish_composition(&self, keep_selection: bool) {
        for browser in self
            .browsers
            .values()
            .filter(|b| b.client.focused_frame().is_some())
        {
            browser.host.ime_finish_composing_text(keep_selection as _);
        }
    }

    fn set_ime_commit_text(&self, text: &str) {
        for browser in self
            .browsers
            .values()
            .filter(|b| b.client.focused_frame().is_some())
        {
            let replacement_range = Self::ime_caret_range();
            browser
                .host
                .ime_commit_text(Some(&text.into()), Some(&replacement_range), 0);
        }
    }

    fn request_context(requester: Requester) -> Option<RequestContext> {
        let mut context = cef::request_context_create_context(
            Some(&RequestContextSettings::default()),
            Some(&mut RequestContextHandlerBuilder::build()),
        );
        if let Some(context) = context.as_mut() {
            context.register_scheme_handler_factory(
                Some(&SCHEME_CEF.into()),
                Some(&HOST_CEF.into()),
                Some(&mut LocalSchemaHandlerBuilder::build(requester)),
            );
        }
        context
    }

    #[allow(clippy::too_many_arguments)]
    fn client_handler(
        &self,
        webview: Entity,
        size: SharedViewSize,
        dpr: SharedDpr,
        ipc_event_sender: Sender<IpcEventRaw>,
        brp_sender: Sender<BrpMessage>,
        system_cursor_icon_sender: SystemCursorIconSenderInner,
        drag_regions_sender: DraggableRegionSenderInner,
    ) -> Client {
        ClientHandlerBuilder::new(RenderHandlerBuilder::build(
            webview,
            self.texture_sender.clone(),
            size.clone(),
            dpr,
        ))
        .with_display_handler(DisplayHandlerBuilder::build(system_cursor_icon_sender))
        .with_drag_handler(DragHandlerBuilder::build(webview, drag_regions_sender))
        .with_message_handler(JsEmitEventHandler::new(webview, ipc_event_sender))
        .with_message_handler(BrpHandler::new(brp_sender))
        .build()
    }

    fn create_extra_info(scripts: &[String]) -> Option<DictionaryValue> {
        if scripts.is_empty() {
            return None;
        }
        let extra = dictionary_value_create()?;
        extra.set_string(
            Some(&CefString::from(INIT_SCRIPT_KEY)),
            Some(&CefString::from(scripts.join(";").as_str())),
        );
        Some(extra)
    }

    #[inline]
    fn get_focused_browser(&self, webview: &Entity) -> Option<&WebviewBrowser> {
        self.browsers
            .get(webview)
            .and_then(|b| b.client.focused_frame().is_some().then_some(b))
    }

    #[inline]
    fn ime_caret_range() -> Range {
        Range {
            from: u32::MAX,
            to: u32::MAX,
        }
    }
}

thread_local! {
    static CEF_BROWSERS: RefCell<Option<BrowsersCefSide>> = const { RefCell::new(None) };
}

/// Initialise the thread-local [`BrowsersCefSide`] on the CEF UI thread.
///
/// Must be called exactly once, from the CEF UI thread, before any calls to
/// [`drain_commands`].
pub fn init_cef_browsers(texture_sender: Sender<RenderTextureMessage>) {
    CEF_BROWSERS.with(|b| {
        *b.borrow_mut() = Some(BrowsersCefSide::new(texture_sender));
    });
}

/// Drain all pending [`CefCommand`]s and execute them on the thread-local
/// [`BrowsersCefSide`].
///
/// This must be called from the CEF UI thread (the same thread that called
/// [`init_cef_browsers`]).
pub fn drain_commands(rx: &Receiver<CefCommand>) {
    CEF_BROWSERS.with(|b| {
        if let Some(browsers) = b.borrow_mut().as_mut() {
            browsers.drain(rx);
        }
    });
}
