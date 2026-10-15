use crate::macros::cef_error;
use crate::prelude::{EXTENSIONS_SWITCH, IntoString};
use crate::render_process::cef_api_handler::CefApiHandler;
use crate::util::v8_accessor::V8DefaultAccessorBuilder;
use crate::util::v8_interceptor::V8DefaultInterceptorBuilder;
use crate::util::{json_to_v8, read_switch_json};
use cef::rc::{Rc, RcImpl};
use cef::{
    Browser, CefString, DictionaryValue, Frame, ImplBrowser, ImplDictionaryValue, ImplFrame,
    ImplListValue, ImplProcessMessage, ImplRenderProcessHandler, ImplV8Context, ImplV8Exception,
    ImplV8Value, ProcessId, ProcessMessage, V8Context, V8Exception, V8Handler, V8Propertyattribute,
    V8Value, WrapRenderProcessHandler, sys, v8_value_create_function, v8_value_create_object,
    v8_value_create_string,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::os::raw::c_int;
use std::sync::{LazyLock, Mutex};

/// Composite key identifying a V8 context: (browser_id, frame_id).
pub(crate) type ContextKey = (i32, String);

pub(crate) static BRP_PROMISES: Mutex<BTreeMap<ContextKey, BTreeMap<String, V8Value>>> =
    Mutex::new(BTreeMap::new());
pub(crate) static LISTEN_EVENTS: Mutex<BTreeMap<ContextKey, BTreeMap<String, V8Value>>> =
    Mutex::new(BTreeMap::new());

static INIT_SCRIPTS: Mutex<BTreeMap<c_int, String>> = Mutex::new(BTreeMap::new());
pub const INIT_SCRIPT_KEY: &str = "init_script";

pub const PROCESS_MESSAGE_BRP: &str = "brp";
pub const PROCESS_MESSAGE_HOST_EMIT: &str = "host-emit";
pub const PROCESS_MESSAGE_JS_EMIT: &str = "js-emit";

/// A BRP reply sent from the browser process to the render process.
///
/// It serializes to the same JSON as `bevy_remote::BrpResult`: `{"Ok": value}` or
/// `{"Err": {"code", "message", "data"}}`.
pub(crate) type BrpReply = Result<Value, BrpReplyError>;

/// The error half of a [`BrpReply`].
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BrpReplyError {
    /// The JSON-RPC error code.
    pub code: i16,
    /// A short, human-readable description of the error.
    pub message: String,
    /// Additional error data, omitted from the JSON when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

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
        let Some(context) = context else {
            return;
        };
        bind_cef_api(context);
        eval_extensions(context);
        if let (Some(browser), Some(frame)) = (browser, frame) {
            inject_initialize_scripts(browser, context, frame);
        }
    }

    fn on_context_released(
        &self,
        browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        _context: Option<&mut V8Context>,
    ) {
        if let (Some(browser), Some(frame)) = (browser, frame) {
            let key = context_key(browser, frame);
            LISTEN_EVENTS.lock().unwrap().remove(&key);
            BRP_PROMISES.lock().unwrap().remove(&key);
        }
    }

    fn on_process_message_received(
        &self,
        browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        _: ProcessId,
        message: Option<&mut ProcessMessage>,
    ) -> c_int {
        if let Some(message) = message
            && let Some(frame) = frame
            && let Some(browser) = browser
            && let Some(ctx) = frame.v8_context()
        {
            match message.name().into_string().as_str() {
                PROCESS_MESSAGE_BRP => {
                    handle_brp_message(message, browser, frame, ctx);
                }
                PROCESS_MESSAGE_HOST_EMIT => {
                    handle_listen_message(message, browser, frame, ctx);
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

/// Defines `window.cef` in `context` with the `brp`, `emit` and `listen` functions.
///
/// On failure, logs with `cef_error!` and leaves `window.cef` undefined.
///
/// Must run while `context` is the entered V8 context, as inside
/// `on_context_created`.
fn bind_cef_api(context: &V8Context) {
    let Some(global) = context.global() else {
        cef_error!("failed to bind window.cef: the context has no global object");
        return;
    };
    let Some(mut cef) = v8_value_create_object(None, None) else {
        cef_error!("failed to bind window.cef: could not create the object");
        return;
    };
    let mut handler = V8Handler::new(CefApiHandler::default());
    for name in ["brp", "emit", "listen"] {
        let key = CefString::from(name);
        let Some(mut function) = v8_value_create_function(Some(&key), Some(&mut handler)) else {
            cef_error!("failed to bind window.cef: could not create cef.{name}");
            return;
        };
        if cef.set_value_bykey(
            Some(&key),
            Some(&mut function),
            V8Propertyattribute::default(),
        ) == 0
        {
            cef_error!("failed to bind window.cef: could not set cef.{name}");
            return;
        }
    }
    if global.set_value_bykey(
        Some(&"cef".into()),
        Some(&mut cef),
        V8Propertyattribute::default(),
    ) == 0
    {
        cef_error!("failed to bind window.cef: could not set it on the global object");
    }
}

/// Evaluates each non-empty `CefExtensions` entry in `context`, in name order.
fn eval_extensions(context: &V8Context) {
    /// The `CefExtensions` that the browser process passes in the extensions switch, iterated
    /// in name order.
    ///
    /// Empty when the switch is absent or holds invalid JSON.
    static EXTENSIONS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
        read_switch_json::<BTreeMap<String, String>>(EXTENSIONS_SWITCH).unwrap_or_default()
    });

    for (name, code) in EXTENSIONS.iter().filter(|(_, code)| !code.is_empty()) {
        eval_script(context, code, &extension_script_url(name));
    }
}

/// Returns the script URL that the extension `name` is evaluated under.
///
/// This URL is the name that console error messages and DevTools use for the extension.
fn extension_script_url(name: &str) -> String {
    format!("cef://localhost/__extensions__/{name}.js")
}

/// Evaluates the browser's `PreloadScripts` in `context` as one script whose URL is the
/// frame's URL.
fn inject_initialize_scripts(browser: &Browser, context: &V8Context, frame: &Frame) {
    let id = browser.identifier();
    let script = INIT_SCRIPTS
        .lock()
        .ok()
        .and_then(|scripts| scripts.get(&id).cloned());
    if let Some(script) = script {
        eval_script(context, &script, &frame.url().into_string());
    }
}

/// Evaluates `code` in `context` under `script_url` and logs an uncaught exception with
/// `cef_error!`.
///
/// The failure is also reported to the context's `console.error` as
/// `<message> (<url>:<line>:<column>)`, with a 1-based column.
///
/// Must run while `context` is the entered V8 context, as inside
/// `on_context_created`.
fn eval_script(context: &V8Context, code: &str, script_url: &str) {
    let mut retval: Option<V8Value> = None;
    let mut exception: Option<V8Exception> = None;
    let result = context.eval(
        Some(&CefString::from(code)),
        Some(&CefString::from(script_url)),
        0,
        Some(&mut retval),
        Some(&mut exception),
    );
    if result != 0 {
        return;
    }
    let message = match exception {
        Some(ex) => format!(
            "{} ({}:{}:{})",
            ex.message().into_string(),
            script_url,
            ex.line_number(),
            ex.start_column() + 1,
        ),
        None => format!("eval failed with no exception details ({script_url})"),
    };
    cef_error!("{message}");
    report_to_console(context, &message);
}

/// Calls `console.error` of `context` with `message`, doing nothing when `console.error` is
/// unavailable.
///
/// Must run while `context` is the entered V8 context, as inside
/// `on_context_created`.
fn report_to_console(context: &V8Context, message: &str) {
    let Some(global) = context.global() else {
        return;
    };
    let Some(mut console) = global.value_bykey(Some(&"console".into())) else {
        return;
    };
    if !console.is_object().is_positive() {
        return;
    }
    let Some(error) = console.value_bykey(Some(&"error".into())) else {
        return;
    };
    if !error.is_function().is_positive() {
        return;
    }
    error.execute_function(
        Some(&mut console),
        Some(&[v8_value_create_string(Some(&message.into()))]),
    );
}

fn context_key(browser: &Browser, frame: &Frame) -> ContextKey {
    (browser.identifier(), frame.identifier().into_string())
}

fn handle_brp_message(
    message: &ProcessMessage,
    browser: &mut Browser,
    frame: &mut Frame,
    ctx: V8Context,
) {
    let Some(argument_list) = message.argument_list() else {
        return;
    };
    let id = argument_list.string(0).into_string();
    let payload = argument_list.string(1).into_string();

    let key = context_key(browser, frame);
    let promise = BRP_PROMISES
        .lock()
        .ok()
        .and_then(|mut promises| promises.get_mut(&key)?.remove(&id));
    let Some(promise) = promise else {
        return;
    };

    if let Ok(brp_result) = serde_json::from_str::<BrpReply>(&payload)
        && ctx.enter() != 0
    {
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

fn handle_listen_message(
    message: &ProcessMessage,
    browser: &mut Browser,
    frame: &mut Frame,
    mut ctx: V8Context,
) {
    let Some(argument_list) = message.argument_list() else {
        return;
    };
    let id = argument_list.string(0).into_string();
    let payload = argument_list.string(1).into_string();

    let key = context_key(browser, frame);
    let callback = LISTEN_EVENTS
        .lock()
        .ok()
        .and_then(|events| events.get(&key)?.get(&id).cloned());
    let Some(callback) = callback else {
        return;
    };

    if ctx.enter() != 0
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&payload)
    {
        let mut obj = v8_value_create_object(
            Some(&mut V8DefaultAccessorBuilder::build()),
            Some(&mut V8DefaultInterceptorBuilder::build()),
        );
        callback.execute_function_with_context(
            Some(&mut ctx),
            obj.as_mut(),
            Some(&[json_to_v8(value)]),
        );
        ctx.exit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that an extension is evaluated under an absolute `cef://localhost` script URL
    /// ending in its name.
    ///
    /// Case: an app registers an extension named `myGame`, and a syntax error in it appears in
    /// the Bevy log with this URL naming the extension.
    #[test]
    fn extension_script_url_names_the_extension() {
        assert_eq!(
            extension_script_url("myGame"),
            "cef://localhost/__extensions__/myGame.js"
        );
    }

    /// Asserts that an error reply in `bevy_remote::BrpResult`'s JSON form decodes into the
    /// host's message.
    ///
    /// Case: a 0.14.0 host answers a page's call to an unknown method, and the render process
    /// must reject the page's promise with that message.
    #[test]
    fn brp_error_reply_decodes_its_message() {
        let reply: BrpReply =
            serde_json::from_str(r#"{"Err":{"code":-32601,"message":"Method `nope` not found"}}"#)
                .expect("an Err reply in bevy_remote's JSON form");
        match reply {
            Err(error) => assert_eq!(error.message, "Method `nope` not found"),
            Ok(value) => panic!("expected an Err reply, got Ok({value})"),
        }
    }

    /// Asserts that a success reply decodes into its JSON value.
    ///
    /// Case: the host's `greet` method answers, and the page's promise resolves with the
    /// greeting.
    #[test]
    fn brp_ok_reply_decodes_its_value() {
        let reply: BrpReply = serde_json::from_str(r#"{"Ok":"Hello, Bevy!"}"#)
            .expect("an Ok reply in bevy_remote's JSON form");
        assert_eq!(reply.ok(), Some(Value::String("Hello, Bevy!".into())));
    }
}
