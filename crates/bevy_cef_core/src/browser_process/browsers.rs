use crate::browser_process::BrpHandler;
use crate::browser_process::ClientHandlerBuilder;
use crate::browser_process::client_handler::{IpcEventRaw, JsEmitEventHandler};
use crate::prelude::IntoString;
use crate::prelude::*;
use async_channel::{Sender, TryRecvError};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_remote::BrpMessage;
use cef::{
    Browser, BrowserHost, BrowserSettings, CefString, Client, CompositionUnderline,
    DictionaryValue, ImplBrowser, ImplBrowserHost, ImplDictionaryValue, ImplFrame, ImplListValue,
    ImplProcessMessage, ImplRequestContext, MouseButtonType, ProcessId, Range, RequestContext,
    RequestContextSettings, WindowInfo, browser_host_create_browser_sync, dictionary_value_create,
    process_message_create,
};
use cef_dll_sys::{cef_event_flags_t, cef_mouse_button_type_t};
#[allow(deprecated)]
use raw_window_handle::RawWindowHandle;
use std::cell::Cell;
use std::rc::Rc;

mod devtool_render_handler;
mod keyboard;

use crate::browser_process::browsers::devtool_render_handler::DevToolRenderHandlerBuilder;
use crate::browser_process::display_handler::{DisplayHandlerBuilder, SystemCursorIconSenderInner};
pub use keyboard::*;

pub struct WebviewBrowser {
    pub client: Browser,
    pub host: BrowserHost,
    pub size: SharedViewSize,
}

pub struct Browsers {
    browsers: HashMap<Entity, WebviewBrowser>,
    sender: TextureSender,
    receiver: TextureReceiver,
    ime_caret: SharedImeCaret,
}

impl Default for Browsers {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded::<RenderTexture>();
        Browsers {
            browsers: HashMap::default(),
            sender,
            receiver,
            ime_caret: Rc::new(Cell::new(0)),
        }
    }
}

