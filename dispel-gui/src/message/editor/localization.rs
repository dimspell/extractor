use dispel_core::TextEntry;
use iced::widget::text_editor;
use std::path::PathBuf;

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
