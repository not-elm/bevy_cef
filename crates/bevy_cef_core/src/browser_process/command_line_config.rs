/// Configuration for CEF command line switches.
///
/// Used to customize CEF behavior at startup.
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
            ],
            switch_values: Vec::new(),
        }
    }
}
