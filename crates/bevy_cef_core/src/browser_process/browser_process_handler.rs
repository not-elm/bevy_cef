use cef::rc::{Rc, RcImpl};
use cef::*;

/// ## Reference
///
/// - [`CefBrowserProcessHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefBrowserProcessHandler.html)
pub struct BrowserProcessHandlerBuilder {
    object: *mut RcImpl<cef_dll_sys::cef_browser_process_handler_t, Self>,
}

impl BrowserProcessHandlerBuilder {
    pub fn build() -> BrowserProcessHandler {
        BrowserProcessHandler::new(Self {
            object: core::ptr::null_mut(),
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

        Self { object }
    }
}

impl ImplBrowserProcessHandler for BrowserProcessHandlerBuilder {
    fn on_before_child_process_launch(&self, command_line: Option<&mut CommandLine>) {
        let Some(command_line) = command_line else {
            return;
        };

        command_line.append_switch(Some(&"disable-web-security".into()));
        command_line.append_switch(Some(&"allow-running-insecure-content".into()));
        command_line.append_switch(Some(&"disable-session-crashed-bubble".into()));
        command_line.append_switch(Some(&"ignore-certificate-errors".into()));
        command_line.append_switch(Some(&"ignore-ssl-errors".into()));
        command_line.append_switch(Some(&"enable-logging=stderr".into()));
    }
    #[inline]
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_browser_process_handler_t {
        self.object.cast()
    }
}
