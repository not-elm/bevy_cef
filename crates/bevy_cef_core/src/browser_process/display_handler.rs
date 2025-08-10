use async_channel::Sender;
use bevy::log::{error, info, trace, warn};
use bevy::window::SystemCursorIcon;
use cef::rc::{ConvertParam, Rc, RcImpl};
use cef::{
    Browser, CefString, CursorInfo, CursorType, ImplDisplayHandler, LogSeverity,
    WrapDisplayHandler, sys,
};
use cef_dll_sys::{cef_cursor_type_t, cef_log_severity_t};
use std::os::raw::c_int;

pub type SystemCursorIconSenderInner = Sender<SystemCursorIcon>;

/// ## Reference
///
/// - [`CefDisplayHandler Class Reference`](https://cef-builds.spotifycdn.com/docs/112.3/classCefDisplayHandler.html#af1cc8410a0b1a97166923428d3794636)
pub struct DisplayHandlerBuilder {
    object: *mut RcImpl<sys::cef_display_handler_t, Self>,
    cursor_icon: SystemCursorIconSenderInner,
}

impl DisplayHandlerBuilder {
    pub fn build(cursor_icon: SystemCursorIconSenderInner) -> cef::DisplayHandler {
        cef::DisplayHandler::new(Self {
            object: core::ptr::null_mut(),
            cursor_icon,
        })
    }
}

impl Rc for DisplayHandlerBuilder {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            core::mem::transmute(&base.cef_object)
        }
    }
}

impl Clone for DisplayHandlerBuilder {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };
        Self {
            object,
            cursor_icon: self.cursor_icon.clone(),
        }
    }
}

impl WrapDisplayHandler for DisplayHandlerBuilder {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::cef_display_handler_t, Self>) {
        self.object = object;
    }
}

impl ImplDisplayHandler for DisplayHandlerBuilder {
    fn on_console_message(
        &self,
        _: Option<&mut Browser>,
        level: LogSeverity,
        message: Option<&CefString>,
        source: Option<&CefString>,
        line: c_int,
    ) -> c_int {
        let message = format!(
            "{}\nline:{line}\n{}",
            source.map(|s| s.to_string()).unwrap_or_default(),
            message.map(|m| m.to_string()).unwrap_or_default()
        );
        match level.into_raw() {
            cef_log_severity_t::LOGSEVERITY_ERROR => {
                error!("{message}");
            }
            cef_log_severity_t::LOGSEVERITY_WARNING => {
                warn!("{message}");
            }
            cef_log_severity_t::LOGSEVERITY_VERBOSE => {
                trace!("{message}");
            }
            _ => {
                info!("{message}");
            }
        }
        1
    }

    fn on_cursor_change(
        &self,
        _browser: Option<&mut Browser>,
        #[cfg(not(target_os = "linux"))]
        _cursor: *mut u8,
        #[cfg(target_os = "linux")]
        _cursor: u64,
        type_: CursorType,
        _: Option<&CursorInfo>,
    ) -> c_int {
        let _ = self
            .cursor_icon
            .send_blocking(to_system_cursor_icon(type_.into_raw()));
        1
    }

    #[inline]
    fn get_raw(&self) -> *mut sys::cef_display_handler_t {
        self.object.cast()
    }
}

