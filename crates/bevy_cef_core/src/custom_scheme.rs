//! Caller-registered custom scheme support: public types, a process-global
//! registry, and a generic CEF factory/resource-handler adapter.

use cef_dll_sys::cef_scheme_options_t::{
    CEF_SCHEME_OPTION_CORS_ENABLED, CEF_SCHEME_OPTION_CSP_BYPASSING,
    CEF_SCHEME_OPTION_DISPLAY_ISOLATED, CEF_SCHEME_OPTION_FETCH_ENABLED,
    CEF_SCHEME_OPTION_LOCAL, CEF_SCHEME_OPTION_SECURE, CEF_SCHEME_OPTION_STANDARD,
};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
