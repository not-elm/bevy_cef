//! Coalesced cookie-store flushes for disk-backed profile contexts.

use bevy::prelude::warn;
use cef::rc::{Rc, RcImpl};
use cef::{
    CompletionCallback, ImplCompletionCallback, ImplCookieManager, ImplRequestContext,
    RequestContext, WrapCompletionCallback, sys,
};
use std::cell::Cell;

/// Where a context's flush stands: none running, or one running with an optional follow-up
/// queued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlushState {
    /// No flush is running.
    Idle,
    /// A flush is running.
    InFlight {
        /// Whether a follow-up flush must start when the running one completes.
        again: bool,
    },
}

impl FlushState {
    /// Records a flush request and returns whether a flush should start now.
    ///
    /// While a flush is in flight the request only sets `again`, so any number of requests
    /// collapse into one follow-up.
    pub fn request(&mut self) -> bool {
        match self {
            Self::Idle => {
                *self = Self::InFlight { again: false };
                true
            }
            Self::InFlight { again } => {
                *again = true;
                false
            }
        }
    }

    /// Records that the running flush finished and returns whether a follow-up should start now.
    pub fn complete(&mut self) -> bool {
        match self {
            Self::InFlight { again: true } => {
                *self = Self::InFlight { again: false };
                true
            }
            _ => {
                *self = Self::Idle;
                false
            }
        }
    }
}

/// Flushes one request context's cookie store, with at most one flush in flight.
///
/// Clones share the same state. Each flush fetches the context's cookie manager afresh and
/// holds no manager between flushes.
#[derive(Clone)]
pub struct CookieFlusher {
    context: RequestContext,
    state: std::rc::Rc<Cell<FlushState>>,
}

impl CookieFlusher {
    /// Creates a flusher for `context`.
    pub fn new(context: RequestContext) -> Self {
        Self {
            context,
            state: std::rc::Rc::new(Cell::new(FlushState::Idle)),
        }
    }

    /// Requests a flush, starting one unless another is in flight.
    pub fn request(&self) {
        let mut state = self.state.get();
        let start = state.request();
        self.state.set(state);
        if start {
            self.start();
        }
    }

    fn complete(&self) {
        let mut state = self.state.get();
        let again = state.complete();
        self.state.set(state);
        if again {
            self.start();
        }
    }

    fn start(&self) {
        let Some(manager) = self.context.cookie_manager(None) else {
            warn!("bevy_cef: a profile context has no cookie manager; cookies were not flushed");
            self.state.set(FlushState::Idle);
            return;
        };
        let mut callback = FlushCompletionBuilder::build(self.clone());
        if manager.flush_store(Some(&mut callback)) == 0 {
            warn!("bevy_cef: CEF refused to flush a profile's cookie store");
            self.state.set(FlushState::Idle);
        }
    }
}

struct FlushCompletionBuilder {
    object: *mut RcImpl<sys::_cef_completion_callback_t, Self>,
    flusher: CookieFlusher,
}

impl FlushCompletionBuilder {
    fn build(flusher: CookieFlusher) -> CompletionCallback {
        CompletionCallback::new(Self {
            object: core::ptr::null_mut(),
            flusher,
        })
    }
}

impl Rc for FlushCompletionBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            core::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for FlushCompletionBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self {
            object,
            flusher: self.flusher.clone(),
        }
    }
}

impl WrapCompletionCallback for FlushCompletionBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_completion_callback_t, Self>) {
        self.object = object;
    }
}

impl ImplCompletionCallback for FlushCompletionBuilder {
    fn on_complete(&self) {
        self.flusher.complete();
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_completion_callback_t {
        self.object.cast()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that a request while idle starts a flush and marks it in flight.
    ///
    /// Case: a profile webview finishes loading and no flush is running.
    #[test]
    fn an_idle_request_starts_a_flush() {
        let mut state = FlushState::Idle;
        assert!(state.request());
        assert_eq!(state, FlushState::InFlight { again: false });
    }

    /// Asserts that requests during a flush collapse into a single follow-up flush.
    ///
    /// Case: several pages of one profile finish loading while a flush is still running.
    #[test]
    fn requests_during_a_flush_coalesce_into_one_follow_up() {
        let mut state = FlushState::InFlight { again: false };
        assert!(!state.request());
        assert!(!state.request());
        assert_eq!(state, FlushState::InFlight { again: true });
        assert!(state.complete());
        assert!(!state.complete());
        assert_eq!(state, FlushState::Idle);
    }
}
