use crate::prelude::{EmitBuilder, IntoString};
use crate::render_process::brp::BrpBuilder;
use crate::render_process::listen::ListenBuilder;
use crate::util::json_to_v8;
use crate::util::v8_accessor::V8DefaultAccessorBuilder;
use crate::util::v8_interceptor::V8DefaultInterceptorBuilder;
use bevy::platform::collections::HashMap;
use bevy_remote::BrpResult;
use cef::rc::{Rc, RcImpl};
use cef::{
    Browser, CefString, DictionaryValue, Frame, ImplBrowser, ImplDictionaryValue, ImplFrame,
    ImplListValue, ImplProcessMessage, ImplRenderProcessHandler, ImplV8Context, ImplV8Value,
    ProcessId, ProcessMessage, V8Context, V8Propertyattribute, V8Value, WrapRenderProcessHandler,
    sys, v8_value_create_function, v8_value_create_object,
};
use std::os::raw::c_int;
use std::sync::Mutex;

pub(crate) static BRP_PROMISES: Mutex<HashMap<String, V8Value>> = Mutex::new(HashMap::new());
pub(crate) static LISTEN_EVENTS: Mutex<HashMap<String, V8Value>> = Mutex::new(HashMap::new());

static INIT_SCRIPTS: Mutex<HashMap<c_int, String>> = Mutex::new(HashMap::new());
pub const INIT_SCRIPT_KEY: &str = "init_script";

pub const PROCESS_MESSAGE_BRP: &str = "brp";
pub const PROCESS_MESSAGE_HOST_EMIT: &str = "host-emit";
pub const PROCESS_MESSAGE_JS_EMIT: &str = "js-emit";

pub struct RenderProcessHandlerBuilder {
    object: *mut RcImpl<sys::_cef_render_process_handler_t, Self>,
}

impl RenderProcessHandlerBuilder {
    pub fn build() -> RenderProcessHandlerBuilder {
        RenderProcessHandlerBuilder {
            object: core::ptr::null_mut(),
        }
    }
}

impl WrapRenderProcessHandler for RenderProcessHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_render_process_handler_t, Self>) {
        self.object = object;
    }
}

impl Rc for RenderProcessHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for RenderProcessHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self { object }
    }
}

impl ImplRenderProcessHandler for RenderProcessHandlerBuilder {
    fn on_browser_created(
        &self,
        browser: Option<&mut Browser>,
        extra: Option<&mut DictionaryValue>,
    ) {
        if let (Some(browser), Some(extra)) = (browser, extra) {
            let script = extra.string(Some(&INIT_SCRIPT_KEY.into())).into_string();
            if script.is_empty() {
                return;
            }
            let id = browser.identifier();
            INIT_SCRIPTS.lock().unwrap().insert(id, script);
        }
    }

    fn on_context_created(
        &self,
        browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        context: Option<&mut V8Context>,
    ) {
        if let Some(context) = context
            && let Some(frame) = frame
            && let Some(browser) = browser
        {
            inject_initialize_scripts(browser, context, frame);
            inject_cef_api(context, frame);
        }
    }

    fn on_process_message_received(
        &self,
        _browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        _: ProcessId,
        message: Option<&mut ProcessMessage>,
    ) -> c_int {
        if let Some(message) = message
            && let Some(frame) = frame
            && let Some(ctx) = frame.v8_context()
        {
            match message.name().into_string().as_str() {
                PROCESS_MESSAGE_BRP => {
                    handle_brp_message(message, ctx);
                }
                PROCESS_MESSAGE_HOST_EMIT => {
                    handle_listen_message(message, ctx);
                }
                _ => {}
            }
        };
        1
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_render_process_handler_t {
        self.object.cast()
    }
}

fn inject_initialize_scripts(browser: &mut Browser, context: &mut V8Context, frame: &mut Frame) {
    let id = browser.identifier();
    if let Some(script) = INIT_SCRIPTS.lock().ok().and_then(|scripts| {
        let script = scripts.get(&id)?;
        Some(CefString::from(script.as_str()))
    }) {
        context.enter();
        frame.execute_java_script(Some(&script), Some(&(&frame.url()).into()), 0);
        context.exit();
    }
}

fn inject_cef_api(context: &mut V8Context, frame: &mut Frame) {
    if let Some(g) = context.global()
        && let Some(mut cef) = v8_value_create_object(
            Some(&mut V8DefaultAccessorBuilder::build()),
            Some(&mut V8DefaultInterceptorBuilder::build()),
        )
        && let Some(mut brp) = v8_value_create_function(
            Some(&"brp".into()),
            Some(&mut BrpBuilder::build(frame.clone())),
        )
        && let Some(mut emit) = v8_value_create_function(
            Some(&"emit".into()),
            Some(&mut EmitBuilder::build(frame.clone())),
        )
        && let Some(mut listen) = v8_value_create_function(
            Some(&"listen".into()),
            Some(&mut ListenBuilder::build(frame.clone())),
        )
    {
        cef.set_value_bykey(
            Some(&"brp".into()),
            Some(&mut brp),
            V8Propertyattribute::default(),
        );
        cef.set_value_bykey(
            Some(&"emit".into()),
            Some(&mut emit),
            V8Propertyattribute::default(),
        );
        cef.set_value_bykey(
            Some(&"listen".into()),
            Some(&mut listen),
            V8Propertyattribute::default(),
        );
        g.set_value_bykey(
            Some(&"cef".into()),
            Some(&mut cef),
            V8Propertyattribute::default(),
        );
    }
}

fn handle_brp_message(message: &ProcessMessage, ctx: V8Context) {
    let Some(argument_list) = message.argument_list() else {
        return;
    };
    let id = argument_list.string(0).into_string();
    let payload = argument_list.string(1).into_string();
    let Ok(Some(promise)) = BRP_PROMISES.lock().map(|mut p| p.remove(&id)) else {
        return;
    };

    if let Ok(brp_result) = serde_json::from_str::<BrpResult>(&payload) {
        ctx.enter();
        match brp_result {
            Ok(v) => {
                promise.resolve_promise(json_to_v8(v).as_mut());
            }
            Err(e) => {
                promise.reject_promise(Some(&e.message.as_str().into()));
            }
        }
        ctx.exit();
    }
}

fn handle_listen_message(message: &ProcessMessage, mut ctx: V8Context) {
    let Some(argument_list) = message.argument_list() else {
        return;
    };
    let id = argument_list.string(0).into_string();
    let payload = argument_list.string(1).into_string();

    ctx.enter();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&payload)
        && let Ok(events) = LISTEN_EVENTS.lock()
    {
        let mut obj = v8_value_create_object(
            Some(&mut V8DefaultAccessorBuilder::build()),
            Some(&mut V8DefaultInterceptorBuilder::build()),
        );
        let Some(callback) = events.get(&id) else {
            return;
        };
        callback.execute_function_with_context(
            Some(&mut ctx),
            obj.as_mut(),
            Some(&[json_to_v8(value)]),
        );
    }
    ctx.exit();
}
