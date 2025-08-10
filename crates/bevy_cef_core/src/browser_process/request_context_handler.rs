use cef::rc::{Rc, RcImpl};
use cef::{ImplRequestContextHandler, WrapRequestContextHandler, sys};

/// ## Reference
///
/// - [`CefRequestContextHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRequestContextHandler.html)
pub struct RequestContextHandlerBuilder {
    object: *mut RcImpl<sys::cef_request_context_handler_t, Self>,
}

impl RequestContextHandlerBuilder {
    pub fn build() -> cef::RequestContextHandler {
        cef::RequestContextHandler::new(Self {
            object: core::ptr::null_mut(),
        })
    }
}

impl WrapRequestContextHandler for RequestContextHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_request_context_handler_t, Self>) {
        self.object = object;
    }
}

impl Rc for RequestContextHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            core::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for RequestContextHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self { object }
    }
}

impl ImplRequestContextHandler for RequestContextHandlerBuilder {
    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_request_context_handler_t {
        self.object.cast()
    }
}
