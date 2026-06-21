use crate::prelude::{CefExtensions, CommandLineConfig, EXTENSIONS_SWITCH, MessageLoopTimer};
use cef::rc::{Rc, RcImpl};
use cef::*;
use std::sync::mpsc::Sender;

/// ## Reference
///
/// - [`CefBrowserProcessHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefBrowserProcessHandler.html)
pub struct BrowserProcessHandlerBuilder {
    object: *mut RcImpl<cef_dll_sys::cef_browser_process_handler_t, Self>,
    message_loop_working_requester: Sender<MessageLoopTimer>,
    config: CommandLineConfig,
    extensions: CefExtensions,
}

impl BrowserProcessHandlerBuilder {
    pub fn build(
        message_loop_working_requester: Sender<MessageLoopTimer>,
        config: CommandLineConfig,
        extensions: CefExtensions,
    ) -> BrowserProcessHandler {
        BrowserProcessHandler::new(Self {
            object: core::ptr::null_mut(),
            message_loop_working_requester,
            config,
            extensions,
        })
    }
}

impl Rc for BrowserProcessHandlerBuilder {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapBrowserProcessHandler for BrowserProcessHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_browser_process_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for BrowserProcessHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self {
            object,
            message_loop_working_requester: self.message_loop_working_requester.clone(),
            config: self.config.clone(),
            extensions: self.extensions.clone(),
        }
    }
}

impl ImplBrowserProcessHandler for BrowserProcessHandlerBuilder {
    fn on_before_child_process_launch(&self, command_line: Option<&mut CommandLine>) {
        let Some(command_line) = command_line else {
            return;
        };

        // Forward user-configured switches to every child process. Chromium enforces
        // CORS / web-security in the network (utility) process under NetworkService,
        // so forwarding to all children — not just the renderer — is required for an
        // opt-in like `disable-web-security` to take effect.
        for switch in &self.config.switches {
            command_line.append_switch(Some(&(*switch).into()));
        }

        // Pass extensions to the render process via command line.
        if !self.extensions.is_empty()
            && let Ok(json) = serde_json::to_string(&self.extensions.0)
        {
            command_line.append_switch_with_value(
                Some(&EXTENSIONS_SWITCH.into()),
                Some(&json.as_str().into()),
            );
        }
        // NOTE: The custom-scheme switch MUST be injected here, not in
        // `App::on_before_command_line_processing`. This hook fires for every
        // child process type (GPU, renderer, and the out-of-process
        // utility/Network Service); `on_before_command_line_processing` only runs
        // for the browser process here (`process_type` is always `None`). Moving
        // it there leaves the Network Service without the scheme and floods
        // `network.mojom.NetworkContext` validation errors. Verified empirically.
        if let Some(json) = crate::custom_scheme::current_scheme_decls_json() {
            command_line.append_switch_with_value(
                Some(&crate::util::CUSTOM_SCHEMES_SWITCH.into()),
                Some(&json.as_str().into()),
            );
        }
    }

    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
        let _ = self
            .message_loop_working_requester
            .send(MessageLoopTimer::new(delay_ms));
    }

    #[inline]
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_browser_process_handler_t {
        self.object.cast()
    }
}
