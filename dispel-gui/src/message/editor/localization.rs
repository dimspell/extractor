use dispel_core::TextEntry;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum LocalizationMessage {
    Scan,
    Scanned(Result<Vec<TextEntry>, String>),
    TranslationChanged { idx: usize, translation: String },
    FilterFile(Option<String>),
    ToggleUntranslatedOnly,
    ExportCsv,
    ExportPo,
    ImportFile,
    Imported(Result<Vec<TextEntry>, String>),
    ModNameChanged(String),
    ModVersionChanged(String),
    ModAuthorChanged(String),
    ApplyAndPackage,
    Applied(Result<PathBuf, String>),
}
