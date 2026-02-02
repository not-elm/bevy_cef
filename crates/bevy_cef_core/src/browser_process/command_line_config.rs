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
