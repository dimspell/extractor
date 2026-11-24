use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::editors::localization_manager::{ImportResult, LocalizationMessage, PackageResult};
use crate::editors::mod_packager::recording::prune_warning;
use crate::message::MessageExt;
use dispel_core::localization::{Localizable, TruncationStatus};
use dispel_core::modding::{ChangeAction, ChangeOp, ModManifest, Modding};
use dispel_core::{
    DialogueParagraph, EditItem, EventItem, EventNpcRef, ExtraRef, Extractor, HealItem, Map,
    Message, MiscItem, MonsterIni, NPC, PartyIniNpc, PartyRef, Quest, Store, TextEntry, WeaponItem,
    export_csv, export_po, import_csv, import_po,
};
use iced::Task;
use std::path::{Path, PathBuf};

pub fn handle(message: LocalizationMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        LocalizationMessage::Scan => {
            if app.state.shared_game_path.is_empty() {
                app.state.editors.localization_manager.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            app.state.editors.localization_manager.loading_state = LoadingState::Loading;
            app.state.editors.localization_manager.status_msg = "Scanning…".into();
            let game_path = PathBuf::from(&app.state.shared_game_path);
            let session_path = app
                .state
                .editors
                .localization_manager
                .session_path(&app.state.shared_game_path);
            Task::perform(
                async move { scan_all_entries(&game_path, session_path.as_deref()) },
                |result| {
                    crate::message::Message::localization(LocalizationMessage::Scanned(result))
                },
            )
        }
        LocalizationMessage::Scanned(result) => {
            match result {
                Ok(entries) => {
                    let count = entries.len();
                    let translated = entries.iter().filter(|e| e.is_translated()).count();
                    app.state.editors.localization_manager.entries = entries;
                    app.state.editors.localization_manager.status_msg =
                        format!("{count} strings loaded ({translated} already translated).");
                    app.state.editors.localization_manager.loading_state = LoadingState::Loaded(());
                    // Auto-select first entry if nothing selected yet
                    if app
                        .state
                        .editors
                        .localization_manager
                        .selected_idx
                        .is_none()
                        && !app.state.editors.localization_manager.entries.is_empty()
                    {
                        return app.update(crate::message::Message::localization(
                            LocalizationMessage::SelectEntry(0),
                        ));
                    }
                }
                Err(e) => {
                    app.state.editors.localization_manager.status_msg = format!("Scan failed: {e}");
                    app.state.editors.localization_manager.loading_state = LoadingState::Failed(e);
                }
            }
            Task::none()
        }
        LocalizationMessage::SelectEntry(idx) => {
            let text = app
                .state
                .editors
                .localization_manager
                .entries
                .get(idx)
                .map(|e| e.translation.as_str())
                .unwrap_or("");
            app.state.editors.localization_manager.selected_idx = Some(idx);
            app.state.editors.localization_manager.translation_content =
                gui_widgets::TextAreaContent::with_text(text);
            Task::none()
        }
        LocalizationMessage::TranslationAction(action) => {
            use iced::widget::text_editor::Action;
            let is_edit = matches!(action, Action::Edit(_));
            app.state
                .editors
                .localization_manager
                .translation_content
                .0
                .perform(action);
            if is_edit {
                let text = app
                    .state
                    .editors
                    .localization_manager
                    .translation_content
                    .0
                    .text();
                // trim trailing newline that text_editor appends
                let text = text.trim_end_matches('\n').to_owned();
                if let Some(idx) = app.state.editors.localization_manager.selected_idx
                    && let Some(entry) = app.state.editors.localization_manager.entries.get_mut(idx)
                {
                    entry.translation = text;
                }
                // Debounced session save
                let game_path = app.state.shared_game_path.clone();
                let session_path = app
                    .state
                    .editors
                    .localization_manager
                    .session_path(&game_path);
                if let Some(path) = session_path {
                    let entries = app.state.editors.localization_manager.entries.clone();
                    return Task::perform(async move { save_session(&path, &entries) }, |_| {
                        crate::message::Message::localization(LocalizationMessage::ExportDone(Ok(
                            (),
                        )))
                    });
                }
            }
            Task::none()
        }
        LocalizationMessage::SearchChanged(q) => {
            app.state.editors.localization_manager.search_query = q;
            app.state.editors.localization_manager.page = 0;
            Task::none()
        }
        LocalizationMessage::NavigatePrev => {
            let state = &app.state.editors.localization_manager;
            let from = state.selected_idx.unwrap_or(0);
            if let Some(idx) = state.prev_untranslated(from) {
                return app.update(crate::message::Message::localization(
                    LocalizationMessage::SelectEntry(idx),
                ));
            }
            Task::none()
        }
        LocalizationMessage::NavigateNext => {
            let state = &app.state.editors.localization_manager;
            let from = state
                .selected_idx
                .unwrap_or(state.entries.len().saturating_sub(1));
            if let Some(idx) = state.next_untranslated(from) {
                return app.update(crate::message::Message::localization(
                    LocalizationMessage::SelectEntry(idx),
                ));
            }
            Task::none()
        }
        LocalizationMessage::FilterFile(f) => {
            app.state.editors.localization_manager.filter_file = f;
            app.state.editors.localization_manager.page = 0;
            Task::none()
        }
        LocalizationMessage::ToggleUntranslatedOnly => {
            let v = app
                .state
                .editors
                .localization_manager
                .show_untranslated_only;
            app.state
                .editors
                .localization_manager
                .show_untranslated_only = !v;
            app.state.editors.localization_manager.page = 0;
            Task::none()
        }
        LocalizationMessage::ToggleOverlongOnly => {
            let v = app.state.editors.localization_manager.show_overlong_only;
            app.state.editors.localization_manager.show_overlong_only = !v;
            app.state.editors.localization_manager.page = 0;
            Task::none()
        }
        LocalizationMessage::PagePrev => {
            if app.state.editors.localization_manager.page > 0 {
                app.state.editors.localization_manager.page -= 1;
            }
            Task::none()
        }
        LocalizationMessage::PageNext => {
            let visible_len = app
                .state
                .editors
                .localization_manager
                .visible_entries()
                .len();
            let max_page = visible_len.saturating_sub(1) / 250;
            if app.state.editors.localization_manager.page < max_page {
                app.state.editors.localization_manager.page += 1;
            }
            Task::none()
        }
        LocalizationMessage::TargetLangChanged(v) => {
            app.state.editors.localization_manager.target_lang = v;
            Task::none()
        }
        LocalizationMessage::ExportCsv => {
            let entries = app.state.editors.localization_manager.entries.clone();
            Task::perform(
                async move {
                    let csv = export_csv(&entries).map_err(|e| e.to_string())?;
                    let path = rfd::AsyncFileDialog::new()
                        .set_file_name("localization.csv")
                        .add_filter("CSV", &["csv"])
                        .save_file()
                        .await
                        .map(|h| h.path().to_path_buf());
                    if let Some(p) = path {
                        std::fs::write(&p, csv.as_bytes()).map_err(|e| e.to_string())?;
                    }
                    Ok::<(), String>(())
                },
                |result| {
                    crate::message::Message::localization(LocalizationMessage::ExportDone(result))
                },
            )
        }
        LocalizationMessage::ExportPo => {
            let entries = app.state.editors.localization_manager.entries.clone();
            let target_lang = app.state.editors.localization_manager.target_lang.clone();
            Task::perform(
                async move {
                    let po = export_po(&entries, "ko", &target_lang);
                    let path = rfd::AsyncFileDialog::new()
                        .set_file_name("localization.po")
                        .add_filter("PO file", &["po"])
                        .save_file()
                        .await
                        .map(|h| h.path().to_path_buf());
                    if let Some(p) = path {
                        std::fs::write(&p, po.as_bytes()).map_err(|e| e.to_string())?;
                    }
                    Ok::<(), String>(())
                },
                |result| {
                    crate::message::Message::localization(LocalizationMessage::ExportDone(result))
                },
            )
        }
        LocalizationMessage::ExportDone(result) => {
            if let Err(e) = result {
                app.state.editors.localization_manager.status_msg = format!("Export failed: {e}");
            }
            Task::none()
        }
        LocalizationMessage::ImportFile => {
            let mut current_entries = app.state.editors.localization_manager.entries.clone();
            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("CSV or PO", &["csv", "po"])
                        .pick_file()
                        .await;
                    let Some(handle) = handle else {
                        return Ok(ImportResult {
                            entries: current_entries,
                            skipped: 0,
                        });
                    };
                    let path = handle.path().to_path_buf();
                    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    let skipped = if path.extension().and_then(|e| e.to_str()) == Some("po") {
                        import_po(&content, &mut current_entries);
                        0
                    } else {
                        import_csv(&content, &mut current_entries)
                            .map_err(|e| e.to_string())?
                            .skipped
                    };
                    Ok(ImportResult {
                        entries: current_entries,
                        skipped,
                    })
                },
                |result| {
                    crate::message::Message::localization(LocalizationMessage::Imported(result))
                },
            )
        }
        LocalizationMessage::Imported(result) => {
            match result {
                Ok(import) => {
                    let ImportResult { entries, skipped } = import;
                    let count = entries.iter().filter(|e| e.is_translated()).count();
                    let overlong = entries.iter().filter(|e| e.would_truncate()).count();
                    app.state.editors.localization_manager.entries = entries;
                    let mut msg = if overlong > 0 {
                        format!("Imported. {count} strings translated, {overlong} overlong.")
                    } else {
                        format!("Imported. {count} strings translated.")
                    };
                    if skipped > 0 {
                        msg.push_str(&format!(" {skipped} row(s) skipped."));
                    }
                    app.state.editors.localization_manager.status_msg = msg;
                    // Refresh editor panel if selected entry changed
                    if let Some(idx) = app.state.editors.localization_manager.selected_idx
                        && let Some(entry) = app.state.editors.localization_manager.entries.get(idx)
                    {
                        let text = entry.translation.clone();
                        app.state.editors.localization_manager.translation_content =
                            gui_widgets::TextAreaContent::with_text(&text);
                    }
                    // Persist session
                    let game_path = app.state.shared_game_path.clone();
                    if let Some(path) = app
                        .state
                        .editors
                        .localization_manager
                        .session_path(&game_path)
                    {
                        let entries = app.state.editors.localization_manager.entries.clone();
                        return Task::perform(async move { save_session(&path, &entries) }, |_| {
                            crate::message::Message::localization(LocalizationMessage::ExportDone(
                                Ok(()),
                            ))
                        });
                    }
                }
                Err(e) => {
                    app.state.editors.localization_manager.status_msg =
                        format!("Import failed: {e}");
                }
            }
            Task::none()
        }
        LocalizationMessage::ModNameChanged(v) => {
            app.state.editors.localization_manager.mod_metadata.name = v;
            Task::none()
        }
        LocalizationMessage::ModVersionChanged(v) => {
            app.state.editors.localization_manager.mod_metadata.version = v;
            Task::none()
        }
        LocalizationMessage::ModAuthorChanged(v) => {
            app.state.editors.localization_manager.mod_metadata.author = v;
            Task::none()
        }
        LocalizationMessage::ApplyAndPackage => {
            let state = &mut app.state.editors.localization_manager;
            if state.entries.is_empty() {
                state.status_msg = "Nothing to apply — scan first.".into();
                return Task::none();
            }
            if state.mod_metadata.name.trim().is_empty() {
                state.status_msg = "Mod name is required.".into();
                return Task::none();
            }
            state.loading_state = LoadingState::Loading;
            state.status_msg = "Applying translations…".into();
            let entries = state.entries.clone();
            let meta = state.mod_metadata.clone();
            let game_path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move { apply_and_package(&game_path, &entries, &meta) },
                |result| {
                    crate::message::Message::localization(LocalizationMessage::Applied(result))
                },
            )
        }
        LocalizationMessage::Applied(result) => {
            let state = &mut app.state.editors.localization_manager;
            match result {
                Ok(ref packaged) => {
                    // Surface the per-file warnings plus the totals for both
                    // kinds of rejected translation (truncated and unencodable).
                    let mut msg = applied_status_msg(packaged);
                    if let Some(warning) = prune_warning(packaged.pruned) {
                        msg.push(' ');
                        msg.push_str(&warning);
                    }
                    state.status_msg = msg;
                    state.loading_state = LoadingState::Loaded(());
                }
                Err(ref e) => {
                    state.status_msg = format!("Apply failed: {e}");
                    state.loading_state = LoadingState::Failed(e.clone());
                }
            }
            Task::none()
        }
        LocalizationMessage::Revert => {
            let state = &app.state.editors.localization_manager;
            let game_path = app.state.shared_game_path.clone();
            let backup_dir = state.backup_dir(&game_path);
            let Some(backup_dir) = backup_dir else {
                app.state.editors.localization_manager.status_msg = "No mod name set.".into();
                return Task::none();
            };
            if !backup_dir.exists() {
                app.state.editors.localization_manager.status_msg =
                    "No backup found to revert.".into();
                return Task::none();
            }
            let game_path = PathBuf::from(&game_path);
            Task::perform(
                async move { revert_from_backup(&game_path, &backup_dir) },
                |result| {
                    crate::message::Message::localization(LocalizationMessage::Reverted(result))
                },
            )
        }
        LocalizationMessage::Reverted(result) => {
            let state = &mut app.state.editors.localization_manager;
            match result {
                Ok(()) => {
                    state.status_msg = "Reverted: original files restored from backup.".into();
                    state.loading_state = LoadingState::Idle;
                }
                Err(e) => {
                    state.status_msg = format!("Revert failed: {e}");
                }
            }
            Task::none()
        }
    }
}

