// use std::os::raw::c_int;
// use bevy::ecs::reflect::ReflectCommandExt;
// use cef::rc::{Rc, RcImpl};
// use cef::{sys, CefString, ImplV8Handler, V8Handler, V8Value, WrapV8Handler};
// use cef_dll_sys::{_cef_v8_handler_t, _cef_v8_value_t, cef_string_t};
//
// pub struct V8HandlerWrapBuilder<T> {
//     base: T,
// }
//
// impl<T> V8HandlerWrapBuilder<T> {
//     pub fn build(base: T) -> V8Handler {
//         V8Handler::new(Self {
//             base,
//         })
//     }
// }
//
// impl<T: Rc> Rc for V8HandlerWrapBuilder<T> {
//     fn as_base(&self) -> &sys::cef_base_ref_counted_t {
//         self.base.as_base()
//     }
// }
//
// impl<T: ImplV8Handler> ImplV8Handler for V8HandlerWrapBuilder<T> {
//     fn execute(&self, name: Option<&CefString>, object: Option<&mut V8Value>, arguments: Option<&[Option<V8Value>]>, retval: Option<&mut Option<V8Value>>, exception: Option<&mut CefString>) -> c_int {
//         self.base.execute(
//             name,
//             object,
//             arguments,
//             retval,
//             exception,
//         )
//     }
//
//     fn get_raw(&self) -> *mut _cef_v8_handler_t {
//         self.base.get_raw()
//     }
//
//     fn init_methods(object: &mut _cef_v8_handler_t) {
//         T::init_methods(object);
//
//     }
// }
//
// impl<T: Clone> Clone for V8HandlerWrapBuilder<T> {
//     fn clone(&self) -> Self {
//         Self{
//             base: self.base.clone(),
//         }
//     }
// }
//
// impl <T: WrapV8Handler> cef::WrapV8Handler for V8HandlerWrapBuilder<T> {
//     fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_v8_handler_t, Self>) {
//         self.base.wrap_rc(object);
//     }
// }
//
// // extern "C" fn execute<I: ImplV8Handler>(
// //     self_: *mut _cef_v8_handler_t,
// //     name: *const cef_string_t,
// //     object: *mut _cef_v8_value_t,
// //     arguments_count: usize,
// //     arguments: *const *mut _cef_v8_value_t,
// //     retval: *mut *mut _cef_v8_value_t,
// //     exception: *mut cef_string_t,
// // ) -> ::std::os::raw::c_int {
// //
// // }
