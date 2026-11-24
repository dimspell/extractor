use dispel_core::TextEntry;
use iced::widget::text_editor;
use std::path::PathBuf;

/// Outcome of packaging translated strings through the core mod pipeline.
///
/// Carries the core packaging result (`path`), the totals for both kinds of
/// rejected translation, and the per-file truncation warnings used to build
/// the status message, so no information is dropped between the async worker
/// and the UI.
#[derive(Debug, Clone)]
pub struct PackageResult {
    /// Path of the exported mod zip.
    pub path: PathBuf,
    /// Total translations that did not fit their on-disk field and were
    /// truncated by the core `Localizable` implementation.
    pub truncated: usize,
    /// Total translations the core refused to encode in the target file's
    /// encoding. These were not applied at all.
    pub unencodable: usize,
    /// Per-file truncation warnings (e.g. `"Store.db: 2 field(s) truncated"`),
    /// surfaced verbatim in `status_msg`.
    pub warnings: Vec<String>,
    /// Oldest changelog entries dropped when the mod history cap was reached.
    pub pruned: usize,
}

/// Outcome of a CSV/PO import, carried back to the UI from the async worker.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Entries with translations applied.
    pub entries: Vec<TextEntry>,
    /// CSV rows that were skipped because their key was invalid or unmatched;
    /// always zero for PO imports.
    pub skipped: usize,
}

#[derive(Debug, Clone)]
pub enum LocalizationMessage {
    Scan,
    Scanned(Result<Vec<TextEntry>, String>),
    SelectEntry(usize),
    TranslationAction(text_editor::Action),
    SearchChanged(String),
    NavigatePrev,
    NavigateNext,
    PagePrev,
    PageNext,
    FilterFile(Option<String>),
    ToggleUntranslatedOnly,
    ToggleOverlongOnly,
    ExportCsv,
    ExportPo,
    ExportDone(Result<(), String>),
    ImportFile,
    Imported(Result<ImportResult, String>),
    TargetLangChanged(String),
    ModNameChanged(String),
    ModVersionChanged(String),
    ModAuthorChanged(String),
    ApplyAndPackage,
    Applied(Result<PackageResult, String>),
    Revert,
    Reverted(Result<(), String>),
}
