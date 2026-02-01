use crate::prelude::RenderProcessHandlerBuilder;
use crate::util::{SCHEME_CEF, cef_scheme_flags};
use cef::rc::{Rc, RcImpl};
use cef::{ImplApp, ImplSchemeRegistrar, RenderProcessHandler, SchemeRegistrar, WrapApp};
use cef_dll_sys::{_cef_app_t, cef_base_ref_counted_t};

#[derive(Default)]
pub struct RenderProcessAppBuilder {
    object: *mut RcImpl<_cef_app_t, Self>,
}

impl RenderProcessAppBuilder {
    pub fn build() -> cef::App {
        cef::App::new(Self {
            object: core::ptr::null_mut(),
        })
    }
}

impl Clone for RenderProcessAppBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            self.object
        };
        Self { object }
    }
}

impl Rc for RenderProcessAppBuilder {
    fn as_base(&self) -> &cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplApp for RenderProcessAppBuilder {
    fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
        if let Some(registrar) = registrar {
            registrar.add_custom_scheme(Some(&SCHEME_CEF.into()), cef_scheme_flags() as _);
        }
    }

    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
        Some(RenderProcessHandler::new(
            RenderProcessHandlerBuilder::build(),
        ))
    }

    #[inline]
    fn get_raw(&self) -> *mut _cef_app_t {
        self.object as *mut _cef_app_t
    }
}

impl WrapApp for RenderProcessAppBuilder {
    #[inline]
    fn wrap_rc(&mut self, object: *mut RcImpl<_cef_app_t, Self>) {
        self.object = object;
    }
}
