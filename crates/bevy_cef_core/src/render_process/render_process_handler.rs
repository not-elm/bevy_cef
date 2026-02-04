use crate::prelude::IntoString;
use crate::render_process::cef_api_handler::CefApiHandler;
use crate::util::json_to_v8;
use crate::util::v8_accessor::V8DefaultAccessorBuilder;
use crate::util::v8_interceptor::V8DefaultInterceptorBuilder;
use bevy::platform::collections::HashMap;
use bevy_remote::BrpResult;
use cef::rc::{Rc, RcImpl};
use cef::{
    command_line_get_global, register_extension, Browser, CefString, DictionaryValue, Frame,
    ImplBrowser, ImplCommandLine, ImplDictionaryValue, ImplFrame, ImplListValue,
    ImplProcessMessage, ImplRenderProcessHandler, ImplV8Context, ImplV8Value, ProcessId,
    ProcessMessage, V8Context, V8Handler, V8Value, WrapRenderProcessHandler, sys,
    v8_value_create_object,
};
use std::collections::HashMap as StdHashMap;
use std::os::raw::c_int;
use std::sync::Mutex;

const EXTENSIONS_SWITCH: &str = "bevy-cef-extensions";

const CEF_API_EXTENSION_NAME: &str = "v8/bevy-cef-api";
const CEF_API_EXTENSION_CODE: &str = r#"
var cef;
if (!cef) cef = {};
(function() {
  native function __cef_brp();
  native function __cef_emit();
  native function __cef_listen();
  cef.brp = __cef_brp;
  cef.emit = __cef_emit;
  cef.listen = __cef_listen;
})();
"#;

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
    fn on_web_kit_initialized(&self) {
        register_cef_api_extension();
        register_extensions_from_command_line();
    }

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
        context.eval(Some(&script), Some(&(&frame.url()).into()), 0, None, None);
        context.exit();
    }
}

fn register_cef_api_extension() {
    register_extension(
        Some(&CEF_API_EXTENSION_NAME.into()),
        Some(&CEF_API_EXTENSION_CODE.into()),
        Some(&mut V8Handler::new(CefApiHandler::new())),
    );
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

fn register_extensions_from_command_line() {
    let Some(cmd_line) = command_line_get_global() else {
        return;
    };
    if cmd_line.has_switch(Some(&EXTENSIONS_SWITCH.into())) == 0 {
        return;
    }
    let json = cmd_line
        .switch_value(Some(&EXTENSIONS_SWITCH.into()))
        .into_string();
    if json.is_empty() {
        return;
    }

    let Ok(extensions) = serde_json::from_str::<StdHashMap<String, String>>(&json) else {
        eprintln!("bevy_cef: failed to parse extensions JSON: {}", json);
        return;
    };

    for (name, code) in extensions {
        let full_name = format!("v8/{}", name);
        register_extension(
            Some(&full_name.as_str().into()),
            Some(&code.as_str().into()),
            None,
        );
    }
}
