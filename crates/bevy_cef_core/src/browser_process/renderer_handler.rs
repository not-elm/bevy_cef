use bevy::prelude::*;
use cef::rc::{Rc, RcImpl};
use cef::*;
use cef_dll_sys::cef_paint_element_type_t;
use std::cell::Cell;
use std::os::raw::c_int;
use std::time::Instant;

/// A shared slot holding the latest texture for a single paint element type.
///
/// Uses `Rc<Cell<Option<T>>>` instead of a channel because both producer (`on_paint`)
/// and consumer (`send_render_textures`) run on the same thread (CEF UI thread =
/// Bevy main thread under `external_message_pump` mode). This eliminates all
/// synchronization overhead and naturally provides "latest frame wins" semantics.
pub type SharedTexture = std::rc::Rc<Cell<Option<RenderTextureMessage>>>;

/// The texture structure passed from [`CefRenderHandler::OnPaint`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRenderHandler.html#a6547d5c9dd472e6b84706dc81d3f1741).
#[derive(Debug, Clone, PartialEq, Message)]
pub struct RenderTextureMessage {
    /// The entity of target rendering webview.
    pub webview: Entity,
    /// The type of the paint element.
    pub ty: RenderPaintElementType,
    /// The width of the texture.
    pub width: u32,
    /// The height of the texture.
    pub height: u32,
    /// This buffer will be `width` *`height` * 4 bytes in size and represents a BGRA image with an upper-left origin
    pub buffer: Vec<u8>,
    /// Timestamp when the texture was created in the CEF on_paint callback.
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPaintElementType {
    /// The main frame of the browser.
    View,
    /// The popup frame of the browser.
    Popup,
}

pub type SharedViewSize = std::rc::Rc<Cell<Vec2>>;

/// ## Reference
///
/// - [`CefRenderHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRenderHandler.html)
pub struct RenderHandlerBuilder {
    object: *mut RcImpl<sys::cef_render_handler_t, Self>,
    webview: Entity,
    view_slot: SharedTexture,
    popup_slot: SharedTexture,
    size: SharedViewSize,
}

impl RenderHandlerBuilder {
    pub fn build(
        webview: Entity,
        view_slot: SharedTexture,
        popup_slot: SharedTexture,
        size: SharedViewSize,
    ) -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
            webview,
            view_slot,
            popup_slot,
            size,
        })
    }
}

impl Rc for RenderHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapRenderHandler for RenderHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_render_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for RenderHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self {
            object,
            webview: self.webview,
            view_slot: self.view_slot.clone(),
            popup_slot: self.popup_slot.clone(),
            size: self.size.clone(),
        }
    }
}

impl ImplRenderHandler for RenderHandlerBuilder {
    fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut cef::Rect>) {
        if let Some(rect) = rect {
            let size = self.size.get();
            rect.width = size.x as _;
            rect.height = size.y as _;
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn on_paint(
        &self,
        _browser: Option<&mut Browser>,
        type_: PaintElementType,
        _dirty_rects: Option<&[cef::Rect]>,
        buffer: *const u8,
        width: c_int,
        height: c_int,
    ) {
        let ty = match type_.as_ref() {
            cef_paint_element_type_t::PET_POPUP => RenderPaintElementType::Popup,
            _ => RenderPaintElementType::View,
        };
        let texture = RenderTextureMessage {
            webview: self.webview,
            ty,
            width: width as u32,
            height: height as u32,
            buffer: unsafe {
                std::slice::from_raw_parts(buffer, (width * height * 4) as usize).to_vec()
            },
            created_at: Instant::now(),
        };
        let slot = match ty {
            RenderPaintElementType::Popup => &self.popup_slot,
            RenderPaintElementType::View => &self.view_slot,
        };
        slot.set(Some(texture));
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_render_handler_t {
        self.object.cast()
    }
}
