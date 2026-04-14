//! CEF LoadHandler — receives page load lifecycle events from the browser.
//!
//! Mirrors the `DragHandlerBuilder` pattern (`drag_handler.rs`).

use async_channel::Sender;
use bevy::prelude::Entity;
use cef::rc::{Rc, RcImpl};
use cef::{
    Browser, CefString, Errorcode, Frame, ImplFrame, ImplLoadHandler, WrapLoadHandler, sys,
};
use std::os::raw::c_int;

/// Messages sent from the CEF load handler to the Bevy drain system.
pub enum LoadHandlerMessage {
    /// The browser's loading state has changed.
    LoadingStateChanged {
        webview: Entity,
        is_loading: bool,
        can_go_back: bool,
        can_go_forward: bool,
    },
    /// The main frame finished loading.
    Finished {
        webview: Entity,
        http_status_code: i32,
    },
    /// The main frame failed to load.
    Error {
        webview: Entity,
        error_code: i32,
        url: String,
    },
}

pub type LoadHandlerSenderInner = Sender<LoadHandlerMessage>;

/// ## Reference
///
/// - [`CefLoadHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/145/classCefLoadHandler.html)
pub struct LoadHandlerBuilder {
    object: *mut RcImpl<sys::_cef_load_handler_t, Self>,
    webview: Entity,
    sender: LoadHandlerSenderInner,
}

impl LoadHandlerBuilder {
    pub fn build(webview: Entity, sender: LoadHandlerSenderInner) -> cef::LoadHandler {
        cef::LoadHandler::new(Self {
            object: core::ptr::null_mut(),
            webview,
            sender,
        })
    }
}

impl Rc for LoadHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            core::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for LoadHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self {
            object,
            webview: self.webview,
            sender: self.sender.clone(),
        }
    }
}

impl WrapLoadHandler for LoadHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_load_handler_t, Self>) {
        self.object = object;
    }
}

impl ImplLoadHandler for LoadHandlerBuilder {
    fn on_loading_state_change(
        &self,
        _browser: Option<&mut Browser>,
        is_loading: c_int,
        can_go_back: c_int,
        can_go_forward: c_int,
    ) {
        let _ = self
            .sender
            .send_blocking(LoadHandlerMessage::LoadingStateChanged {
                webview: self.webview,
                is_loading: is_loading != 0,
                can_go_back: can_go_back != 0,
                can_go_forward: can_go_forward != 0,
            });
    }

    fn on_load_end(
        &self,
        _browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        http_status_code: c_int,
    ) {
        if let Some(frame) = frame
            && frame.is_main() != 0
        {
            let _ = self.sender.send_blocking(LoadHandlerMessage::Finished {
                webview: self.webview,
                http_status_code,
            });
        }
    }

    fn on_load_error(
        &self,
        _browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        error_code: Errorcode,
        _error_text: Option<&CefString>,
        failed_url: Option<&CefString>,
    ) {
        if let Some(frame) = frame
            && frame.is_main() != 0
        {
            let raw: cef_dll_sys::cef_errorcode_t = error_code.into();
            let _ = self.sender.send_blocking(LoadHandlerMessage::Error {
                webview: self.webview,
                error_code: raw as i32,
                url: failed_url.map(|u| u.to_string()).unwrap_or_default(),
            });
        }
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_load_handler_t {
        self.object.cast()
    }
}
