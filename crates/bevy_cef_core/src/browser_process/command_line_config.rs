/// Configuration for CEF command line switches.
///
/// Used to customize CEF behavior at startup.
///
/// # Default Switches
///
/// On macOS debug builds, the following switches are enabled by default:
/// - `use-mock-keychain`: Uses a mock keychain for testing
///
/// # Example
///
/// ```no_run
/// use bevy_cef_core::prelude::*;
///
/// // Add switches while preserving defaults (recommended)
/// let config = CommandLineConfig::default()
///     .with_switch("disable-gpu")
///     .with_switch_value("remote-debugging-port", "9222");
///
/// // Or use direct initialization (replaces defaults)
/// let config = CommandLineConfig {
///     switches: vec!["disable-gpu"],
///     switch_values: vec![("remote-debugging-port", "9222")],
/// };
/// ```
#[derive(Clone, Debug)]
pub struct CommandLineConfig {
    pub switches: Vec<&'static str>,
    pub switch_values: Vec<(&'static str, &'static str)>,
}

impl Default for CommandLineConfig {
    fn default() -> Self {
        Self {
            switches: vec![
                #[cfg(all(target_os = "macos", debug_assertions))]
                "use-mock-keychain",
                // Without this Chromium tries to launch a zygote process on Linux even
                // with `no_sandbox: true`, which fails with "No such file or directory"
                // in `ZygoteHostImpl` (see issue #9).
                #[cfg(target_os = "linux")]
                "no-zygote",
            ],
            switch_values: Vec::new(),
        }
    }
}

impl CommandLineConfig {
    /// Add a command line switch (e.g., "disable-gpu", "disable-web-security").
    pub fn with_switch(mut self, name: &'static str) -> Self {
        self.switches.push(name);
        self
    }

    /// Add a command line switch with a value (e.g., "remote-debugging-port", "9222").
    pub fn with_switch_value(mut self, name: &'static str, value: &'static str) -> Self {
        self.switch_values.push((name, value));
        self
    }
}

/// Typo-safe constants for common security-relaxing CEF switch names, plus helpers
/// to detect them.
///
/// Use these with [`CommandLineConfig::with_switch`] instead of raw strings:
///
/// ```no_run
/// use bevy_cef_core::prelude::*;
///
/// let config = CommandLineConfig::default().with_switch(switches::DISABLE_WEB_SECURITY);
/// ```
pub mod switches {
    /// Disables the Same-Origin Policy (SOP/CORS). **Insecure** — trusted content only.
    pub const DISABLE_WEB_SECURITY: &str = "disable-web-security";
    /// Allows mixed (HTTP) content on HTTPS pages. **Insecure.**
    pub const ALLOW_RUNNING_INSECURE_CONTENT: &str = "allow-running-insecure-content";
    /// Disables TLS certificate validation. **Insecure** (enables MITM).
    pub const IGNORE_CERTIFICATE_ERRORS: &str = "ignore-certificate-errors";
    /// Disables SSL error handling. **Insecure** (enables MITM).
    pub const IGNORE_SSL_ERRORS: &str = "ignore-ssl-errors";

    /// Switches that relax browser security; their presence triggers a startup warning.
    pub const RISKY_SWITCHES: &[&str] = &[
        DISABLE_WEB_SECURITY,
        ALLOW_RUNNING_INSECURE_CONTENT,
        IGNORE_CERTIFICATE_ERRORS,
        IGNORE_SSL_ERRORS,
    ];

    /// Returns the subset of `switches` present in [`RISKY_SWITCHES`], preserving
    /// order. Names are compared verbatim — normalize a leading `--` first if needed.
    pub fn risky_present<'a>(switches: &[&'a str]) -> Vec<&'a str> {
        switches
            .iter()
            .copied()
            .filter(|s| RISKY_SWITCHES.iter().any(|&r| r == *s))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risky_switches_contains_disable_web_security() {
        assert!(switches::RISKY_SWITCHES.contains(&switches::DISABLE_WEB_SECURITY));
    }

    #[test]
    fn risky_present_filters_to_risky_only() {
        let input = [
            switches::DISABLE_WEB_SECURITY,
            "disable-gpu",
            switches::IGNORE_SSL_ERRORS,
        ];
        assert_eq!(
            switches::risky_present(&input),
            vec![switches::DISABLE_WEB_SECURITY, switches::IGNORE_SSL_ERRORS],
        );
    }

    #[test]
    fn risky_present_empty_when_none_risky() {
        let input = ["disable-gpu", "no-zygote"];
        assert!(switches::risky_present(&input).is_empty());
    }
}
