//! Configuration for Chromium's OS-level process sandbox.

/// Controls Chromium's OS-level sandbox (`Settings.no_sandbox`).
///
/// The default, [`SandboxMode::PlatformDefault`], reproduces bevy_cef's existing
/// per-platform behavior: sandbox **on** for macOS release builds; **off** for
/// macOS debug builds, Windows, and Linux.
///
/// Enabling the sandbox is best-effort and platform-specific (Linux needs a SUID
/// `chrome-sandbox` helper; macOS needs a `cef_sandbox`-linked render process).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SandboxMode {
    /// Preserve bevy_cef's current per-platform default.
    #[default]
    PlatformDefault,
    /// Force the Chromium sandbox ON (`no_sandbox = false`).
    Enabled,
    /// Force the Chromium sandbox OFF (`no_sandbox = true`).
    Disabled,
}

/// Resolves a [`SandboxMode`] to the `no_sandbox` boolean passed to CEF `Settings`.
///
/// Lives in the root `bevy_cef` crate so `cfg!(feature = "debug")` observes the same
/// `debug` feature value as the historical `#[cfg(feature = "debug")]` gate in
/// `message_loop.rs`.
pub fn resolve_no_sandbox(mode: SandboxMode) -> bool {
    match mode {
        SandboxMode::Enabled => false,
        SandboxMode::Disabled => true,
        SandboxMode::PlatformDefault => {
            #[cfg(target_os = "macos")]
            {
                cfg!(feature = "debug")
            }
            #[cfg(not(target_os = "macos"))]
            {
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_means_sandbox_on() {
        assert!(!resolve_no_sandbox(SandboxMode::Enabled));
    }

    #[test]
    fn disabled_means_no_sandbox() {
        assert!(resolve_no_sandbox(SandboxMode::Disabled));
    }

    // PlatformDefault on macOS legitimately depends on the `debug` feature, so we do
    // not assert a specific value there (that would just mirror the implementation).
    // On every other platform the invariant is concrete: the sandbox is off by default.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn platform_default_is_no_sandbox_on_non_macos() {
        assert!(resolve_no_sandbox(SandboxMode::PlatformDefault));
    }
}
