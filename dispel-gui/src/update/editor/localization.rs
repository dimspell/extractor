use crate::app::App;
use crate::loading_state::LoadingState;
use crate::message::editor::localization::LocalizationMessage;
use crate::message::MessageExt;
use dispel_core::localization::Localizable;
use dispel_core::{
    export_csv, export_po, import_csv, import_po, DialogueParagraph, Extractor, Message, Store,
    TextEntry, WeaponItem,
};
use iced::Task;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn handle(message: LocalizationMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        LocalizationMessage::Scan => {
            if app.state.shared_game_path.is_empty() {
                app.state.localization_manager.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            app.state.localization_manager.loading_state = LoadingState::Loading;
            app.state.localization_manager.status_msg = "Scanning…".into();
            let game_path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move { scan_all_entries(&game_path) },
                |result| crate::message::Message::localization(LocalizationMessage::Scanned(result)),
            )
        }
        LocalizationMessage::Scanned(result) => {
            match result {
                Ok(entries) => {
                    let count = entries.len();
                    app.state.localization_manager.entries = entries;
                    app.state.localization_manager.status_msg =
                        format!("{count} strings loaded.");
                    app.state.localization_manager.loading_state = LoadingState::Loaded(());
                }
                Err(e) => {
                    app.state.localization_manager.status_msg = format!("Scan failed: {e}");
                    app.state.localization_manager.loading_state =
                        LoadingState::Failed(e);
                }
            }
            Task::none()
        }
        LocalizationMessage::TranslationChanged { idx, translation } => {
            if let Some(entry) = app.state.localization_manager.entries.get_mut(idx) {
                entry.translation = translation;
                app.state.localization_manager.recompute_truncation();
            }
            Task::none()
        }
        LocalizationMessage::FilterFile(f) => {
            app.state.localization_manager.filter_file = f;
            Task::none()
        }
        LocalizationMessage::ToggleUntranslatedOnly => {
            let v = app.state.localization_manager.show_untranslated_only;
            app.state.localization_manager.show_untranslated_only = !v;
            Task::none()
        }
        LocalizationMessage::ExportCsv => {
            let entries = app.state.localization_manager.entries.clone();
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
                        Ok::<(), String>(())
                    } else {
                        Ok(())
                    }
                },
                |result: Result<(), String>| match result {
                    Ok(()) => crate::message::Message::localization(
                        LocalizationMessage::Scanned(Ok(vec![])),
                    ),
                    Err(e) => crate::message::Message::localization(
                        LocalizationMessage::Scanned(Err(e)),
                    ),
                },
            )
        }
        LocalizationMessage::ExportPo => {
            let entries = app.state.localization_manager.entries.clone();
            Task::perform(
                async move {
                    let po = export_po(&entries, "ko", "");
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
                |result: Result<(), String>| match result {
                    Ok(()) => crate::message::Message::localization(
                        LocalizationMessage::Scanned(Ok(vec![])),
                    ),
                    Err(e) => crate::message::Message::localization(
                        LocalizationMessage::Scanned(Err(e)),
                    ),
                },
            )
        }
        LocalizationMessage::ImportFile => {
            let mut current_entries = app.state.localization_manager.entries.clone();
            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("CSV or PO", &["csv", "po"])
                        .pick_file()
                        .await;
                    let Some(handle) = handle else {
                        return Ok(current_entries);
                    };
                    let path = handle.path().to_path_buf();
                    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    if path.extension().and_then(|e| e.to_str()) == Some("po") {
                        import_po(&content, &mut current_entries);
                    } else {
                        import_csv(&content, &mut current_entries).map_err(|e| e.to_string())?;
                    }
                    Ok(current_entries)
                },
                |result| crate::message::Message::localization(LocalizationMessage::Imported(result)),
            )
        }
        LocalizationMessage::Imported(result) => {
            match result {
                Ok(entries) => {
                    let count = entries.iter().filter(|e| e.is_translated()).count();
                    app.state.localization_manager.entries = entries;
                    app.state.localization_manager.recompute_truncation();
                    app.state.localization_manager.status_msg =
                        format!("Imported. {count} strings translated.");
                }
                Err(e) => {
                    app.state.localization_manager.status_msg = format!("Import failed: {e}");
                }
            }
            Task::none()
        }
        LocalizationMessage::ModNameChanged(v) => {
            app.state.localization_manager.mod_metadata.name = v;
            Task::none()
        }
        LocalizationMessage::ModVersionChanged(v) => {
            app.state.localization_manager.mod_metadata.version = v;
            Task::none()
        }
        LocalizationMessage::ModAuthorChanged(v) => {
            app.state.localization_manager.mod_metadata.author = v;
            Task::none()
        }
        LocalizationMessage::ApplyAndPackage => {
            let state = &mut app.state.localization_manager;
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
            let state = &mut app.state.localization_manager;
            match result {
                Ok(ref path) => {
                    state.status_msg = format!("Done. Mod saved to {}", path.display());
                    state.loading_state = LoadingState::Loaded(());
                }
                Err(ref e) => {
                    state.status_msg = format!("Apply failed: {e}");
                    state.loading_state = LoadingState::Failed(e.clone());
                }
            }
            Task::none()
        }
    }
}

// ─── Scan ────────────────────────────────────────────────────────────────────

