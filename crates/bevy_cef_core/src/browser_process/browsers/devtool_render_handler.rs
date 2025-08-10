use cef::rc::{Rc, RcImpl};
use cef::{Browser, ImplRenderHandler, Rect, RenderHandler, WrapRenderHandler, sys};

/// ## Reference
///
/// - [`CefRenderHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRenderHandler.html)
pub struct DevToolRenderHandlerBuilder {
    object: *mut RcImpl<sys::cef_render_handler_t, Self>,
}

impl DevToolRenderHandlerBuilder {
    pub fn build() -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
        })
    }
}

impl Rc for DevToolRenderHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapRenderHandler for DevToolRenderHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_render_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for DevToolRenderHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self { object }
    }
}

impl ImplRenderHandler for DevToolRenderHandlerBuilder {
    fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
        if let Some(rect) = rect {
            rect.width = 800;
            rect.height = 800;
        }
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_render_handler_t {
        self.object.cast()
    }
}
