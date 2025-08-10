use cef::rc::{Rc, RcImpl};
use cef::{ImplV8Interceptor, V8Interceptor, WrapV8Interceptor, sys};
use cef_dll_sys::_cef_v8_interceptor_t;

pub struct V8DefaultInterceptorBuilder {
    object: *mut RcImpl<_cef_v8_interceptor_t, Self>,
}

impl V8DefaultInterceptorBuilder {
    pub fn build() -> V8Interceptor {
        V8Interceptor::new(Self {
            object: core::ptr::null_mut(),
        })
    }
}

impl WrapV8Interceptor for V8DefaultInterceptorBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<_cef_v8_interceptor_t, Self>) {
        self.object = object;
    }
}

impl Rc for V8DefaultInterceptorBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for V8DefaultInterceptorBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self { object }
    }
}

impl ImplV8Interceptor for V8DefaultInterceptorBuilder {
    #[inline]
    fn get_raw(&self) -> *mut _cef_v8_interceptor_t {
        self.object.cast()
    }
}
