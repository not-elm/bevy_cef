use crate::prelude::PROCESS_MESSAGE_JS_EMIT;
use crate::util::v8_value_to_json;
use cef::rc::{Rc, RcImpl};
use cef::{
    CefString, Frame, ImplFrame, ImplListValue, ImplProcessMessage, ImplV8Handler, ProcessId,
    V8Handler, V8Value, WrapV8Handler, process_message_create, sys,
};
use cef_dll_sys::cef_process_id_t;
use std::os::raw::c_int;

pub struct EmitBuilder {
    object: *mut RcImpl<sys::_cef_v8_handler_t, Self>,
    frame: Frame,
}

impl EmitBuilder {
    pub fn build(frame: Frame) -> V8Handler {
        V8Handler::new(Self {
            object: core::ptr::null_mut(),
            frame,
        })
    }
}

impl Rc for EmitBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapV8Handler for EmitBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_v8_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for EmitBuilder {
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

impl ImplV8Handler for EmitBuilder {
    fn execute(
        &self,
        _: Option<&CefString>,
        _: Option<&mut V8Value>,
        arguments: Option<&[Option<V8Value>]>,
        _: Option<&mut Option<V8Value>>,
        _: Option<&mut CefString>,
    ) -> c_int {
        if let Some(mut process) = process_message_create(Some(&PROCESS_MESSAGE_JS_EMIT.into()))
            && let Some(arguments_list) = process.argument_list()
            && let Some(arguments) = arguments
            && let Some(Some(arg)) = arguments.first()
            && let Some(arg) = v8_value_to_json(arg)
            && let Ok(arg) = serde_json::to_string(&arg)
        {
            arguments_list.set_string(0, Some(&arg.as_str().into()));
            self.frame.send_process_message(
                ProcessId::from(cef_process_id_t::PID_BROWSER),
                Some(&mut process),
            );
        }
        1
    }

    fn get_raw(&self) -> *mut sys::_cef_v8_handler_t {
        self.object.cast()
    }
}
