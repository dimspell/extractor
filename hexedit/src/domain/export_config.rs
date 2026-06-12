//! Configuration for the export-to-text feature.
//!
//! Captures user choices (address gutter, address format, ASCII column)
//! set in the export config modal before the file-dialog is opened.

/// User-configurable options for the hex dump text export.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Show the address column on the left of each row.
    pub show_address: bool,
    /// If `show_address`, use decimal instead of hex.
    pub address_decimal: bool,
    /// Show the ASCII representation column on the right.
    pub show_ascii: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            show_address: true,
            address_decimal: false,
            show_ascii: true,
        }
    }
}
