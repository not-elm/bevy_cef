//! The shared request contexts of named, disk-backed profiles.

use crate::browser_process::cookie_flush::ProfileFlusher;
use cef::RequestContext;
use std::cell::Cell;
use std::rc::Rc;

/// A profile's request context, shared by every webview of that profile.
pub(crate) struct ProfileContext {
    pub(crate) context: RequestContext,
    /// Set to `true` by the context's handler once CEF reports it initialized.
    pub(crate) ready: Rc<Cell<bool>>,
    /// The sole strong owner of the context's flush target; load handlers hold weak handles.
    pub(crate) flusher: ProfileFlusher,
}

impl ProfileContext {
    /// Whether CEF has reported the context initialized, so browsers can be created on it.
    pub(crate) fn is_ready(&self) -> bool {
        self.ready.get()
    }
}
