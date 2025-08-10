mod brp_handler;
mod js_emit_event_handler;

use crate::browser_process::ContextMenuHandlerBuilder;
use crate::prelude::IntoString;
use cef::rc::{Rc, RcImpl};
use cef::{
    Browser, Client, ContextMenuHandler, DisplayHandler, Frame, ImplClient, ImplProcessMessage,
    ListValue, ProcessId, ProcessMessage, RenderHandler, WrapClient, sys,
};
use std::os::raw::c_int;

pub use brp_handler::BrpHandler;
pub use js_emit_event_handler::{IpcEventRaw, JsEmitEventHandler};

pub trait ProcessMessageHandler {
    fn process_name(&self) -> &'static str;

    fn handle_message(&self, browser: &mut Browser, frame: &mut Frame, args: Option<ListValue>);
}

/// ## Reference
///
/// - [`CefBrowser Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefBrowser.html)
pub struct ClientHandlerBuilder {
    object: *mut RcImpl<sys::cef_client_t, Self>,
    render_handler: RenderHandler,
    context_menu_handler: ContextMenuHandler,
    message_handlers: Vec<std::rc::Rc<dyn ProcessMessageHandler>>,
    display_handler: Option<DisplayHandler>,
}

impl ClientHandlerBuilder {
    pub fn new(render_handler: RenderHandler) -> Self {
        Self {
            object: std::ptr::null_mut(),
            render_handler,
            context_menu_handler: ContextMenuHandlerBuilder::build(),
            message_handlers: Vec::new(),
            display_handler: None,
        }
    }

    pub fn with_display_handler(mut self, display_handler: DisplayHandler) -> Self {
        self.display_handler = Some(display_handler);
        self
    }

    pub fn with_message_handler(mut self, handler: impl ProcessMessageHandler + 'static) -> Self {
        self.message_handlers.push(std::rc::Rc::new(handler));
        self
    }

    pub fn build(self) -> Client {
        Client::new(self)
    }
}

impl Rc for ClientHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapClient for ClientHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::cef_client_t, Self>) {
        self.object = object;
    }
}

impl Clone for ClientHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self {
            object,
            render_handler: self.render_handler.clone(),
            context_menu_handler: self.context_menu_handler.clone(),
            message_handlers: self.message_handlers.clone(),
            display_handler: self.display_handler.clone(),
        }
    }
}

impl ImplClient for ClientHandlerBuilder {
    fn render_handler(&self) -> Option<RenderHandler> {
        Some(self.render_handler.clone())
    }

    fn display_handler(&self) -> Option<DisplayHandler> {
        self.display_handler.clone()
    }

    fn on_process_message_received(
        &self,
        browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        _: ProcessId,
        message: Option<&mut ProcessMessage>,
    ) -> c_int {
        if let Some(message) = message
            && let Some(browser) = browser
            && let Some(frame) = frame
            && let Some(name) = Some(message.name().into_string())
            && let Some(handler) = self
                .message_handlers
                .iter()
                .find(|h| h.process_name() == name.as_str())
        {
            {
                let args = message.argument_list();
                handler.handle_message(browser, frame, args);
            }
        };
        1
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_client_t {
        self.object.cast()
    }
}
