use crate::app::App;
use crate::loading_state::LoadingState;
use crate::message::editor::mod_packager::ModPackagerMessage;
use crate::message::MessageExt;
use crate::state::mod_packager::ModMetadata;
use iced::Task;
use std::io::Write;
use std::path::PathBuf;

pub fn handle(message: ModPackagerMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        ModPackagerMessage::BrowseFiles => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_files()
                    .await
                    .map(|handles| handles.iter().map(|h| h.path().to_path_buf()).collect())
                    .unwrap_or_default()
            },
            |paths: Vec<PathBuf>| {
                crate::message::Message::mod_packager(ModPackagerMessage::FilesChosen(paths))
            },
        ),
        ModPackagerMessage::FilesChosen(paths) => {
            let state = &mut app.state.mod_packager_editor;
            for path in paths {
                if !state.selected_files.contains(&path) {
                    state.selected_files.push(path);
                }
            }
            state.status_msg = format!("{} file(s) selected", state.selected_files.len());
            Task::none()
        }
        ModPackagerMessage::AddFile(path) => {
            let state = &mut app.state.mod_packager_editor;
            if !state.selected_files.contains(&path) {
                state.selected_files.push(path);
                state.status_msg = format!("{} file(s) selected", state.selected_files.len());
            }
            Task::none()
        }
        ModPackagerMessage::RemoveFile(idx) => {
            let state = &mut app.state.mod_packager_editor;
            if idx < state.selected_files.len() {
                state.selected_files.remove(idx);
                state.status_msg = format!("{} file(s) selected", state.selected_files.len());
            }
            Task::none()
        }
        ModPackagerMessage::NameChanged(v) => {
            app.state.mod_packager_editor.metadata.name = v;
            Task::none()
        }
        ModPackagerMessage::VersionChanged(v) => {
            app.state.mod_packager_editor.metadata.version = v;
            Task::none()
        }
        ModPackagerMessage::AuthorChanged(v) => {
            app.state.mod_packager_editor.metadata.author = v;
            Task::none()
        }
        ModPackagerMessage::DescriptionChanged(v) => {
            app.state.mod_packager_editor.metadata.description = v;
            Task::none()
        }
        ModPackagerMessage::Export => {
            {
                let state = &mut app.state.mod_packager_editor;
                if state.selected_files.is_empty() {
                    state.status_msg = "No files selected.".into();
                    return Task::none();
                }
                if state.metadata.name.trim().is_empty() {
                    state.status_msg = "Mod name is required.".into();
                    return Task::none();
                }
                state.loading_state = LoadingState::Loading;
                state.status_msg = "Building package…".into();
            }
            let files = app.state.mod_packager_editor.selected_files.clone();
            let meta = app.state.mod_packager_editor.metadata.clone();
            let game_path = app.state.shared_game_path.clone();

            Task::perform(
                async move { build_zip(files, meta, game_path) },
                |result| {
                    crate::message::Message::mod_packager(ModPackagerMessage::Exported(result))
                },
            )
        }
        ModPackagerMessage::Exported(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(ref path) => {
                    state.status_msg = format!("Exported to {}", path.display());
                    state.loading_state = LoadingState::Loaded(());
                }
                Err(ref e) => {
                    state.status_msg = format!("Export failed: {}", e);
                    state.loading_state = LoadingState::Failed(e.clone());
                }
            }
            Task::none()
        }
    }
}

fn build_zip(files: Vec<PathBuf>, meta: ModMetadata, game_path: String) -> Result<PathBuf, String> {
    let mod_name = meta.name.trim().replace(' ', "_");
    let output_dir = if game_path.is_empty() {
        std::env::temp_dir()
    } else {
        let p = PathBuf::from(&game_path).join("mod_output");
        std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
        p
    };
    let output_path = output_dir.join(format!("{}.zip", mod_name));

    let file = std::fs::File::create(&output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for src in &files {
        let entry_name = src
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| src.to_string_lossy().into_owned());
        zip.start_file(&entry_name, options)
            .map_err(|e| e.to_string())?;
        let data = std::fs::read(src).map_err(|e| format!("{}: {}", src.display(), e))?;
        zip.write_all(&data).map_err(|e| e.to_string())?;
    }

    let manifest = serde_json::json!({
        "name": meta.name,
        "version": meta.version,
        "author": meta.author,
        "description": meta.description,
        "files": files.iter()
            .filter_map(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
    });
    zip.start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    zip.write_all(&manifest_bytes).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(output_path)
}
