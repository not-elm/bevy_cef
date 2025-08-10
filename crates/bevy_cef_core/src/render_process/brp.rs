use crate::prelude::{BRP_PROMISES, PROCESS_MESSAGE_BRP, v8_value_to_json};
use cef::rc::{ConvertParam, ConvertReturnValue, Rc, RcImpl};
use cef::{
    CefString, Frame, ImplFrame, ImplListValue, ImplProcessMessage, ImplV8Handler, ImplV8Value,
    ProcessId, V8Handler, V8Value, WrapV8Handler, process_message_create, sys,
    v8_value_create_promise, v8_value_create_string,
};
use cef_dll_sys::{_cef_v8_handler_t, _cef_v8_value_t, cef_process_id_t, cef_string_t};
use std::os::raw::c_int;
use std::ptr::write;

/// Implements the `window.brp` function in JavaScript.
///
/// The function definition is `async <T>(request: BrpRequest) -> Promise<T>`.
///
/// The flow from the execution of the function to the return of the result to the Javascript side is as follows:
/// 1. Send `BrpRequest` to the browser process.
/// 2. The browser process receives `BrpResult` and sends it to the render process.
/// 3. The render process receives the result in `on_process_message_received`.
/// 4. The render process resolves the result to the `Promise` of `window.brp`.
pub struct BrpBuilder {
    object: *mut RcImpl<_cef_v8_handler_t, Self>,
    frame: Frame,
}

impl BrpBuilder {
    pub fn build(frame: Frame) -> V8Handler {
        V8Handler::new(Self {
            object: core::ptr::null_mut(),
            frame,
        })
    }
}

impl Rc for BrpBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapV8Handler for BrpBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<_cef_v8_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for BrpBuilder {
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

impl ImplV8Handler for BrpBuilder {
    fn execute(
        &self,
        _: Option<&CefString>,
        _: Option<&mut V8Value>,
        arguments: Option<&[Option<V8Value>]>,
        ret: Option<&mut Option<V8Value>>,
        _: Option<&mut CefString>,
    ) -> c_int {
        if let Some(mut process) = process_message_create(Some(&PROCESS_MESSAGE_BRP.into()))
            && let Some(promise) = v8_value_create_promise()
        {
            if let Some(arguments_list) = process.argument_list()
                && let Some(arguments) = arguments
                && let Some(Some(arg)) = arguments.first()
                && let Some(brp_request) = v8_value_to_json(arg)
                && let Ok(brp_request) = serde_json::to_string(&brp_request)
                && let Some(ret) = ret
            {
                let id = uuid::Uuid::new_v4().to_string();
                arguments_list.set_string(0, Some(&id.as_str().into()));
                arguments_list.set_string(1, Some(&brp_request.as_str().into()));
                self.frame.send_process_message(
                    ProcessId::from(cef_process_id_t::PID_BROWSER),
                    Some(&mut process),
                );
                ret.replace(promise.clone());
                let mut promises = BRP_PROMISES.lock().unwrap();
                promises.insert(id, promise);
            } else {
                let mut exception =
                    v8_value_create_string(Some(&"Failed to execute BRP request".into()));
                promise.resolve_promise(exception.as_mut());
            }
        }
        1
    }

    fn init_methods(object: &mut _cef_v8_handler_t) {
        init_methods::<Self>(object);
    }

    fn get_raw(&self) -> *mut _cef_v8_handler_t {
        self.object.cast()
    }
}

fn init_methods<I: ImplV8Handler>(object: &mut _cef_v8_handler_t) {
    object.execute = Some(execute::<I>);
}

extern "C" fn execute<I: ImplV8Handler>(
    self_: *mut _cef_v8_handler_t,
    name: *const cef_string_t,
    object: *mut _cef_v8_value_t,
    arguments_count: usize,
    arguments: *const *mut _cef_v8_value_t,
    retval: *mut *mut _cef_v8_value_t,
    exception: *mut cef_string_t,
) -> c_int {
    let (arg_self_, arg_name, arg_object, arg_arguments_count, arg_arguments, _, arg_exception) = (
        self_,
        name,
        object,
        arguments_count,
        arguments,
        retval,
        exception,
    );
    let arg_self_: &RcImpl<_, I> = RcImpl::get(arg_self_);
    let arg_name = if arg_name.is_null() {
        None
    } else {
        Some(arg_name.into())
    };
    let arg_name = arg_name.as_ref();
    let mut arg_object =
        unsafe { arg_object.as_mut() }.map(|arg| (arg as *mut _cef_v8_value_t).wrap_result());
    let arg_object = arg_object.as_mut();
    let vec_arguments = unsafe { arg_arguments.as_ref() }.map(|arg| {
        let arg =
            unsafe { std::slice::from_raw_parts(std::ptr::from_ref(arg), arg_arguments_count) };
        arg.iter()
            .map(|arg| {
                if arg.is_null() {
                    None
                } else {
                    Some((*arg).wrap_result())
                }
            })
            .collect::<Vec<_>>()
    });
    let arg_arguments = vec_arguments.as_deref();
    let mut arg_retval: Option<V8Value> = None;
    let arg = Some(&mut arg_retval);
    let mut arg_exception = if arg_exception.is_null() {
        None
    } else {
        Some(arg_exception.into())
    };
    let arg_exception = arg_exception.as_mut();
    let r = ImplV8Handler::execute(
        &arg_self_.interface,
        arg_name,
        arg_object,
        arg_arguments,
        arg,
        arg_exception,
    );
    if let Some(ret) = arg_retval {
        // When the result is received, the pointer should be updated here
        // and the exception should also be updated.
        unsafe {
            write(retval, ret.into_raw());
        }
    }
    r
}