// ─── Session persistence ──────────────────────────────────────────────────────

/// Lightweight session record — fully owned so it serializes/deserializes without lifetime issues.
/// `TextEntry.field_name` is `&'static str` and cannot be deserialized from JSON directly.
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedTranslation {
    file_path: String,
    record_id: usize,
    field_name: String,
    translation: String,
}

fn save_session(path: &Path, entries: &[TextEntry]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let saved: Vec<SavedTranslation> = entries
        .iter()
        .filter(|e| e.is_translated())
        .map(|e| SavedTranslation {
            file_path: e.file_path.clone(),
            record_id: e.record_id,
            field_name: e.field_name.to_owned(),
            translation: e.translation.clone(),
        })
        .collect();
    let json = serde_json::to_string(&saved).map_err(|e| e.to_string())?;
    std::fs::write(path, json.as_bytes()).map_err(|e| e.to_string())
}

fn load_session(path: &Path) -> Option<Vec<SavedTranslation>> {
    let s = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&s).ok()
}

/// Merge saved translations into freshly-scanned entries.
/// Matches by (file_path, record_id, field_name).
fn merge_session(entries: &mut [TextEntry], saved: &[SavedTranslation]) {
    use std::collections::HashMap;
    let saved_map: HashMap<(&str, usize, &str), &str> = saved
        .iter()
        .map(|e| {
            (
                (e.file_path.as_str(), e.record_id, e.field_name.as_str()),
                e.translation.as_str(),
            )
        })
        .collect();
    for entry in entries.iter_mut() {
        if let Some(&t) =
            saved_map.get(&(entry.file_path.as_str(), entry.record_id, entry.field_name))
            && !t.is_empty()
        {
            entry.translation = t.to_owned();
        }
    }
}

