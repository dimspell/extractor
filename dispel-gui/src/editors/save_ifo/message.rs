//! Messages for the Save.ifo editor.

/// The global tail fields are recorded in the mod changelog under this
/// record id (`Save.ifo` is a single-record file; the hand-written
/// `SaveIfoPatcher` in dispel-core rejects any other id). Slot records
/// themselves are display-only here.
pub const TAIL_RECORD_ID: u32 = 0;

#[derive(Debug, Clone)]
pub enum SaveIfoEditorMessage {
    /// Load `Save.ifo` plus per-slot summaries from the configured game path.
    LoadCatalog,
    CatalogLoaded(Result<(Vec<dispel_core::SlotSummary>, dispel_core::SaveIfo), String>),
    /// A tail field changed: (field path, new string value).
    FieldChanged(String, String),
    /// User asked to swap two slots — opens the confirmation modal.
    SwapRequested(usize, usize),
    /// User confirmed the pending swap (persisted immediately, non-undoable).
    SwapConfirm,
    /// User dismissed the swap confirmation.
    SwapCancel,
    SwapDone(Result<(Vec<dispel_core::SlotSummary>, dispel_core::SaveIfo), String>),
    Save,
    Saved(Result<(), String>),
}
