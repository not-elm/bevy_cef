//! Caller-registered custom scheme support: public types, a process-global
//! registry, and a generic CEF factory/resource-handler adapter.

use cef_dll_sys::cef_scheme_options_t::{
    CEF_SCHEME_OPTION_CORS_ENABLED, CEF_SCHEME_OPTION_CSP_BYPASSING,
    CEF_SCHEME_OPTION_DISPLAY_ISOLATED, CEF_SCHEME_OPTION_FETCH_ENABLED,
    CEF_SCHEME_OPTION_LOCAL, CEF_SCHEME_OPTION_SECURE, CEF_SCHEME_OPTION_STANDARD,
};
use cef::rc::Rc;
use cef::{
    Callback, CefString, ImplCommandLine, ImplRequest, ImplResponse, ImplResourceHandler,
    ImplSchemeHandlerFactory, Request, ResourceHandler, ResourceReadCallback, Response,
    SchemeHandlerFactory, WrapResourceHandler, WrapSchemeHandlerFactory, command_line_get_global,
    wrap_resource_handler, wrap_scheme_handler_factory,
};
use crate::util::{CUSTOM_SCHEMES_SWITCH, IntoString};
use serde::{Deserialize, Serialize};
use std::io::{self, Cursor, Read};
use std::sync::{Arc, Mutex, OnceLock};

/// Option flags for a custom scheme. A thin serializable wrapper over a raw
/// `u32` mask so the declaration can cross to the render subprocess.
///
/// The constants enumerate the full `cef_scheme_options_t` set (including
/// `CSP_BYPASSING`), sourced the same way as `util::cef_scheme_flags`, so the
/// list cannot silently drift. `bitflags` is intentionally not pulled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CefSchemeOptions(pub u32);

impl CefSchemeOptions {
    pub const STANDARD: Self = Self(CEF_SCHEME_OPTION_STANDARD as u32);
    pub const SECURE: Self = Self(CEF_SCHEME_OPTION_SECURE as u32);
    pub const LOCAL: Self = Self(CEF_SCHEME_OPTION_LOCAL as u32);
    pub const DISPLAY_ISOLATED: Self = Self(CEF_SCHEME_OPTION_DISPLAY_ISOLATED as u32);
    pub const CORS_ENABLED: Self = Self(CEF_SCHEME_OPTION_CORS_ENABLED as u32);
    pub const CSP_BYPASSING: Self = Self(CEF_SCHEME_OPTION_CSP_BYPASSING as u32);
    pub const FETCH_ENABLED: Self = Self(CEF_SCHEME_OPTION_FETCH_ENABLED as u32);
}

impl std::ops::BitOr for CefSchemeOptions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Response body. `Reader` streams large bodies without buffering; `Bytes`
/// covers the small in-memory case; `Empty` is a zero-length body.
pub enum CefSchemeBody {
    Empty,
    Bytes(Vec<u8>),
    Reader {
        reader: Box<dyn Read + Send>,
        len: Option<u64>,
    },
}

/// A handler's reply for one request.
pub struct CefSchemeResponse {
    pub status: u16,
    pub mime_type: String,
    pub headers: Vec<(String, String)>,
    pub body: CefSchemeBody,
}

impl CefSchemeResponse {
    /// A 404 `text/plain` response with a short body.
    pub fn not_found() -> Self {
        Self {
            status: 404,
            mime_type: "text/plain".to_string(),
            headers: Vec::new(),
            body: CefSchemeBody::Bytes(b"404 Not Found".to_vec()),
        }
    }

    /// A 200 response carrying in-memory bytes with the given MIME type.
    pub fn bytes(mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            status: 200,
            mime_type: mime_type.into(),
            headers: Vec::new(),
            body: CefSchemeBody::Bytes(data),
        }
    }
}

/// Collapses any body variant into one `Read` source plus an optional known
/// length, so `read()` has a single draining path.
fn body_to_reader(body: CefSchemeBody) -> (Box<dyn Read + Send>, Option<u64>) {
    match body {
        CefSchemeBody::Empty => (Box::new(io::empty()), Some(0)),
        CefSchemeBody::Bytes(data) => {
            let len = data.len() as u64;
            (Box::new(Cursor::new(data)), Some(len))
        }
        CefSchemeBody::Reader { reader, len } => (reader, len),
    }
}

