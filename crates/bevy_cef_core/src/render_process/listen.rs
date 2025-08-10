use crate::prelude::LISTEN_EVENTS;
use crate::util::IntoString;
use cef::rc::{Rc, RcImpl};
use cef::{CefString, Frame, ImplV8Handler, ImplV8Value, V8Handler, V8Value, WrapV8Handler, sys};
use std::os::raw::c_int;

pub struct ListenBuilder {
    object: *mut RcImpl<sys::_cef_v8_handler_t, Self>,
    frame: Frame,
}

impl ListenBuilder {
    pub fn build(frame: Frame) -> V8Handler {
        V8Handler::new(Self {
            object: core::ptr::null_mut(),
            frame,
        })
    }
}

impl Rc for ListenBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapV8Handler for ListenBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_v8_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for ListenBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self {
            object,
            frame: self.frame.clone(),
        }
    }
}

impl ImplV8Handler for ListenBuilder {
    fn execute(
        &self,
        _: Option<&CefString>,
        _: Option<&mut V8Value>,
        arguments: Option<&[Option<V8Value>]>,
        _: Option<&mut Option<V8Value>>,
        _: Option<&mut CefString>,
    ) -> c_int {
        if let Some(arguments) = arguments
            && let Some(Some(id)) = arguments.first()
            && 0 < id.is_string()
            && let Some(Some(callback)) = arguments.get(1)
            && 0 < callback.is_function()
        {
            LISTEN_EVENTS
                .lock()
                .unwrap()
                .insert(id.string_value().into_string(), callback.clone());
        }
        1
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_v8_handler_t {
        self.object.cast()
    }
}
