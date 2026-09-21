use bevy::prelude::*;
use cef::rc::{Rc, RcImpl};
use cef::*;
use cef_dll_sys::cef_paint_element_type_t;
#[cfg(not(target_os = "macos"))]
use std::cell::Cell;
use std::os::raw::c_int;

/// A shared slot holding the latest texture for a single paint element type.
///
/// Uses `Rc<Cell<Option<T>>>` instead of a channel because both producer (`on_paint`)
/// and consumer (`send_render_textures`) run on the same thread (CEF UI thread =
/// Bevy main thread under `external_message_pump` mode). This eliminates all
/// synchronization overhead and naturally provides "latest frame wins" semantics.
///
/// Non-macOS only: the CPU `OnPaint` path (Linux and Windows). macOS uses the GPU
/// IOSurface accelerated-paint path (no slots).
#[cfg(not(target_os = "macos"))]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPaintElementType {
    /// The main frame of the browser.
    View,
    /// The popup frame of the browser.
    Popup,
}

pub type SharedViewSize = std::rc::Rc<std::cell::Cell<Vec2>>;

/// Slot for a webview's current `device_scale_factor`.
///
/// The CEF UI thread is the Bevy main thread on every platform
/// (`external_message_pump`), so no locking is needed.
pub type SharedDpr = std::rc::Rc<std::cell::Cell<f32>>;

/// ## Reference
///
/// - [`CefRenderHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/106.1/classCefRenderHandler.html)
pub struct RenderHandlerBuilder {
    object: *mut RcImpl<sys::cef_render_handler_t, Self>,
    webview: Entity,
    #[cfg(not(target_os = "macos"))]
    view_slot: SharedTexture,
    #[cfg(not(target_os = "macos"))]
    popup_slot: SharedTexture,
    size: SharedViewSize,
    dpr: SharedDpr,
    /// Latest retained IOSurface for this webview's main view (Approach 2).
    ///
    /// `on_accelerated_paint` does no GPU work here — it only retains the latest
    /// IOSurface into this slot. The `webview_blit` render-graph-schedule system
    /// imports and blits it using the render-world device, so no
    /// `RenderDevice`/`RenderQueue` is needed in the callback path.
    #[cfg(target_os = "macos")]
    latest_iosurface: crate::browser_process::accelerated_paint::SharedRetainedIoSurface,
}

impl RenderHandlerBuilder {
    #[cfg(target_os = "macos")]
    pub fn build(
        webview: Entity,
        size: SharedViewSize,
        dpr: SharedDpr,
        latest_iosurface: crate::browser_process::accelerated_paint::SharedRetainedIoSurface,
    ) -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
            webview,
            size,
            dpr,
            latest_iosurface,
        })
    }

    #[cfg(not(target_os = "macos"))]
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
            #[cfg(not(target_os = "macos"))]
            view_slot: self.view_slot.clone(),
            #[cfg(not(target_os = "macos"))]
            popup_slot: self.popup_slot.clone(),
            size: self.size.clone(),
            dpr: self.dpr.clone(),
            #[cfg(target_os = "macos")]
            latest_iosurface: self.latest_iosurface.clone(),
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

    fn screen_info(
        &self,
        _browser: Option<&mut Browser>,
        screen_info: Option<&mut cef::ScreenInfo>,
    ) -> c_int {
        let Some(info) = screen_info else { return 0 };

        let dpr = self.dpr.get();

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

    // macOS uses the GPU accelerated-paint path (on_accelerated_paint);
    // `on_paint` is never called when `shared_texture_enabled` is true, so we
    // don't override it at all on macOS (the trait provides a default no-op).
    #[cfg(not(target_os = "macos"))]
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

        let slot = match ty {
            RenderPaintElementType::Popup => &self.popup_slot,
            RenderPaintElementType::View => &self.view_slot,
        };
        slot.set(Some(texture));
    }

    #[cfg(target_os = "macos")]
    fn on_accelerated_paint(
        &self,
        _browser: Option<&mut Browser>,
        type_: PaintElementType,
        _dirty_rects: Option<&[cef::Rect]>,
        info: Option<&AcceleratedPaintInfo>,
    ) {
        // MVP: handle the main view only; popup widgets (PET_POPUP — e.g.
        // <select> dropdown lists) are dropped and therefore do not render on
        // macOS yet.
        //
        // Approach 2: do NO GPU work here. Bevy owns ordered command submission
        // (its render graph submits once per frame and then presents); an
        // out-of-band `queue.submit` from this callback (which runs in the Main
        // schedule via `cef_do_message_loop_work`) corrupts rendering. So we only
        // *retain* the freshly delivered IOSurface and publish it to the latest-
        // frame slot. The actual import + blit happens in the `webview_blit`
        // render-graph-schedule system that records into the frame's command
        // encoder.
        if !matches!(type_.as_ref(), cef_paint_element_type_t::PET_VIEW) {
            return;
        }
        let Some(info) = info else { return };
        // The whole downstream path assumes BGRA (Bgra8UnormSrgb import, alpha at
        // byte +3). macOS OSR delivers BGRA today; if Chromium ever switches the
        // capture format, drop the frame loudly instead of silently swapping R/B.
        if !matches!(
            info.format.as_ref(),
            cef_dll_sys::cef_color_type_t::CEF_COLOR_TYPE_BGRA_8888
        ) {
            bevy::log::error_once!(
                "[macos-gpu-osr] unsupported accelerated-paint color type {:?} \
                 (expected CEF_COLOR_TYPE_BGRA_8888); dropping frames",
                info.format.as_ref()
            );
            return;
        }
        if info.shared_texture_io_surface.is_null() {
            return;
        }
        let width = info.extra.coded_size.width as u32;
        let height = info.extra.coded_size.height as u32;
        if width == 0 || height == 0 {
            return;
        }

        // Safety: null-checked above; the pointer is valid for the duration of
        // this callback.
        let retained = unsafe {
            crate::browser_process::accelerated_paint::RetainedIoSurface::retain(
                info.shared_texture_io_surface,
                width,
                height,
            )
        };

        *self.latest_iosurface.borrow_mut() = Some(retained);
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::_cef_render_handler_t {
        self.object.cast()
    }
}