impl Browsers {
    #[allow(clippy::too_many_arguments)]
    pub fn create_browser(
        &mut self,
        webview: Entity,
        uri: &str,
        webview_size: Vec2,
        requester: Requester,
        ipc_event_sender: Sender<IpcEventRaw>,
        brp_sender: Sender<BrpMessage>,
        system_cursor_icon_sender: SystemCursorIconSenderInner,
        initialize_scripts: &[String],
        _window_handle: Option<RawWindowHandle>,
    ) {
        let mut context = Self::request_context(requester);
        let size = Rc::new(Cell::new(webview_size));
        let browser = browser_host_create_browser_sync(
            Some(&WindowInfo {
                windowless_rendering_enabled: true as _,
                external_begin_frame_enabled: true as _,
                #[cfg(target_os = "macos")]
                parent_view: match _window_handle {
                    Some(RawWindowHandle::AppKit(handle)) => handle.ns_view.as_ptr(),
                    Some(RawWindowHandle::Win32(handle)) => handle.hwnd.get() as _,
                    Some(RawWindowHandle::Xlib(handle)) => handle.window as _,
                    Some(RawWindowHandle::Wayland(handle)) => handle.surface.as_ptr(),
                    _ => std::ptr::null_mut(),
                },
                // shared_texture_enabled: true as _,
                ..Default::default()
            }),
            Some(&mut self.client_handler(
                webview,
                size.clone(),
                ipc_event_sender,
                brp_sender,
                system_cursor_icon_sender,
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

        self.browsers.insert(
            webview,
            WebviewBrowser {
                host: browser.host().expect("Failed to get browser host"),
                client: browser,
                size,
            },
        );
    }

    pub fn send_external_begin_frame(&mut self) {
        for browser in self.browsers.values_mut() {
            browser.host.send_external_begin_frame();
        }
    }

    pub fn send_mouse_move<'a>(
        &self,
        webview: &Entity,
        buttons: impl IntoIterator<Item = &'a MouseButton>,
        position: Vec2,
        mouse_leave: bool,
    ) {
        if let Some(browser) = self.get_focused_browser(webview) {
            let mouse_event = cef::MouseEvent {
                x: position.x as i32,
                y: position.y as i32,
                modifiers: modifiers_from_mouse_buttons(buttons),
            };
            browser
                .host
                .send_mouse_move_event(Some(&mouse_event), mouse_leave as _);
        }
    }

    pub fn send_mouse_click(
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
                    PointerButton::Primary => cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON,
                    PointerButton::Secondary => cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON,
                    PointerButton::Middle => cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON,
                } as _, // No modifiers for simplicity
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

    /// [`SendMouseWheelEvent`](https://cef-builds.spotifycdn.com/docs/106.1/classCefBrowserHost.html#acd5d057bd5230baa9a94b7853ba755f7)
    pub fn send_mouse_wheel(&self, webview: &Entity, position: Vec2, delta: Vec2) {
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

    #[inline]
    pub fn send_key(&self, webview: &Entity, event: cef::KeyEvent) {
        if let Some(browser) = self.get_focused_browser(webview) {
            browser.host.send_key_event(Some(&event));
        }
    }

    pub fn emit_event(&self, webview: &Entity, id: impl Into<String>, event: &serde_json::Value) {
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

    pub fn resize(&self, webview: &Entity, size: Vec2) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.size.set(size);
            browser.host.was_resized();
        }
    }

    /// Closes the browser associated with the given webview entity.
    ///
    /// The browser will be removed from the hash map after closing.
    pub fn close(&mut self, webview: &Entity) {
        if let Some(browser) = self.browsers.remove(webview) {
            browser.host.close_browser(true as _);
            debug!("Closed browser with webview: {:?}", webview);
        }
    }

    #[inline]
    pub fn try_receive_texture(&self) -> core::result::Result<RenderTexture, TryRecvError> {
        self.receiver.try_recv()
    }

    /// Shows the DevTools for the specified webview.
    pub fn show_devtool(&self, webview: &Entity) {
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

    /// Closes the DevTools for the specified webview.
    pub fn close_devtools(&self, webview: &Entity) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.host.close_dev_tools();
        }
    }

    /// Navigate backwards.
    ///
    /// ## Reference
    ///
    /// - [`GoBack`](https://cef-builds.spotifycdn.com/docs/122.0/classCefBrowser.html#a85b02760885c070e4ad2a2705cea56cb)
    pub fn go_back(&self, webview: &Entity) {
        if let Some(browser) = self.browsers.get(webview)
            && browser.client.can_go_back() == 1
        {
            browser.client.go_back();
        }
    }

    /// Navigate forwards.
    ///
    /// ## Reference
    ///
    /// - [`GoForward`](https://cef-builds.spotifycdn.com/docs/122.0/classCefBrowser.html#aa8e97fc210ee0e73f16b2d98482419d0)
    pub fn go_forward(&self, webview: &Entity) {
        if let Some(browser) = self.browsers.get(webview)
            && browser.client.can_go_forward() == 1
        {
            browser.client.go_forward();
        }
    }

    /// Returns the current zoom level for the specified webview.
    ///
    /// ## Reference
    ///
    /// - [`GetZoomLevel`](https://cef-builds.spotifycdn.com/docs/122.0/classCefBrowserHost.html#a524d4a358287dab284c0dfec6d6d229e)
    pub fn zoom_level(&self, webview: &Entity) -> Option<f64> {
        self.browsers
            .get(webview)
            .map(|browser| browser.host.zoom_level())
    }

    /// Sets the zoom level for the specified webview.
    ///
    /// ## Reference
    ///
    /// - [`SetZoomLevel`](https://cef-builds.spotifycdn.com/docs/122.0/classCefBrowserHost.html#af2b7bf250ac78345117cd575190f2f7b)
    pub fn set_zoom_level(&self, webview: &Entity, zoom_level: f64) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.host.set_zoom_level(zoom_level);
        }
    }

    /// Sets whether the audio is muted for the specified webview.
    ///
    /// ## Reference
    ///
    /// - [`SetAudioMuted`](https://cef-builds.spotifycdn.com/docs/122.0/classCefBrowserHost.html#a153d179c9ff202c8bb8869d2e9a820a2)
    pub fn set_audio_muted(&self, webview: &Entity, muted: bool) {
        if let Some(browser) = self.browsers.get(webview) {
            browser.host.set_audio_muted(muted as _);
        }
    }

    #[inline]
    pub fn reload(&self) {
        for browser in self.browsers.values() {
            if let Some(frame) = browser.client.main_frame() {
                let url = frame.url().into_string();
                info!("Reloading browser with URL: {}", url);
                frame.load_url(Some(&url.as_str().into()));
            }
        }
    }

    /// ## Reference
    ///
    /// - [`ImeSetComposition`](https://cef-builds.spotifycdn.com/docs/122.0/classCefBrowserHost.html#a567b41fb2d3917843ece3b57adc21ebe)
    pub fn set_ime_composition(&self, text: &str, cursor_utf16: Option<u32>) {
        let underlines = make_underlines_for(text, cursor_utf16.map(|i| (i, i)));
        let i = text.encode_utf16().count();
        let selection_range = Range {
            from: i as _,
            to: i as _,
        };
        let replacement_range = self.ime_caret_range();
        for browser in self
            .browsers
            .values()
            .filter(|b| b.client.focused_frame().is_some())
        {
            browser.host.ime_set_composition(
                Some(&text.into()),
                underlines.len(),
                Some(&underlines[0]),
                Some(&replacement_range),
                Some(&selection_range),
            );
        }
    }

    /// ## Reference
    ///
    /// [`ImeSetComposition`](https://cef-builds.spotifycdn.com/docs/122.0/classCefBrowserHost.html#a567b41fb2d3917843ece3b57adc21ebe)
    pub fn ime_finish_composition(&self, keep_selection: bool) {
        for browser in self
            .browsers
            .values()
            .filter(|b| b.client.focused_frame().is_some())
        {
            browser.host.ime_finish_composing_text(keep_selection as _);
        }
    }

    pub fn set_ime_commit_text(&self, text: &str) {
        let replacement_range = self.ime_caret_range();
        for browser in self
            .browsers
            .values()
            .filter(|b| b.client.focused_frame().is_some())
        {
            browser
                .host
                .ime_commit_text(Some(&text.into()), Some(&replacement_range), 0)
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

    fn client_handler(
        &self,
        webview: Entity,
        size: SharedViewSize,
        ipc_event_sender: Sender<IpcEventRaw>,
        brp_sender: Sender<BrpMessage>,
        system_cursor_icon_sender: SystemCursorIconSenderInner,
    ) -> Client {
        ClientHandlerBuilder::new(RenderHandlerBuilder::build(
            webview,
            self.sender.clone(),
            size.clone(),
            self.ime_caret.clone(),
        ))
        .with_display_handler(DisplayHandlerBuilder::build(system_cursor_icon_sender))
        .with_message_handler(JsEmitEventHandler::new(webview, ipc_event_sender))
        .with_message_handler(BrpHandler::new(brp_sender))
        .build()
    }

    #[inline]
    fn ime_caret_range(&self) -> Range {
        let caret = self.ime_caret.get();
        Range {
            from: caret,
            to: caret,
        }
    }

    #[inline]
    fn get_focused_browser(&self, webview: &Entity) -> Option<&WebviewBrowser> {
        self.browsers
            .get(webview)
            .and_then(|b| b.client.focused_frame().is_some().then_some(b))
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
}

pub fn modifiers_from_mouse_buttons<'a>(buttons: impl IntoIterator<Item = &'a MouseButton>) -> u32 {
    let mut modifiers = cef_event_flags_t::EVENTFLAG_NONE as u32;
    for button in buttons {
        match button {
            MouseButton::Left => modifiers |= cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON as u32,
            MouseButton::Right => {
                modifiers |= cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON as u32
            }
            MouseButton::Middle => {
                modifiers |= cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON as u32
            }
            _ => {}
        }
    }
    modifiers
}

pub fn make_underlines_for(
    text: &str,
    selection_utf16: Option<(u32, u32)>,
) -> Vec<CompositionUnderline> {
    let len16 = utf16_len(text);

    let base = CompositionUnderline {
        size: size_of::<CompositionUnderline>(),
        range: Range { from: 0, to: len16 },
        color: 0,
        background_color: 0,
        thick: 0,
        style: Default::default(),
    };

    if let Some((from, to)) = selection_utf16
        && from < to
    {
        let sel = CompositionUnderline {
            size: size_of::<CompositionUnderline>(),
            range: Range { from, to },
            color: 0,
            background_color: 0,
            thick: 1,
            style: Default::default(),
        };
        return vec![base, sel];
    }
    vec![base]
}

#[inline]
fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

#[allow(dead_code)]
fn utf16_index_from_byte(s: &str, byte_idx: usize) -> u32 {
    s[..byte_idx].encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
    use crate::prelude::modifiers_from_mouse_buttons;
    use bevy::prelude::*;

    #[test]
    fn test_modifiers_from_mouse_buttons() {
        let buttons = vec![&MouseButton::Left, &MouseButton::Right];
        let modifiers = modifiers_from_mouse_buttons(buttons);
        assert_eq!(
            modifiers,
            cef_dll_sys::cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON as u32
                | cef_dll_sys::cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON as u32
        );
    }
}