pub fn to_system_cursor_icon(cursor_type: cef_dll_sys::cef_cursor_type_t) -> SystemCursorIcon {
    match cursor_type {
        cef_cursor_type_t::CT_POINTER => SystemCursorIcon::Default,
        cef_cursor_type_t::CT_CROSS => SystemCursorIcon::Crosshair,
        cef_cursor_type_t::CT_HAND => SystemCursorIcon::Pointer,
        cef_cursor_type_t::CT_IBEAM => SystemCursorIcon::Text,
        cef_cursor_type_t::CT_WAIT => SystemCursorIcon::Wait,
        cef_cursor_type_t::CT_HELP => SystemCursorIcon::Help,
        cef_cursor_type_t::CT_EASTRESIZE => SystemCursorIcon::EResize,
        cef_cursor_type_t::CT_NORTHRESIZE => SystemCursorIcon::NResize,
        cef_cursor_type_t::CT_NORTHEASTRESIZE => SystemCursorIcon::NeResize,
        cef_cursor_type_t::CT_NORTHWESTRESIZE => SystemCursorIcon::NwResize,
        cef_cursor_type_t::CT_SOUTHRESIZE => SystemCursorIcon::SResize,
        cef_cursor_type_t::CT_SOUTHEASTRESIZE => SystemCursorIcon::SeResize,
        cef_cursor_type_t::CT_SOUTHWESTRESIZE => SystemCursorIcon::SwResize,
        cef_cursor_type_t::CT_WESTRESIZE => SystemCursorIcon::WResize,
        cef_cursor_type_t::CT_NORTHSOUTHRESIZE => SystemCursorIcon::NsResize,
        cef_cursor_type_t::CT_EASTWESTRESIZE => SystemCursorIcon::EwResize,
        cef_cursor_type_t::CT_NORTHEASTSOUTHWESTRESIZE => SystemCursorIcon::NeswResize,
        cef_cursor_type_t::CT_NORTHWESTSOUTHEASTRESIZE => SystemCursorIcon::NwseResize,
        cef_cursor_type_t::CT_COLUMNRESIZE => SystemCursorIcon::ColResize,
        cef_cursor_type_t::CT_ROWRESIZE => SystemCursorIcon::RowResize,
        cef_cursor_type_t::CT_MIDDLEPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_EASTPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_NORTHPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_NORTHEASTPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_NORTHWESTPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_SOUTHPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_SOUTHEASTPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_SOUTHWESTPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_WESTPANNING => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_MOVE => SystemCursorIcon::Move,
        cef_cursor_type_t::CT_VERTICALTEXT => SystemCursorIcon::VerticalText,
        cef_cursor_type_t::CT_CELL => SystemCursorIcon::Cell,
        cef_cursor_type_t::CT_CONTEXTMENU => SystemCursorIcon::ContextMenu,
        cef_cursor_type_t::CT_ALIAS => SystemCursorIcon::Alias,
        cef_cursor_type_t::CT_PROGRESS => SystemCursorIcon::Progress,
        cef_cursor_type_t::CT_NODROP => SystemCursorIcon::NoDrop,
        cef_cursor_type_t::CT_COPY => SystemCursorIcon::Copy,
        cef_cursor_type_t::CT_NONE => SystemCursorIcon::Default,
        cef_cursor_type_t::CT_NOTALLOWED => SystemCursorIcon::NotAllowed,
        cef_cursor_type_t::CT_ZOOMIN => SystemCursorIcon::ZoomIn,
        cef_cursor_type_t::CT_ZOOMOUT => SystemCursorIcon::ZoomOut,
        cef_cursor_type_t::CT_GRAB => SystemCursorIcon::Grab,
        cef_cursor_type_t::CT_GRABBING => SystemCursorIcon::Grabbing,
        cef_cursor_type_t::CT_MIDDLE_PANNING_VERTICAL => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_MIDDLE_PANNING_HORIZONTAL => SystemCursorIcon::AllScroll,
        cef_cursor_type_t::CT_CUSTOM => SystemCursorIcon::Default,
        cef_cursor_type_t::CT_DND_NONE => SystemCursorIcon::Default,
        cef_cursor_type_t::CT_DND_MOVE => SystemCursorIcon::Move,
        cef_cursor_type_t::CT_DND_COPY => SystemCursorIcon::Copy,
        cef_cursor_type_t::CT_DND_LINK => SystemCursorIcon::Alias,
        cef_cursor_type_t::CT_NUM_VALUES => SystemCursorIcon::Default,
        _ => SystemCursorIcon::Default,
    }
}
