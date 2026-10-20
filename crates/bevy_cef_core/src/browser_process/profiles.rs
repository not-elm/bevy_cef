//! The shared request contexts of named, disk-backed profiles.

use crate::browser_process::cookie_flush::CookieFlusher;
use cef::RequestContext;
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

/// Why a profile's request context could not be prepared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileContextError {
    /// CEF returned no request context.
    CreationFailed,
    /// CEF rejected the cache path and created an in-memory context instead.
    DidNotLoad,
}

impl fmt::Display for ProfileContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreationFailed => write!(f, "CEF did not create the profile's request context"),
            Self::DidNotLoad => write!(
                f,
                "CEF rejected the profile's cache path and fell back to in-memory storage"
            ),
        }
    }
}

impl std::error::Error for ProfileContextError {}

/// A profile's request context, shared by every webview of that profile.
pub(crate) struct ProfileContext {
    pub(crate) context: RequestContext,
    /// Set to `true` by the context's handler once CEF reports it initialized.
    pub(crate) ready: Rc<Cell<bool>>,
    /// Flushes the context's cookie store; held only here and by the profile's load handlers.
    pub(crate) flusher: CookieFlusher,
}

impl ProfileContext {
    /// Whether CEF has reported the context initialized, so browsers can be created on it.
    pub(crate) fn is_ready(&self) -> bool {
        self.ready.get()
    }
}
