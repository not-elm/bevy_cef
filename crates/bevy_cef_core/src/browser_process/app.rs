use crate::browser_process::browser_process_handler::BrowserProcessHandlerBuilder;
use crate::util::{SCHEME_CEF, cef_scheme_flags};
use cef::rc::{Rc, RcImpl};
use cef::{
    BrowserProcessHandler, CefString, CommandLine, ImplApp, ImplCommandLine, ImplSchemeRegistrar,
    SchemeRegistrar, WrapApp,
};
use cef_dll_sys::{_cef_app_t, cef_base_ref_counted_t};

/// ## Reference
///
/// - [`CefApp Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefApp.html)
#[derive(Default)]
pub struct BrowserProcessAppBuilder {
    object: *mut RcImpl<_cef_app_t, Self>,
}

impl BrowserProcessAppBuilder {
    pub fn build() -> cef::App {
        cef::App::new(Self {
            object: core::ptr::null_mut(),
        })
    }
}

impl Clone for BrowserProcessAppBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            self.object
        };
        Self { object }
    }
}

impl Rc for BrowserProcessAppBuilder {
    fn as_base(&self) -> &cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplApp for BrowserProcessAppBuilder {
    fn on_before_command_line_processing(
        &self,
        _: Option<&CefString>,
        command_line: Option<&mut CommandLine>,
    ) {
        let Some(command_line) = command_line else {
            return;
        };
        //TODO: フラグで切り替えるようにする
        command_line.append_switch(Some(&"use-mock-keychain".into()));
        #[cfg(feature = "debug")]
        {
            command_line.append_switch(Some(&"disable-gpu".into()));
            command_line.append_switch(Some(&"disable-gpu-compositing".into()));
            command_line.append_switch(Some(&"disable-software-rasterizer".into()));
        }
    }

    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(BrowserProcessHandlerBuilder::build())
    }

    fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
        if let Some(registrar) = registrar {
            registrar.add_custom_scheme(Some(&SCHEME_CEF.into()), cef_scheme_flags() as _);
        }
    }

    #[inline]
    fn get_raw(&self) -> *mut _cef_app_t {
        self.object as *mut cef::sys::_cef_app_t
    }
}

impl WrapApp for BrowserProcessAppBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<_cef_app_t, Self>) {
        self.object = object;
    }
}
