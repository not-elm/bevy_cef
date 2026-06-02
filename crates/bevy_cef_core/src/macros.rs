//! Internal logging macros. They forward to `bevy::log` only when the `log`
//! feature is enabled; otherwise they expand to a zero-cost no-op that still
//! references the format args (so disabling `log` never triggers
//! `unused_variables`). Every message is auto-prefixed with `bevy_cef: `.

macro_rules! cef_error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "log")]
        {
            ::bevy::log::error!("bevy_cef: {}", format_args!($($arg)*));
        }
        #[cfg(not(feature = "log"))]
        {
            let _ = format_args!($($arg)*);
        }
    }};
}
pub(crate) use cef_error;

macro_rules! cef_warn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "log")]
        {
            ::bevy::log::warn!("bevy_cef: {}", format_args!($($arg)*));
        }
        #[cfg(not(feature = "log"))]
        {
            let _ = format_args!($($arg)*);
        }
    }};
}
pub(crate) use cef_warn;
