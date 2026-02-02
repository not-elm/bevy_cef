/// Configuration for CEF command line switches.
///
/// Used to customize CEF behavior at startup.
#[derive(Clone, Default, Debug)]
pub struct CommandLineConfig {
    pub switches: Vec<String>,
    pub switch_values: Vec<(String, String)>,
}