// ─── Scan ─────────────────────────────────────────────────────────────────────

fn scan_one<T: Extractor + Localizable>(
    game_path: &Path,
    rel: &str,
    entries: &mut Vec<TextEntry>,
) -> Result<(), String> {
    let abs = game_path.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !abs.exists() {
        return Ok(());
    }
    let records = T::read_file(&abs).map_err(|e| e.to_string())?;
    for (i, record) in records.iter().enumerate() {
        entries.extend(record.extract_texts(i, rel));
    }
    Ok(())
}

const NPC_REF_FILES: &[&str] = &[
    "NpcInGame/Npccat1.ref",
    "NpcInGame/Npccat2.ref",
    "NpcInGame/Npccat3.ref",
    "NpcInGame/Npccatp.ref",
    "NpcInGame/npcdun08.ref",
    "NpcInGame/npcdun19.ref",
    "NpcInGame/Npcmap1.ref",
    "NpcInGame/Npcmap2.ref",
    "NpcInGame/Npcmap3.ref",
];

const EXTRA_REF_FILES: &[&str] = &[
    "ExtraInGame/Extcat3.ref",
    "ExtraInGame/Extdun01.ref",
    "ExtraInGame/Extdun02.ref",
    "ExtraInGame/Extdun03.ref",
    "ExtraInGame/Extdun04.ref",
    "ExtraInGame/Extdun05.ref",
    "ExtraInGame/Extdun06.ref",
    "ExtraInGame/Extdun07.ref",
    "ExtraInGame/Extdun08.ref",
    "ExtraInGame/Extdun09.ref",
    "ExtraInGame/Extdun10.ref",
    "ExtraInGame/Extdun11.ref",
    "ExtraInGame/Extdun12.ref",
    "ExtraInGame/Extdun13.ref",
    "ExtraInGame/Extdun14.ref",
    "ExtraInGame/Extdun15.ref",
    "ExtraInGame/Extdun16.ref",
    "ExtraInGame/Extdun17.ref",
    "ExtraInGame/Extdun18.ref",
    "ExtraInGame/Extdun19.ref",
    "ExtraInGame/Extdun20.ref",
    "ExtraInGame/Extdun21.ref",
    "ExtraInGame/Extdun22.ref",
    "ExtraInGame/Extdun23.ref",
    "ExtraInGame/Extdun24.ref",
    "ExtraInGame/Extdun25.ref",
    "ExtraInGame/Extfinal.ref",
    "ExtraInGame/Extmap1.ref",
    "ExtraInGame/Extmap2.ref",
    "ExtraInGame/Extmap3.ref",
];

