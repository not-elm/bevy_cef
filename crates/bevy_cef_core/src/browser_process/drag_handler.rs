//! CEF DragHandler — receives draggable region updates from the browser.
//!
//! Mirrors the `DisplayHandlerBuilder` pattern (`display_handler.rs`).

use async_channel::Sender;
use bevy::log::info;
use bevy::prelude::Entity;
use cef::rc::{Rc, RcImpl};
use cef::{Browser, DraggableRegion, Frame, ImplDragHandler, WrapDragHandler, sys};

pub type DraggableRegionSenderInner = Sender<(Entity, Vec<DraggableRegion>)>;

/// ## Reference
///
/// - [`CefDragHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/145/classCefDragHandler.html)
pub struct DragHandlerBuilder {
    object: *mut RcImpl<sys::cef_drag_handler_t, Self>,
    webview: Entity,
    sender: DraggableRegionSenderInner,
}

impl DragHandlerBuilder {
    pub fn build(webview: Entity, sender: DraggableRegionSenderInner) -> cef::DragHandler {
        cef::DragHandler::new(Self {
            object: core::ptr::null_mut(),
            webview,
            sender,
        })
    }
}

impl Rc for DragHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            core::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for DragHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self {
            object,
            webview: self.webview,
            sender: self.sender.clone(),
        }
    }
}

impl WrapDragHandler for DragHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::cef_drag_handler_t, Self>) {
        self.object = object;
    }
}

impl ImplDragHandler for DragHandlerBuilder {
    fn on_draggable_regions_changed(
        &self,
        _browser: Option<&mut Browser>,
        _frame: Option<&mut Frame>,
        regions: Option<&[DraggableRegion]>,
    ) {
        let regions_vec = regions.unwrap_or(&[]).to_vec();
        info!(
            "[OSR SPIKE] on_draggable_regions_changed fired: entity={:?}, count={}, regions={:?}",
            self.webview,
            regions_vec.len(),
            regions_vec
                .iter()
                .map(|r| (r.bounds.x, r.bounds.y, r.bounds.width, r.bounds.height, r.draggable))
                .collect::<Vec<_>>()
        );
        let _ = self.sender.send_blocking((self.webview, regions_vec));
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::cef_drag_handler_t {
        self.object.cast()
    }
}