/// The materialized response for one in-flight request: headers metadata plus a
/// single draining reader. Stored across the CEF `open → headers → read`
/// callback sequence.
struct ResponseState {
    status: u16,
    mime_type: String,
    headers: Vec<(String, String)>,
    reader: Box<dyn Read + Send>,
    len: Option<u64>,
}

/// Calls the handler with the request, isolating panics: a panic that unwinds
/// across the CEF FFI boundary is UB, so a caught panic becomes a 500. (Only
/// effective under `panic = "unwind"`; under `panic = "abort"` the process
/// aborts before this runs.)
fn invoke_handler(handler: &Arc<dyn CefSchemeHandler>, request: &CefSchemeRequest) -> ResponseState {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler.handle(request)));
    let response = outcome.unwrap_or_else(|_| {
        eprintln!("bevy_cef: custom scheme handler panicked; returning 500");
        CefSchemeResponse {
            status: 500,
            mime_type: "text/plain".to_string(),
            headers: Vec::new(),
            body: CefSchemeBody::Bytes(b"500 Internal Server Error".to_vec()),
        }
    });
    let (reader, len) = body_to_reader(response.body);
    ResponseState {
        status: response.status,
        mime_type: response.mime_type,
        headers: response.headers,
        reader,
        len,
    }
}

/// The request handed to a [`CefSchemeHandler`]. v1 carries only the URL;
/// method/headers can be added later without breaking consumers (they only
/// read `&CefSchemeRequest`).
pub struct CefSchemeRequest {
    pub url: String,
}

/// Caller-implemented request servicing for a custom scheme.
///
/// `handle` runs on a CEF resource-handler worker thread — not the Bevy thread,
/// and not CEF's IO or UI thread. It may run concurrently across requests, so
/// implementors share state via `Arc` / `Arc<RwLock<…>>` rather than touching
/// the Bevy `World`.
pub trait CefSchemeHandler: Send + Sync + 'static {
    fn handle(&self, request: &CefSchemeRequest) -> CefSchemeResponse;
}

/// One complete custom-scheme registration: declaration (build-time) + handler
/// (runtime). Pass these to `CefPlugin { custom_schemes, .. }`.
#[derive(Clone)]
pub struct CefCustomScheme {
    /// Scheme name without `://`, e.g. `"demo"`.
    pub name: String,
    pub options: CefSchemeOptions,
    /// `None` matches all hosts (standard schemes); `Some(host)` restricts to
    /// that host.
    pub domain: Option<String>,
    pub handler: Arc<dyn CefSchemeHandler>,
}

/// Serializable declaration half — the only part that crosses to the render
/// subprocess (the handler `Arc<dyn>` cannot be serialized). Private DTO.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct CefSchemeDecl {
    name: String,
    options: CefSchemeOptions,
}

impl From<&CefCustomScheme> for CefSchemeDecl {
    fn from(scheme: &CefCustomScheme) -> Self {
        Self {
            name: scheme.name.clone(),
            options: scheme.options,
        }
    }
}

/// Parses the switch JSON into declarations, logging and yielding an empty
/// vec on malformed input.
// consumed by the CEF adapter + lifecycle wiring in later commits
#[allow(dead_code)]
fn parse_decls_json(json: &str) -> Vec<CefSchemeDecl> {
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("bevy_cef: failed to parse custom-scheme switch JSON: {}", e);
        Vec::new()
    })
}

/// Process-global set-once registry of custom schemes. Populated in the browser
/// process by `CefPlugin::build` before CEF init; empty in the render
/// subprocess (which reads declarations from the command line instead).
static REGISTERED: OnceLock<Vec<CefCustomScheme>> = OnceLock::new();

/// Installs the custom schemes for this process. Idempotent: a second call is
/// ignored. De-duplicates by name (first wins). Call before CEF initialization.
pub fn init_registered_schemes(schemes: Vec<CefCustomScheme>) {
    let _ = REGISTERED.set(dedup_by_name(schemes));
}