fn scan_all_entries(
    game_path: &Path,
    session_path: Option<&Path>,
) -> Result<Vec<TextEntry>, String> {
    let mut entries = Vec::new();

    // Store.db — uses record.index as logical ID, not position
    let store_path = game_path.join("CharacterInGame").join("STORE.DB");
    if store_path.exists() {
        let records = Store::read_file(&store_path).map_err(|e| e.to_string())?;
        for (i, record) in records.iter().enumerate() {
            entries.extend(record.extract_texts(i, "CharacterInGame/STORE.DB"));
        }
    }

    scan_one::<WeaponItem>(game_path, "CharacterInGame/weaponItem.db", &mut entries)?;
    scan_one::<HealItem>(game_path, "CharacterInGame/HealItem.db", &mut entries)?;
    scan_one::<EditItem>(game_path, "CharacterInGame/EditItem.db", &mut entries)?;
    scan_one::<EventItem>(game_path, "CharacterInGame/EventItem.db", &mut entries)?;
    scan_one::<MiscItem>(game_path, "CharacterInGame/MiscItem.db", &mut entries)?;
    scan_one::<Message>(game_path, "ExtraInGame/Message.scr", &mut entries)?;
    scan_one::<PartyIniNpc>(game_path, "NpcInGame/PrtIni.db", &mut entries)?;
    scan_one::<EventNpcRef>(game_path, "NpcInGame/Eventnpc.ref", &mut entries)?;
    scan_one::<Quest>(game_path, "ExtraInGame/Quest.scr", &mut entries)?;
    scan_one::<Map>(game_path, "AllMap.ini", &mut entries)?;
    scan_one::<MonsterIni>(game_path, "Monster.ini", &mut entries)?;
    scan_one::<PartyRef>(game_path, "Ref/PartyRef.ref", &mut entries)?;

    for rel in NPC_REF_FILES {
        scan_one::<NPC>(game_path, rel, &mut entries)?;
    }
    for rel in EXTRA_REF_FILES {
        scan_one::<ExtraRef>(game_path, rel, &mut entries)?;
    }

    // Dialogue paragraphs — scan all *.pgp files
    scan_pgp_files(game_path, &mut entries)?;

    // Merge saved session translations on top of fresh scan
    if let Some(path) = session_path
        && let Some(saved) = load_session(path)
    {
        merge_session(&mut entries, &saved);
    }

    Ok(entries)
}

