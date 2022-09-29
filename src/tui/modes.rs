use tui::widgets::TableState;

pub enum DisplayMode {
    Input(TableState),
    Memory { address: u16, state: TableState },
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: String,
    pub display_mode: DisplayMode,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            display_mode: DisplayMode::Input(TableState::default()),
        }
    }
}