/// The custom schemes registered in this process (empty if none / not yet set).
// consumed by the CEF adapter + lifecycle wiring in later commits
#[allow(dead_code)]
pub(crate) fn registered_schemes() -> &'static [CefCustomScheme] {
    REGISTERED.get().map(Vec::as_slice).unwrap_or(&[])
}

fn dedup_by_name(schemes: Vec<CefCustomScheme>) -> Vec<CefCustomScheme> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for scheme in schemes {
        if seen.insert(scheme.name.clone()) {
            out.push(scheme);
        } else {
            eprintln!("bevy_cef: duplicate custom scheme name ignored: {}", scheme.name);
        }
    }
    out
}

/// Serializes declarations (name + flags) to JSON for the child-process switch.
/// `None` when there are no schemes (so no switch is appended).
// consumed by the CEF adapter + lifecycle wiring in later commits
#[allow(dead_code)]
fn decls_json_for(schemes: &[CefCustomScheme]) -> Option<String> {
    if schemes.is_empty() {
        return None;
    }
    let decls: Vec<CefSchemeDecl> = schemes.iter().map(CefSchemeDecl::from).collect();
    match serde_json::to_string(&decls) {
        Ok(json) => Some(json),
        Err(e) => {
            eprintln!("bevy_cef: failed to serialize custom-scheme declarations: {e}");
            None
        }
    }
}

/// JSON of the schemes registered in this (browser) process, for switch
/// injection. `None` if none are registered.
// consumed by the CEF adapter + lifecycle wiring in later commits
#[allow(dead_code)]
pub(crate) fn current_scheme_decls_json() -> Option<String> {
    decls_json_for(registered_schemes())
}

/// Reads custom-scheme declarations from this process's command line (set by the
/// parent via [`CUSTOM_SCHEMES_SWITCH`]). Used by the render process, which has
/// no access to the browser-process registry.
// consumed by the CEF adapter + lifecycle wiring in later commits
#[allow(dead_code)]
pub(crate) fn decls_from_command_line() -> Vec<CefSchemeDecl> {
    let Some(cmd) = command_line_get_global() else {
        return Vec::new();
    };
    if cmd.has_switch(Some(&CUSTOM_SCHEMES_SWITCH.into())) == 0 {
        return Vec::new();
    }
    let json = cmd
        .switch_value(Some(&CUSTOM_SCHEMES_SWITCH.into()))
        .into_string();
    if json.is_empty() {
        return Vec::new();
    }
    parse_decls_json(&json)
}

/// Builds the opaque `SchemeHandlerFactory` for a caller's handler, ready to
/// hand to `RequestContext::register_scheme_handler_factory`.
// consumed by the per-RequestContext registration wiring in a later commit
#[allow(dead_code)]
pub(crate) fn make_factory(handler: Arc<dyn CefSchemeHandler>) -> SchemeHandlerFactory {
    GenericSchemeHandlerFactory::new(handler)
}

wrap_scheme_handler_factory! {
    struct GenericSchemeHandlerFactory {
        handler: Arc<dyn CefSchemeHandler>,
    }

    impl SchemeHandlerFactory {
        fn create(
            &self,
            _browser: Option<&mut cef::Browser>,
            _frame: Option<&mut cef::Frame>,
            _scheme_name: Option<&CefString>,
            _request: Option<&mut Request>,
        ) -> Option<ResourceHandler> {
            Some(GenericResourceHandler::new(
                self.handler.clone(),
                Arc::new(Mutex::new(None)),
            ))
        }
    }
}

