use tui::widgets::TableState;

pub enum DisplayMode {
    Input,
    Memory { address: u16, state: TableState },
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: String,
    pub displayMode: DisplayMode,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            displayMode: DisplayMode::Input,
        }
    }
}
