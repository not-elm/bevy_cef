use async_channel::{Receiver, Sender};
use bevy::prelude::{Entity, Event};
use cef::rc::{Rc, RcImpl};
use cef::{
    Browser, CefString, ImplRenderHandler, PaintElementType, Range, Rect, RenderHandler,
    WrapRenderHandler, sys,
};
use cef_dll_sys::cef_paint_element_type_t;
use std::cell::Cell;
use std::os::raw::c_int;

pub type TextureSender = Sender<RenderTexture>;

pub type TextureReceiver = Receiver<RenderTexture>;

/// The texture structure passed from [`CefRenderHandler::OnPaint`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRenderHandler.html#a6547d5c9dd472e6b84706dc81d3f1741).
#[derive(Debug, Clone, PartialEq, Event)]
pub struct RenderTexture {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPaintElementType {
    /// The main frame of the browser.
    View,
    /// The popup frame of the browser.
    Popup,
}

pub type SharedViewSize = std::rc::Rc<Cell<bevy::prelude::Vec2>>;
pub type SharedImeCaret = std::rc::Rc<Cell<u32>>;

/// ## Reference
///
/// - [`CefRenderHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRenderHandler.html)
pub struct RenderHandlerBuilder {
    object: *mut RcImpl<sys::cef_render_handler_t, Self>,
    webview: Entity,
    texture_sender: TextureSender,
    size: SharedViewSize,
    ime_caret: SharedImeCaret,
}

impl RenderHandlerBuilder {
    pub fn build(
        webview: Entity,
        texture_sender: TextureSender,
        size: SharedViewSize,
        ime_caret: SharedImeCaret,
    ) -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
            webview,
            texture_sender,
            size,
            ime_caret,
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
            texture_sender: self.texture_sender.clone(),
            size: self.size.clone(),
            ime_caret: self.ime_caret.clone(),
        }
    }
}

impl ImplRenderHandler for RenderHandlerBuilder {
    fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
        if let Some(rect) = rect {
            let size = self.size.get();
            rect.width = size.x as _;
            rect.height = size.y as _;
        }
    }

    fn on_text_selection_changed(
        &self,
        _browser: Option<&mut Browser>,
        _: Option<&CefString>,
        selected_range: Option<&Range>,
    ) {
        if let Some(selected_range) = selected_range {
            self.ime_caret.set(selected_range.to);
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn on_paint(
        &self,
        _browser: Option<&mut Browser>,
        type_: PaintElementType,
        _dirty_rects_count: usize,
        _dirty_rects: Option<&Rect>,
        buffer: *const u8,
        width: c_int,
        height: c_int,
    ) {
        let ty = match type_.as_ref() {
            cef_paint_element_type_t::PET_POPUP => RenderPaintElementType::Popup,
            _ => RenderPaintElementType::View,
        };
        let texture = RenderTexture {
            webview: self.webview,
            ty,
            width: width as u32,
            height: height as u32,
            buffer: unsafe {
                std::slice::from_raw_parts(buffer, (width * height * 4) as usize).to_vec()
            },
        };
        let _ = self.texture_sender.send_blocking(texture);
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_render_handler_t {
        self.object.cast()
    }
}