fn scan_all_entries(game_path: &Path) -> Result<Vec<TextEntry>, String> {
    let mut entries = Vec::new();

    // Store.db
    let store_path = game_path.join("CharacterInGame").join("STORE.DB");
    if store_path.exists() {
        let records = Store::read_file(&store_path).map_err(|e| e.to_string())?;
        for (i, record) in records.iter().enumerate() {
            entries.extend(record.extract_texts(i, "CharacterInGame/STORE.DB"));
        }
    }

    // weaponItem.db
    let weapon_path = game_path.join("CharacterInGame").join("weaponItem.db");
    if weapon_path.exists() {
        let records = WeaponItem::read_file(&weapon_path).map_err(|e| e.to_string())?;
        for (i, record) in records.iter().enumerate() {
            entries.extend(record.extract_texts(i, "CharacterInGame/weaponItem.db"));
        }
    }

    // Message.scr
    let msg_path = game_path.join("ExtraInGame").join("Message.scr");
    if msg_path.exists() {
        let records = Message::read_file(&msg_path).map_err(|e| e.to_string())?;
        for (i, record) in records.iter().enumerate() {
            entries.extend(record.extract_texts(i, "ExtraInGame/Message.scr"));
        }
    }

    // Dialogue paragraphs — scan all *.pgp files
    scan_pgp_files(game_path, &mut entries)?;

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
            let records =
                DialogueParagraph::read_file(&path).map_err(|e| e.to_string())?;
            for (i, record) in records.iter().enumerate() {
                entries.extend(record.extract_texts(i, &rel));
            }
        }
    }
    Ok(())
}

fn walkdir(root: &Path) -> impl Iterator<Item = Result<PathBuf, std::io::Error>> {
    fn collect(dir: &Path, out: &mut Vec<Result<PathBuf, std::io::Error>>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
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

// ─── Apply & Package ─────────────────────────────────────────────────────────

fn apply_and_package(
    game_path: &Path,
    entries: &[TextEntry],
    meta: &crate::state::mod_packager::ModMetadata,
) -> Result<PathBuf, String> {
    let mod_name = meta.name.trim().replace(' ', "_");
    let backup_dir = game_path.join("mods").join(&mod_name).join("backup");
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    // Group entries by file_path
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

    for (rel_path, file_entries) in &by_file {
        let abs_path = game_path.join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !abs_path.exists() {
            continue;
        }
        // Backup original
        let backup_path = backup_dir.join(
            PathBuf::from(rel_path)
                .file_name()
                .unwrap_or_default(),
        );
        if !backup_path.exists() {
            std::fs::copy(&abs_path, &backup_path).map_err(|e| e.to_string())?;
        }
        // Apply translations
        apply_entries_to_file(&abs_path, rel_path, file_entries)?;
    }

    // Package as zip
    let output_dir = game_path.join("mod_output");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    let zip_path = output_dir.join(format!("{}.zip", mod_name));
    let file = std::fs::File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for rel_path in by_file.keys() {
        let abs_path = game_path.join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if abs_path.exists() {
            let entry_name = PathBuf::from(rel_path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| rel_path.to_string());
            zip.start_file(&entry_name, options).map_err(|e| e.to_string())?;
            let data = std::fs::read(&abs_path).map_err(|e| e.to_string())?;
            zip.write_all(&data).map_err(|e| e.to_string())?;
        }
    }

    let manifest = serde_json::json!({
        "name": meta.name,
        "version": meta.version,
        "author": meta.author,
        "description": meta.description,
        "files": by_file.keys().collect::<Vec<_>>(),
        "backup_dir": backup_dir.to_string_lossy(),
    });
    zip.start_file("manifest.json", options).map_err(|e| e.to_string())?;
    zip.write_all(&serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?;

    Ok(zip_path)
}

fn apply_entries_to_file(
    abs_path: &Path,
    rel_path: &str,
    file_entries: &[&TextEntry],
) -> Result<(), String> {
    // Group by record_id
    let mut by_record: std::collections::HashMap<usize, Vec<&TextEntry>> =
        std::collections::HashMap::new();
    for e in file_entries {
        by_record.entry(e.record_id).or_default().push(e);
    }

    let _sep = std::path::MAIN_SEPARATOR_STR;
    let lower = rel_path.to_lowercase();

    if lower.ends_with("store.db") {
        let mut records = Store::read_file(abs_path).map_err(|e| e.to_string())?;
        for record in &mut records {
            let idx = record.index as usize;
            if let Some(entries) = by_record.get(&idx) {
                let owned: Vec<dispel_core::TextEntry> =
                    entries.iter().map(|e| (*e).clone()).collect();
                record.apply_texts(&owned);
            }
        }
        Store::save_file(&records, abs_path).map_err(|e| e.to_string())?;
    } else if lower.ends_with("weaponitem.db") {
        let mut records = WeaponItem::read_file(abs_path).map_err(|e| e.to_string())?;
        for (i, record) in records.iter_mut().enumerate() {
            if let Some(entries) = by_record.get(&i) {
                let owned: Vec<dispel_core::TextEntry> =
                    entries.iter().map(|e| (*e).clone()).collect();
                record.apply_texts(&owned);
            }
        }
        WeaponItem::save_file(&records, abs_path).map_err(|e| e.to_string())?;
    } else if lower.ends_with("message.scr") {
        let mut records = Message::read_file(abs_path).map_err(|e| e.to_string())?;
        for (i, record) in records.iter_mut().enumerate() {
            if let Some(entries) = by_record.get(&i) {
                let owned: Vec<dispel_core::TextEntry> =
                    entries.iter().map(|e| (*e).clone()).collect();
                record.apply_texts(&owned);
            }
        }
        Message::save_file(&records, abs_path).map_err(|e| e.to_string())?;
    } else if lower.ends_with(".pgp") {
        let mut records = DialogueParagraph::read_file(abs_path).map_err(|e| e.to_string())?;
        for (i, record) in records.iter_mut().enumerate() {
            if let Some(entries) = by_record.get(&i) {
                let owned: Vec<dispel_core::TextEntry> =
                    entries.iter().map(|e| (*e).clone()).collect();
                record.apply_texts(&owned);
            }
        }
        DialogueParagraph::save_file(&records, abs_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
