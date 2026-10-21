//! CEF LifeSpanHandler — tracks which browsers are alive.

use cef::rc::{Rc, RcImpl};
use cef::{Browser, ImplBrowser, ImplLifeSpanHandler, WrapLifeSpanHandler, sys};
use std::cell::RefCell;

/// The browsers CEF has created and not yet closed, popups and DevTools included.
pub type SharedLiveBrowsers = std::rc::Rc<RefCell<Vec<Browser>>>;

/// Adds each created browser to a [`SharedLiveBrowsers`] list and removes it when CEF closes it.
///
/// ## Reference
///
/// - [`CefLifeSpanHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/145/classCefLifeSpanHandler.html)
pub struct LifeSpanHandlerBuilder {
    object: *mut RcImpl<sys::_cef_life_span_handler_t, Self>,
    live: SharedLiveBrowsers,
}

impl LifeSpanHandlerBuilder {
    /// Builds a handler that records into `live`.
    ///
    /// A popup shares its opener's client, so it is recorded in the same list.
    pub fn build(live: SharedLiveBrowsers) -> cef::LifeSpanHandler {
        cef::LifeSpanHandler::new(Self {
            object: core::ptr::null_mut(),
            live,
        })
    }
}

impl Rc for LifeSpanHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            core::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for LifeSpanHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self {
            object,
            live: self.live.clone(),
        }
    }
}

impl WrapLifeSpanHandler for LifeSpanHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_life_span_handler_t, Self>) {
        self.object = object;
    }
}

impl ImplLifeSpanHandler for LifeSpanHandlerBuilder {
    fn on_after_created(&self, browser: Option<&mut Browser>) {
        if let Some(browser) = browser {
            self.live.borrow_mut().push(browser.clone());
        }
    }

    fn on_before_close(&self, browser: Option<&mut Browser>) {
        if let Some(browser) = browser {
            let identifier = browser.identifier();
            self.live
                .borrow_mut()
                .retain(|live| live.identifier() != identifier);
        }
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_life_span_handler_t {
        self.object.cast()
    }
}
