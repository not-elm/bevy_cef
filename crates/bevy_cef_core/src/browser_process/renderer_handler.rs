use bevy::prelude::*;
use cef::rc::{Rc, RcImpl};
use cef::*;
use cef_dll_sys::cef_paint_element_type_t;
use std::cell::Cell;
use std::os::raw::c_int;

/// A shared slot holding the latest texture for a single paint element type.
///
/// Uses `Rc<Cell<Option<T>>>` instead of a channel because both producer (`on_paint`)
/// and consumer (`send_render_textures`) run on the same thread (CEF UI thread =
/// Bevy main thread under `external_message_pump` mode). This eliminates all
/// synchronization overhead and naturally provides "latest frame wins" semantics.
pub type SharedTexture = std::rc::Rc<Cell<Option<RenderTextureMessage>>>;

#[cfg(target_os = "windows")]
pub type TextureSender = async_channel::Sender<RenderTextureMessage>;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPaintElementType {
    /// The main frame of the browser.
    View,
    /// The popup frame of the browser.
    Popup,
}

#[cfg(not(target_os = "windows"))]
pub type SharedViewSize = std::rc::Rc<std::cell::Cell<Vec2>>;
#[cfg(target_os = "windows")]
pub type SharedViewSize = std::sync::Arc<std::sync::Mutex<Vec2>>;

/// Thread-safe slot for a webview's current `device_scale_factor`.
///
/// Mirrors `SharedViewSize`'s platform split: on non-Windows the CEF UI
/// thread is the Bevy main thread, so no locking is needed; on Windows the
/// CEF UI thread is separate, so an `Arc<Mutex<_>>` is required.
#[cfg(not(target_os = "windows"))]
pub type SharedDpr = std::rc::Rc<std::cell::Cell<f32>>;
#[cfg(target_os = "windows")]
pub type SharedDpr = std::sync::Arc<std::sync::Mutex<f32>>;

/// ## Reference
///
/// - [`CefRenderHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRenderHandler.html)
pub struct RenderHandlerBuilder {
    object: *mut RcImpl<sys::cef_render_handler_t, Self>,
    webview: Entity,
    #[cfg(not(target_os = "windows"))]
    view_slot: SharedTexture,
    #[cfg(not(target_os = "windows"))]
    popup_slot: SharedTexture,
    #[cfg(target_os = "windows")]
    texture_sender: TextureSender,
    size: SharedViewSize,
    dpr: SharedDpr,
}

impl RenderHandlerBuilder {
    #[cfg(not(target_os = "windows"))]
    pub fn build(
        webview: Entity,
        view_slot: SharedTexture,
        popup_slot: SharedTexture,
        size: SharedViewSize,
        dpr: SharedDpr,
    ) -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
            webview,
            view_slot,
            popup_slot,
            size,
            dpr,
        })
    }

    #[cfg(target_os = "windows")]
    pub fn build(
        webview: Entity,
        texture_sender: TextureSender,
        size: SharedViewSize,
        dpr: SharedDpr,
    ) -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
            webview,
            texture_sender,
            size,
            dpr,
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
            #[cfg(not(target_os = "windows"))]
            view_slot: self.view_slot.clone(),
            #[cfg(not(target_os = "windows"))]
            popup_slot: self.popup_slot.clone(),
            #[cfg(target_os = "windows")]
            texture_sender: self.texture_sender.clone(),
            size: self.size.clone(),
            dpr: self.dpr.clone(),
        }
    }
}

impl ImplRenderHandler for RenderHandlerBuilder {
    fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut cef::Rect>) {
        if let Some(rect) = rect {
            #[cfg(not(target_os = "windows"))]
            let size = self.size.get();
            #[cfg(target_os = "windows")]
            let size = *self.size.lock().unwrap();
            rect.width = size.x as _;
            rect.height = size.y as _;
        }
    }

    fn screen_info(
        &self,
        _browser: Option<&mut Browser>,
        screen_info: Option<&mut cef::ScreenInfo>,
    ) -> c_int {
        let Some(info) = screen_info else { return 0 };

        #[cfg(not(target_os = "windows"))]
        let dpr = self.dpr.get();
        #[cfg(target_os = "windows")]
        let dpr = *self.dpr.lock().unwrap();

        info.device_scale_factor = dpr;
        info.depth = 24;
        info.depth_per_component = 8;
        info.is_monochrome = 0;
        // `rect` / `available_rect` describe the monitor in virtual-screen coords
        // per CEF (`cef_types.h:1911-1923`), not the view size. For HiDPI quality
        // only `device_scale_factor` matters — leave rects at their defaults.
        info.rect = cef::Rect::default();
        info.available_rect = cef::Rect::default();
        1
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
        };

        #[cfg(not(target_os = "windows"))]
        {
            let slot = match ty {
                RenderPaintElementType::Popup => &self.popup_slot,
                RenderPaintElementType::View => &self.view_slot,
            };
            slot.set(Some(texture));
        }

        #[cfg(target_os = "windows")]
        {
            let _ = self.texture_sender.send_blocking(texture);
        }
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_render_handler_t {
        self.object.cast()
    }
}