fn scan_pgp_files(game_path: &Path, entries: &mut Vec<TextEntry>) -> Result<(), String> {
    for entry in walkdir(game_path) {
        let path = entry.map_err(|e| e.to_string())?;
        if path.extension().and_then(|e| e.to_str()) == Some("pgp") {
            let rel = path
                .strip_prefix(game_path)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let records = DialogueParagraph::read_file(&path).map_err(|e| e.to_string())?;
            for (i, record) in records.iter().enumerate() {
                entries.extend(record.extract_texts(i, &rel));
            }
        }
    }
    Ok(())
}

fn walkdir(root: &Path) -> impl Iterator<Item = Result<PathBuf, std::io::Error>> {
    fn collect(dir: &Path, out: &mut Vec<Result<PathBuf, std::io::Error>>) {
        let rd = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) => {
                eprintln!(
                    "Localization scan: cannot read directory {}: {}",
                    dir.display(),
                    e
                );
                return;
            }
        };
        for entry in rd {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Localization scan: skipping unreadable entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            if path.is_dir() {
                collect(&path, out);
            } else {
                out.push(Ok(path));
            }
        }
    }
    let mut results = Vec::new();
    collect(root, &mut results);
    results.into_iter()
}

// ─── Apply & Package ──────────────────────────────────────────────────────────