wrap_resource_handler! {
    struct GenericResourceHandler {
        handler: Arc<dyn CefSchemeHandler>,
        state: Arc<Mutex<Option<ResponseState>>>,
    }

    impl ResourceHandler {
        fn open(
            &self,
            request: Option<&mut Request>,
            handle_request: Option<&mut i32>,
            _callback: Option<&mut Callback>,
        ) -> i32 {
            let Some(request) = request else {
                return 0;
            };
            let scheme_request = CefSchemeRequest {
                url: request.url().into_string(),
            };
            let response = invoke_handler(&self.handler, &scheme_request);
            if let Ok(mut guard) = self.state.lock() {
                *guard = Some(response);
            }
            if let Some(handle_request) = handle_request {
                *handle_request = 1;
            }
            1
        }

        fn response_headers(
            &self,
            response: Option<&mut Response>,
            response_length: Option<&mut i64>,
            _redirect_url: Option<&mut CefString>,
        ) {
            let Some(response) = response else {
                return;
            };
            let Ok(guard) = self.state.lock() else {
                response.set_status(500);
                if let Some(out) = response_length {
                    *out = 0;
                }
                return;
            };
            let Some(state) = guard.as_ref() else {
                response.set_status(500);
                if let Some(out) = response_length {
                    *out = 0;
                }
                return;
            };
            response.set_status(state.status as i32);
            response.set_mime_type(Some(&CefString::from(state.mime_type.as_str())));
            for (name, value) in &state.headers {
                response.set_header_by_name(
                    Some(&CefString::from(name.as_str())),
                    Some(&CefString::from(value.as_str())),
                    1,
                );
            }
            if let Some(out) = response_length {
                *out = state.len.map(|l| l as i64).unwrap_or(-1);
            }
        }

        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        fn read(
            &self,
            data_out: *mut u8,
            bytes_to_read: i32,
            bytes_read: Option<&mut i32>,
            _callback: Option<&mut ResourceReadCallback>,
        ) -> i32 {
            let Some(bytes_read) = bytes_read else {
                return 0;
            };
            if bytes_to_read <= 0 {
                *bytes_read = 0;
                return 0;
            }
            let Ok(mut guard) = self.state.lock() else {
                *bytes_read = 0;
                return 0;
            };
            let Some(state) = guard.as_mut() else {
                *bytes_read = 0;
                return 0;
            };
            // SAFETY: CEF guarantees that when `bytes_to_read > 0` the `data_out` pointer is
            // non-null, valid for writes of exactly `bytes_to_read` bytes, and exclusively
            // owned by this call for its duration. The temporary `&mut [u8]` does not
            // outlive the call.
            let buf = unsafe { std::slice::from_raw_parts_mut(data_out, bytes_to_read as usize) };
            match state.reader.read(buf) {
                Ok(0) => {
                    *bytes_read = 0;
                    0
                }
                Ok(n) => {
                    *bytes_read = n as i32;
                    1
                }
                Err(e) => {
                    eprintln!("bevy_cef: custom scheme read failed: {e}");
                    *bytes_read = 0;
                    0
                }
            }
        }

        fn cancel(&self) {
            if let Ok(mut guard) = self.state.lock() {
                *guard = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn body_to_reader_bytes_reports_len_and_content() {
        let (mut reader, len) = body_to_reader(CefSchemeBody::Bytes(b"hello".to_vec()));
        assert_eq!(len, Some(5));
        let mut s = String::new();
        reader.read_to_string(&mut s).unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn body_to_reader_empty_is_zero_len_eof() {
        let (mut reader, len) = body_to_reader(CefSchemeBody::Empty);
        assert_eq!(len, Some(0));
        let mut buf = [0u8; 4];
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn not_found_is_404() {
        let r = CefSchemeResponse::not_found();
        assert_eq!(r.status, 404);
        assert_eq!(r.mime_type, "text/plain");
    }

    #[test]
    fn bytes_constructor_is_200_with_mime() {
        let r = CefSchemeResponse::bytes("text/css", b"body{}".to_vec());
        assert_eq!(r.status, 200);
        assert_eq!(r.mime_type, "text/css");
        let (mut reader, _len) = body_to_reader(r.body);
        let mut body = Vec::new();
        reader.read_to_end(&mut body).unwrap();
        assert_eq!(body, b"body{}");
    }

    struct Dummy;
    impl CefSchemeHandler for Dummy {
        fn handle(&self, _request: &CefSchemeRequest) -> CefSchemeResponse {
            CefSchemeResponse::not_found()
        }
    }

    #[test]
    fn decl_projects_name_and_options() {
        let scheme = CefCustomScheme {
            name: "demo".to_string(),
            options: CefSchemeOptions::STANDARD,
            domain: None,
            handler: std::sync::Arc::new(Dummy),
        };
        let decl = CefSchemeDecl::from(&scheme);
        assert_eq!(decl.name, "demo");
        assert_eq!(decl.options, CefSchemeOptions::STANDARD);
    }

    #[test]
    fn decl_json_round_trip() {
        let decls = vec![CefSchemeDecl {
            name: "demo".to_string(),
            options: CefSchemeOptions::STANDARD | CefSchemeOptions::SECURE,
        }];
        let json = serde_json::to_string(&decls).unwrap();
        assert_eq!(parse_decls_json(&json), decls);
    }

    #[test]
    fn parse_decls_json_bad_input_is_empty() {
        assert!(parse_decls_json("not json").is_empty());
    }

    #[test]
    fn dedup_keeps_first_occurrence_by_name() {
        let first = CefCustomScheme {
            name: "x".to_string(),
            options: CefSchemeOptions::STANDARD,
            domain: None,
            handler: std::sync::Arc::new(Dummy),
        };
        let second = CefCustomScheme {
            name: "x".to_string(),
            options: CefSchemeOptions::SECURE,
            domain: Some("host".to_string()),
            handler: std::sync::Arc::new(Dummy),
        };
        let out = dedup_by_name(vec![first, second]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].options, CefSchemeOptions::STANDARD);
    }

    #[test]
    fn decls_to_json_then_parse_round_trips() {
        let schemes = vec![CefCustomScheme {
            name: "demo".to_string(),
            options: CefSchemeOptions::STANDARD,
            domain: None,
            handler: std::sync::Arc::new(Dummy),
        }];
        let json = decls_json_for(&schemes).expect("non-empty");
        let parsed = parse_decls_json(&json);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "demo");
    }

    #[test]
    fn decls_json_for_empty_is_none() {
        assert!(decls_json_for(&[]).is_none());
    }

    struct BytesHandler;
    impl CefSchemeHandler for BytesHandler {
        fn handle(&self, _request: &CefSchemeRequest) -> CefSchemeResponse {
            CefSchemeResponse::bytes("text/plain", b"ok".to_vec())
        }
    }

    struct PanicHandler;
    impl CefSchemeHandler for PanicHandler {
        fn handle(&self, _request: &CefSchemeRequest) -> CefSchemeResponse {
            panic!("handler boom")
        }
    }

    #[test]
    fn invoke_handler_streams_bytes_with_status_and_len() {
        let handler: std::sync::Arc<dyn CefSchemeHandler> = std::sync::Arc::new(BytesHandler);
        let mut state = invoke_handler(&handler, &CefSchemeRequest { url: "demo://x/".into() });
        assert_eq!(state.status, 200);
        assert_eq!(state.len, Some(2));
        let mut s = String::new();
        state.reader.read_to_string(&mut s).unwrap();
        assert_eq!(s, "ok");
    }

    #[test]
    fn invoke_handler_converts_panic_to_500() {
        // The default panic hook still prints a backtrace to stderr; that is
        // expected and harmless for this test.
        let handler: std::sync::Arc<dyn CefSchemeHandler> = std::sync::Arc::new(PanicHandler);
        let state = invoke_handler(&handler, &CefSchemeRequest { url: "demo://x/".into() });
        assert_eq!(state.status, 500);
    }

    #[test]
    fn options_bitor_combines_bits() {
        let combined = CefSchemeOptions::STANDARD | CefSchemeOptions::SECURE;
        assert_eq!(combined.0, CefSchemeOptions::STANDARD.0 | CefSchemeOptions::SECURE.0);
        assert_ne!(combined.0 & CefSchemeOptions::STANDARD.0, 0);
        assert_ne!(combined.0 & CefSchemeOptions::SECURE.0, 0);
    }

    #[test]
    fn options_include_csp_bypassing() {
        // Regression: an earlier design draft omitted this flag.
        assert_eq!(
            CefSchemeOptions::CSP_BYPASSING.0,
            CEF_SCHEME_OPTION_CSP_BYPASSING as u32
        );
    }
}
