/// Messages for the `ExtraInGame/fogdata.dat` editor.
///
/// Every variant carries the owning `tab_id` so handlers always mutate the
/// correct per-tab state, even when several fog tabs are open at once.
#[derive(Debug, Clone)]
pub enum FogDataMessage {
    /// Select a light level (1-based, `1..=123`) in a tab.
    LevelSelected(usize, u32),
    /// Select a pixel-pair index (`0..512`) in a tab.
    PairSelected(usize, usize),
    /// Live paint while dragging across the curve canvas.
    /// `(tab_id, pair, quantized factor value)`
    FactorPainted(usize, usize, u8),
    /// A paint stroke finished (mouse released) — commits the undo snapshot
    /// taken at stroke start, if anything actually changed.
    StrokeEnded(usize),
    /// Commit a single factor edit from the inspector field / steppers.
    /// Pushes an undo snapshot before applying. `(tab_id, pair, value)`
    FactorCommitted(usize, usize, u8),
    /// The numeric edit field changed — validated live, committed on submit.
    ValueInputChanged(usize, String),
    /// Enter pressed in the numeric field — commit if valid.
    ValueSubmitted(usize),
    /// Save the fade tables back to disk (async).
    Save(usize),
    /// Save finished. `(tab_id, save generation the run was based on, result)`
    /// The completion only clears `dirty` when its generation still matches
    /// the editor's current one (no edit landed while saving). `Ok` carries
    /// `(status, optional mod-recording failure)` — the file was written.
    SaveComplete(usize, u64, Result<(String, Option<String>), String>),
    /// Reload from disk; asks for confirmation when dirty.
    Revert(usize),
    /// User confirmed discarding unsaved edits.
    RevertConfirmed(usize),
    /// User cancelled the revert confirmation.
    RevertCancelled(usize),
    /// Undo the last paint stroke / committed edit.
    Undo(usize),
    /// Redo the last undone edit.
    Redo(usize),
}
