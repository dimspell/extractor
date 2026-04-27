use dispel_core::TextEntry;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum LocalizationMessage {
    Scan,
    Scanned(Result<Vec<TextEntry>, String>),
    TranslationChanged { idx: usize, translation: String },
    FilterFile(Option<String>),
    ToggleUntranslatedOnly,
    ToggleOverlongOnly,
    ExportCsv,
    ExportPo,
    ExportDone(Result<(), String>),
    ImportFile,
    Imported(Result<Vec<TextEntry>, String>),
    TargetLangChanged(String),
    ModNameChanged(String),
    ModVersionChanged(String),
    ModAuthorChanged(String),
    ApplyAndPackage,
    Applied(Result<PathBuf, String>),
    Revert,
    Reverted(Result<(), String>),
}