/// Apply translated strings to the game files, then package the result through
/// the core modding facade so the produced zip is the same
/// `manifest.json` / `changes.json` / `files` layout the Mod Manager imports —
/// no second, incompatible zip/manifest pipeline here. Truncation is reported
/// as a per-file warning list plus a total; unencodable translations are
/// reported as a separate total in the status message.
fn apply_and_package(
    game_path: &Path,
    entries: &[TextEntry],
    meta: &crate::editors::mod_packager::state::ModMetadata,
) -> Result<PackageResult, String> {
    let mod_name = meta.name.trim().replace(' ', "_");
    let backup_dir = game_path.join("mods").join(&mod_name).join("backup");
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    // Group translated entries by file_path
    let mut by_file: std::collections::HashMap<&str, Vec<&TextEntry>> =
        std::collections::HashMap::new();
    for e in entries {
        if e.is_translated() {
            by_file.entry(&e.file_path).or_default().push(e);
        }
    }

    if by_file.is_empty() {
        return Err("No translated strings to apply.".into());
    }

    let mut warnings: Vec<String> = Vec::new();
    let mut truncated = 0usize;
    let mut unencodable = 0usize;
    for (rel_path, file_entries) in &by_file {
        let abs_path = game_path.join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !abs_path.exists() {
            continue;
        }
        // Backup original (preserve relative path structure inside backup dir)
        let rel_pb = PathBuf::from(rel_path);
        let backup_path = if let Some(parent) = rel_pb.parent() {
            let d = backup_dir.join(parent);
            std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
            d.join(rel_pb.file_name().unwrap_or_default())
        } else {
            backup_dir.join(rel_pb.file_name().unwrap_or_default())
        };
        if !backup_path.exists() {
            std::fs::copy(&abs_path, &backup_path).map_err(|e| e.to_string())?;
        }
        let (file_truncated, file_unencodable) =
            apply_entries_to_file(&abs_path, rel_path, file_entries)?;
        truncated += file_truncated;
        unencodable += file_unencodable;
        if file_truncated > 0 {
            warnings.push(format!("{rel_path}: {file_truncated} field(s) truncated"));
        }
    }

    // Route packaging through the single core modding facade. The resulting
    // zip has the same manifest.json / changes.json / files layout the Mod
    // Manager imports — no second, incompatible zip/manifest pipeline here.
    let workspace_root = game_path.join(".dispel-mods");
    let pipeline = Modding::open(&workspace_root, game_path).map_err(|e| e.to_string())?;
    let mut pruned = 0;
    let mut manifest = ModManifest::new(meta.name.trim());
    manifest.version = meta.version.clone();
    manifest.author = meta.author.clone();
    manifest.description = meta.description.clone();
    let slug = pipeline.create_mod(manifest).map_err(|e| e.to_string())?;

    for rel_path in by_file.keys() {
        let abs_path = game_path.join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !abs_path.exists() {
            continue;
        }
        let content = std::fs::read(&abs_path).map_err(|e| e.to_string())?;
        let action = ChangeAction::new(*rel_path, ChangeOp::FileReplace { content });
        pruned += pipeline
            .append_reporting(&slug, action)
            .map_err(|e| e.to_string())?;
    }

    let output_dir = game_path.join("mod_output");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    let zip_path = output_dir.join(format!("{mod_name}.zip"));
    pipeline
        .export_zip(&slug, &zip_path)
        .map_err(|e| e.to_string())?;

    Ok(PackageResult {
        path: zip_path,
        truncated,
        unencodable,
        warnings,
        pruned,
    })
}

/// Status text shown after packaging. Truncation and encoding-rejection counts
/// are appended so a shortened or refused translation is never reported as a
/// clean success.
fn applied_status_msg(packaged: &PackageResult) -> String {
    let mut msg = format!("Done. Mod saved to {}", packaged.path.display());
    if !packaged.warnings.is_empty() {
        msg.push_str(&format!(
            " WARNING: {} — translations were truncated to the game's byte limits; check the Overlong filter.",
            packaged.warnings.join("; ")
        ));
    }
    if packaged.truncated > 0 {
        msg.push_str(&format!(
            " — {} translation(s) were truncated to fit their field.",
            packaged.truncated
        ));
    }
    if packaged.unencodable > 0 {
        msg.push_str(&format!(
            " — {} translation(s) were NOT applied because they could not be encoded in the target file's encoding.",
            packaged.unencodable
        ));
    }
    msg
}

