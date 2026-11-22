use std::path::PathBuf;
use std::sync::Arc;

use dispel_core::modding::{ChangeOp, Modding};
use hexedit::{SaveOutcome, SavePort, SaveRequest};
use iced::Task;

/// Build the host-side implementation of hexedit's [`SavePort`].
///
/// The editor hands us a domain-level [`SaveRequest`] (path + bytes); we turn
/// it into a mod changelog action through the core [`Modding`] facade and
/// return a [`SaveOutcome`]. No hexedit message types are constructed here —
/// the editor maps the outcome back onto its own internal message. Returns
/// `None` when there's no active recording or game path.
pub fn build_save_callback(
    recording: &Option<crate::state::RecordingSession>,
    game_path: &Option<PathBuf>,
) -> Option<SavePort> {
    let session = recording.as_ref()?;
    let game_path = game_path.clone()?;
    let workspace_root = session.workspace_root.clone();
    let mod_slug = session.mod_slug.clone();

    Some(Arc::new(move |request: SaveRequest| {
        let game_path = game_path.clone();
        let workspace_root = workspace_root.clone();
        let mod_slug = mod_slug.clone();
        let SaveRequest {
            path,
            bytes: current_bytes,
        } = request;

        let Ok(relative) = path.strip_prefix(&game_path) else {
            return Task::done(SaveOutcome::Failed(
                "File is outside the active game directory.".to_string(),
            ));
        };
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || -> Result<String, String> {
                    build_and_append_action(
                        &workspace_root,
                        &game_path,
                        &mod_slug,
                        &relative_str,
                        current_bytes,
                    )
                })
                .await
                .unwrap_or_else(|e| Err(e.to_string()))
            },
            SaveOutcome::from_result,
        )
    }))
}

/// Pure(-ish) helper: open the workspace through the core modding facade,
/// ensure a vanilla snapshot exists, pick the most compact change op and
/// append the resulting [`dispel_core::modding::ChangeAction`].
/// Returns a human-readable summary on success.
fn build_and_append_action(
    workspace_root: &std::path::Path,
    game_dir: &std::path::Path,
    mod_slug: &str,
    relative: &str,
    current_bytes: Vec<u8>,
) -> Result<String, String> {
    let pipeline = Modding::open(workspace_root, game_dir).map_err(|e| e.to_string())?;
    // The facade snapshots vanilla and picks the compact op; no hexedit or
    // package internals leak into this callback.
    let action = pipeline
        .file_change(relative, &current_bytes)
        .map_err(|e| e.to_string())?;
    let summary = match &action.op {
        ChangeOp::BinaryDelta { patch_bytes } => {
            format!(
                "Saved into `{mod_slug}` as BinaryDelta — {} byte patch.",
                patch_bytes.len()
            )
        }
        ChangeOp::FileReplace { content } => {
            format!(
                "Saved into `{mod_slug}` as FileReplace — {} bytes.",
                content.len()
            )
        }
        _ => format!("Saved into `{mod_slug}`."),
    };
    let pruned = pipeline
        .append_reporting(mod_slug, action)
        .map_err(|e| e.to_string())?;
    if pruned > 0 {
        return Ok(format!(
            "{summary} Warning: the mod changelog is full — {pruned} oldest edit(s) were dropped and are not in the mod."
        ));
    }
    Ok(summary)
}

pub use super::recording::decide_op;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_op_picks_binary_delta_for_small_patches() {
        let vanilla = vec![0xABu8; 4096];
        let mut current = vanilla.clone();
        current[0] = 0xCD;
        current[100] = 0xEF;
        let op = decide_op(&vanilla, &current).unwrap();
        assert!(matches!(op, ChangeOp::BinaryDelta { .. }));
    }

    #[test]
    fn decide_op_picks_file_replace_when_files_diverge_heavily() {
        let vanilla: Vec<u8> = (0u8..64).collect();
        let current: Vec<u8> = (0u8..64).map(|b| 255 - b).collect();
        let op = decide_op(&vanilla, &current).unwrap();
        assert!(matches!(op, ChangeOp::FileReplace { .. }));
    }
}
