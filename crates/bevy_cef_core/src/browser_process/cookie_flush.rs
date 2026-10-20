//! Coalesced cookie-store flushes for disk-backed profile contexts.

use crate::util::IntoString;
use bevy::prelude::warn;
use cef::rc::{Rc, RcImpl};
use cef::{
    CompletionCallback, ImplCompletionCallback, ImplCookieManager, ImplRequestContext,
    RequestContext, WrapCompletionCallback, sys,
};
use std::cell::Cell;
use std::rc::Weak;

/// Where a context's flush stands: none running, or one running with an optional follow-up
/// queued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlushState {
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
    fn request(&mut self) -> bool {
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
    fn complete(&mut self) -> bool {
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

    /// Records that the running flush was dropped without completing and returns whether one
    /// was in flight.
    ///
    /// The state becomes `Idle`, discarding any queued follow-up.
    fn abandon(&mut self) -> bool {
        let was_in_flight = matches!(self, Self::InFlight { .. });
        *self = Self::Idle;
        was_in_flight
    }
}

struct FlushTarget {
    context: RequestContext,
    state: Cell<FlushState>,
}

impl FlushTarget {
    fn request(self: &std::rc::Rc<Self>) {
        let mut state = self.state.get();
        let start = state.request();
        self.state.set(state);
        if start {
            self.start();
        }
    }

    fn complete(self: &std::rc::Rc<Self>) {
        let mut state = self.state.get();
        let again = state.complete();
        self.state.set(state);
        if again {
            self.start();
        }
    }

    fn start(self: &std::rc::Rc<Self>) {
        let Some(manager) = self.context.cookie_manager(None) else {
            warn!(
                "bevy_cef: profile {} has no cookie manager; cookies were not flushed",
                self.cache_path()
            );
            self.state.set(FlushState::Idle);
            return;
        };
        let mut callback = FlushCompletionBuilder::build(std::rc::Rc::downgrade(self));
        if manager.flush_store(Some(&mut callback)) == 0 {
            warn!(
                "bevy_cef: CEF refused to flush the cookie store of profile {}",
                self.cache_path()
            );
            self.state.set(FlushState::Idle);
        }
    }

    fn cache_path(&self) -> String {
        self.context.cache_path().into_string()
    }
}

/// The sole strong owner of a profile context's flush target.
///
/// Dropping it releases the context; every [`CookieFlusher`] handle then does nothing.
pub(crate) struct ProfileFlusher {
    target: std::rc::Rc<FlushTarget>,
}

impl ProfileFlusher {
    /// Creates the owner for `context`.
    pub fn new(context: RequestContext) -> Self {
        Self {
            target: std::rc::Rc::new(FlushTarget {
                context,
                state: Cell::new(FlushState::Idle),
            }),
        }
    }

    /// Requests a flush, starting one unless another is in flight.
    pub fn request(&self) {
        self.target.request();
    }

    /// Returns a weak handle for a load handler.
    pub fn handle(&self) -> CookieFlusher {
        CookieFlusher {
            target: std::rc::Rc::downgrade(&self.target),
        }
    }
}

/// A weak handle that requests cookie flushes on a profile's context.
///
/// Requests do nothing once the profile context has been released.
#[derive(Clone)]
pub struct CookieFlusher {
    target: Weak<FlushTarget>,
}

impl CookieFlusher {
    /// Requests a flush, starting one unless another is in flight.
    pub fn request(&self) {
        if let Some(target) = self.target.upgrade() {
            target.request();
        }
    }
}

/// Resets the flush state when CEF drops the callback without calling `on_complete`.
struct CompletionGuard {
    target: Weak<FlushTarget>,
    completed: Cell<bool>,
}

impl Drop for CompletionGuard {
    fn drop(&mut self) {
        if self.completed.get() {
            return;
        }
        let Some(target) = self.target.upgrade() else {
            return;
        };
        let mut state = target.state.get();
        if state.abandon() {
            target.state.set(state);
            warn!(
                "bevy_cef: CEF dropped the cookie flush of profile {} without completing it",
                target.cache_path()
            );
        }
    }
}

struct FlushCompletionBuilder {
    object: *mut RcImpl<sys::_cef_completion_callback_t, Self>,
    guard: std::rc::Rc<CompletionGuard>,
}

impl FlushCompletionBuilder {
    fn build(target: Weak<FlushTarget>) -> CompletionCallback {
        CompletionCallback::new(Self {
            object: core::ptr::null_mut(),
            guard: std::rc::Rc::new(CompletionGuard {
                target,
                completed: Cell::new(false),
            }),
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
            guard: self.guard.clone(),
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
        self.guard.completed.set(true);
        if let Some(target) = self.guard.target.upgrade() {
            target.complete();
        }
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

    /// Asserts that abandoning an in-flight flush returns the state to idle, discarding a
    /// queued follow-up, and that abandoning an idle state reports nothing was in flight.
    ///
    /// Case: CEF drops a flush callback without calling it after the network service
    /// disconnects, and the profile must be able to flush again.
    #[test]
    fn an_abandoned_flush_returns_to_idle() {
        let mut state = FlushState::InFlight { again: true };
        assert!(state.abandon());
        assert_eq!(state, FlushState::Idle);
        assert!(!state.abandon());
        assert!(state.request());
    }
}