/// Count `(truncated, unencodable)` results from applying translated texts.
/// `Truncated` fields were silently shortened to the game's fixed byte width;
/// `Unencodable` translations were refused outright by the core `Localizable`
/// implementation. Both are surfaced in the packaging status rather than
/// discarded.
fn count_rejected(statuses: &[TruncationStatus]) -> (usize, usize) {
    let truncated = statuses
        .iter()
        .filter(|s| matches!(s, TruncationStatus::Truncated { .. }))
        .count();
    let unencodable = statuses
        .iter()
        .filter(|s| matches!(s, TruncationStatus::Unencodable))
        .count();
    (truncated, unencodable)
}

// A2: generic helper eliminates repeated read→apply→save pattern
fn apply_one<T: Extractor + Localizable>(
    abs_path: &Path,
    by_record: &std::collections::HashMap<usize, Vec<&TextEntry>>,
) -> Result<(usize, usize), String> {
    let mut records = T::read_file(abs_path).map_err(|e| e.to_string())?;
    let mut truncated = 0;
    let mut unencodable = 0;
    for (i, record) in records.iter_mut().enumerate() {
        if let Some(entries) = by_record.get(&i) {
            let owned: Vec<TextEntry> = entries.iter().map(|e| (*e).clone()).collect();
            let (t, u) = count_rejected(&record.apply_texts(&owned));
            truncated += t;
            unencodable += u;
        }
    }
    T::save_file(&records, abs_path).map_err(|e| e.to_string())?;
    Ok((truncated, unencodable))
}

fn apply_entries_to_file(
    abs_path: &Path,
    rel_path: &str,
    file_entries: &[&TextEntry],
) -> Result<(usize, usize), String> {
    let mut by_record: std::collections::HashMap<usize, Vec<&TextEntry>> =
        std::collections::HashMap::new();
    for e in file_entries {
        by_record.entry(e.record_id).or_default().push(e);
    }

    let lower = rel_path.to_lowercase();

    if lower.ends_with("store.db") {
        // Store uses record.index as the logical ID, not position
        let mut records = Store::read_file(abs_path).map_err(|e| e.to_string())?;
        let mut truncated = 0;
        let mut unencodable = 0;
        for record in &mut records {
            let idx = record.index as usize;
            if let Some(entries) = by_record.get(&idx) {
                let owned: Vec<TextEntry> = entries.iter().map(|e| (*e).clone()).collect();
                let (t, u) = count_rejected(&record.apply_texts(&owned));
                truncated += t;
                unencodable += u;
            }
        }
        Store::save_file(&records, abs_path).map_err(|e| e.to_string())?;
        Ok((truncated, unencodable))
    } else if lower.ends_with("weaponitem.db") {
        apply_one::<WeaponItem>(abs_path, &by_record)
    } else if lower.ends_with("healitem.db") {
        apply_one::<HealItem>(abs_path, &by_record)
    } else if lower.ends_with("edititem.db") {
        apply_one::<EditItem>(abs_path, &by_record)
    } else if lower.ends_with("eventitem.db") {
        apply_one::<EventItem>(abs_path, &by_record)
    } else if lower.ends_with("miscitem.db") {
        apply_one::<MiscItem>(abs_path, &by_record)
    } else if lower.ends_with("message.scr") {
        apply_one::<Message>(abs_path, &by_record)
    } else if lower.ends_with("prtini.db") {
        apply_one::<PartyIniNpc>(abs_path, &by_record)
    } else if lower.ends_with("eventnpc.ref") {
        apply_one::<EventNpcRef>(abs_path, &by_record)
    } else if lower.contains("npcingame/") && lower.ends_with(".ref") {
        apply_one::<NPC>(abs_path, &by_record)
    } else if lower.contains("extraingame/") && lower.ends_with(".ref") {
        apply_one::<ExtraRef>(abs_path, &by_record)
    } else if lower.ends_with(".pgp") {
        apply_one::<DialogueParagraph>(abs_path, &by_record)
    } else if lower.ends_with("quest.scr") {
        apply_one::<Quest>(abs_path, &by_record)
    } else if lower.ends_with("allmap.ini") {
        apply_one::<Map>(abs_path, &by_record)
    } else if lower.ends_with("monster.ini") {
        apply_one::<MonsterIni>(abs_path, &by_record)
    } else if lower.ends_with("partyref.ref") {
        apply_one::<PartyRef>(abs_path, &by_record)
    } else {
        Ok((0, 0))
    }
}

