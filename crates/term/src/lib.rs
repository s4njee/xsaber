#![forbid(unsafe_code)]

//! Terminal state shared by the desktop shell and headless tools.

/// Minimal terminal model placeholder for the E0 workspace scaffold.
#[derive(Debug, Default)]
pub struct TerminalModel {
    rows: usize,
    columns: usize,
}

impl TerminalModel {
    /// Creates an empty terminal model. Terminal emulation lands in E7.
    pub const fn new() -> Self {
        Self {
            rows: 0,
            columns: 0,
        }
    }

    /// Returns the configured terminal dimensions.
    pub const fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.columns)
    }
}
