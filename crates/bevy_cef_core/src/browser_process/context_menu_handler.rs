use cef::rc::{Rc, RcImpl};
use cef::{ImplContextMenuHandler, WrapContextMenuHandler, sys};

/// ## Reference
///
/// - [`CefContextMenuHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefContextMenuHandler.html)
pub struct ContextMenuHandlerBuilder {
    object: *mut RcImpl<sys::_cef_context_menu_handler_t, Self>,
}

impl ContextMenuHandlerBuilder {
    pub fn build() -> cef::ContextMenuHandler {
        cef::ContextMenuHandler::new(Self {
            object: core::ptr::null_mut(),
        })
    }
}

impl WrapContextMenuHandler for ContextMenuHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_context_menu_handler_t, Self>) {
        self.object = object;
    }
}

impl Rc for ContextMenuHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            core::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for ContextMenuHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self { object }
    }
}

impl ImplContextMenuHandler for ContextMenuHandlerBuilder {
    #[inline]
    fn get_raw(&self) -> *mut sys::cef_context_menu_handler_t {
        self.object as *mut sys::cef_context_menu_handler_t
    }
}