// ─── Revert ───────────────────────────────────────────────────────────────────

fn revert_from_backup(game_path: &Path, backup_dir: &Path) -> Result<(), String> {
    // Walk backup dir; for each file, restore to game_path keeping relative structure
    for entry in walkdir(backup_dir) {
        let src = entry.map_err(|e| e.to_string())?;
        let rel = src
            .strip_prefix(backup_dir)
            .unwrap_or(&src)
            .to_string_lossy()
            .replace('\\', "/");
        let dest = game_path.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&src, &dest)
            .map_err(|e| format!("Failed to restore {}: {e}", dest.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packaged(
        path: &str,
        warnings: &[&str],
        truncated: usize,
        unencodable: usize,
    ) -> PackageResult {
        PackageResult {
            path: PathBuf::from(path),
            truncated,
            unencodable,
            warnings: warnings.iter().map(|s| s.to_string()).collect(),
            pruned: 0,
        }
    }

    #[test]
    fn test_count_rejected_ignores_ok_statuses() {
        assert_eq!(count_rejected(&[]), (0, 0));
        assert_eq!(
            count_rejected(&[TruncationStatus::Ok, TruncationStatus::Ok]),
            (0, 0)
        );
    }

    #[test]
    fn test_count_rejected_counts_truncated_and_unencodable_statuses() {
        let statuses = [
            TruncationStatus::Ok,
            TruncationStatus::Truncated {
                original_bytes: 128,
            },
            TruncationStatus::Truncated { original_bytes: 20 },
            TruncationStatus::Unencodable,
            TruncationStatus::Unencodable,
            TruncationStatus::Unencodable,
        ];
        assert_eq!(count_rejected(&statuses), (2, 3));
    }

    #[test]
    fn test_applied_status_msg_without_warnings_reports_success_only() {
        let msg = applied_status_msg(&packaged("mod_output/my_mod.zip", &[], 0, 0));
        assert_eq!(msg, "Done. Mod saved to mod_output/my_mod.zip");
    }

    #[test]
    fn test_applied_status_msg_with_warnings_surfaces_truncation() {
        let msg = applied_status_msg(&packaged(
            "mod_output/my_mod.zip",
            &["Store.db: 2 field(s) truncated"],
            2,
            0,
        ));
        assert!(msg.starts_with("Done. Mod saved to mod_output/my_mod.zip"));
        assert!(msg.contains("WARNING"));
        assert!(msg.contains("Store.db: 2 field(s) truncated"));
        assert!(msg.contains("Overlong filter"));
        assert!(msg.contains("2 translation(s) were truncated to fit their field."));
        assert!(!msg.contains("NOT applied"));
    }

    #[test]
    fn test_applied_status_msg_surfaces_unencodable_distinct_from_truncated() {
        let msg = applied_status_msg(&packaged("mod_output/my_mod.zip", &[], 0, 3));
        assert!(msg.starts_with("Done. Mod saved to mod_output/my_mod.zip"));
        assert!(msg.contains("3 translation(s) were NOT applied"));
        assert!(msg.contains("could not be encoded"));
        assert!(!msg.contains("were truncated"));
    }

    #[test]
    fn test_applied_status_msg_surfaces_both_rejection_kinds() {
        let msg = applied_status_msg(&packaged("out.zip", &[], 2, 4));
        assert!(msg.contains("2 translation(s) were truncated to fit their field."));
        assert!(msg.contains("4 translation(s) were NOT applied"));
    }

    #[test]
    fn test_applied_status_msg_joins_multiple_file_warnings() {
        let msg = applied_status_msg(&packaged(
            "out.zip",
            &[
                "Store.db: 1 field(s) truncated",
                "WeaponItem.db: 3 field(s) truncated",
            ],
            4,
            0,
        ));
        assert!(
            msg.contains("Store.db: 1 field(s) truncated; WeaponItem.db: 3 field(s) truncated")
        );
    }
}
