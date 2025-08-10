use cef::rc::{Rc, RcImpl};
use cef::{ImplV8Accessor, V8Accessor, WrapV8Accessor, sys};
use cef_dll_sys::_cef_v8_accessor_t;

pub struct V8DefaultAccessorBuilder {
    object: *mut RcImpl<_cef_v8_accessor_t, Self>,
}

impl V8DefaultAccessorBuilder {
    pub fn build() -> V8Accessor {
        V8Accessor::new(Self {
            object: core::ptr::null_mut(),
        })
    }
}

impl Rc for V8DefaultAccessorBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for V8DefaultAccessorBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self { object }
    }
}

impl WrapV8Accessor for V8DefaultAccessorBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<_cef_v8_accessor_t, Self>) {
        self.object = object;
    }
}

impl ImplV8Accessor for V8DefaultAccessorBuilder {
    #[inline]
    fn get_raw(&self) -> *mut _cef_v8_accessor_t {
        self.object.cast()
    }
}
