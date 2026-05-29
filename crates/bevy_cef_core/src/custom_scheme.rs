//! Caller-registered custom scheme support: public types, a process-global
//! registry, and a generic CEF factory/resource-handler adapter.

use cef_dll_sys::cef_scheme_options_t::{
    CEF_SCHEME_OPTION_CORS_ENABLED, CEF_SCHEME_OPTION_CSP_BYPASSING,
    CEF_SCHEME_OPTION_DISPLAY_ISOLATED, CEF_SCHEME_OPTION_FETCH_ENABLED,
    CEF_SCHEME_OPTION_LOCAL, CEF_SCHEME_OPTION_SECURE, CEF_SCHEME_OPTION_STANDARD,
};
use serde::{Deserialize, Serialize};
use std::io::{self, Cursor, Read};
use std::sync::Arc;

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
struct CefSchemeDecl {
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
fn parse_decls_json(json: &str) -> Vec<CefSchemeDecl> {
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("bevy_cef: failed to parse custom-scheme switch JSON: {}", e);
        Vec::new()
    })
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
        assert_eq!(CefSchemeResponse::not_found().status, 404);
    }

    #[test]
    fn bytes_constructor_is_200_with_mime() {
        let r = CefSchemeResponse::bytes("text/css", b"body{}".to_vec());
        assert_eq!(r.status, 200);
        assert_eq!(r.mime_type, "text/css");
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
