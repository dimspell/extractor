use crate::app::App;
use crate::loading_state::LoadingState;
use crate::message::editor::localization::LocalizationMessage;
use crate::message::{Message, MessageExt};
use crate::style;
use iced::widget::{
    button, checkbox, column, container, pick_list, progress_bar, row, scrollable, text, text_input,
};
use iced::{Alignment, Border, Color, Element, Fill, Length};

// ─── File filter display wrapper ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum FileFilter {
    All,
    File(String),
}

impl std::fmt::Display for FileFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFilter::All => write!(f, "All files"),
            FileFilter::File(name) => {
                let short = name.split('/').last().unwrap_or(name);
                write!(f, "{short}")
            }
        }
    }
}

// ─── View ────────────────────────────────────────────────────────────────────

impl App {
    pub fn view_localization_manager_tab(&self) -> Element<'_, Message> {
        let state = &self.state.localization_manager;
        let is_loading = matches!(state.loading_state, LoadingState::Loading);
        let has_entries = !state.entries.is_empty();
        let backup_exists = state.backup_exists(&self.state.shared_game_path);

        // ── Toolbar ──────────────────────────────────────────────────────────
        let scan_btn = button(text("Scan").size(12)).on_press_maybe(if is_loading {
            None
        } else {
            Some(Message::localization(LocalizationMessage::Scan))
        });
        let csv_btn =
            button(text("Export CSV").size(12)).on_press_maybe(if is_loading || !has_entries {
                None
            } else {
                Some(Message::localization(LocalizationMessage::ExportCsv))
            });
        let po_btn =
            button(text("Export PO").size(12)).on_press_maybe(if is_loading || !has_entries {
                None
            } else {
                Some(Message::localization(LocalizationMessage::ExportPo))
            });
        let import_btn =
            button(text("Import…").size(12)).on_press_maybe(if is_loading || !has_entries {
                None
            } else {
                Some(Message::localization(LocalizationMessage::ImportFile))
            });

        // Target language input for PO export
        let target_lang_input = text_input("Lang (e.g. pl)", &state.target_lang)
            .on_input(|v| Message::localization(LocalizationMessage::TargetLangChanged(v)))
            .width(Length::Fixed(90.0))
            .size(12);

        let toolbar = row![scan_btn, csv_btn, po_btn, target_lang_input, import_btn]
            .spacing(6)
            .align_y(Alignment::Center);

        // ── Filter row ───────────────────────────────────────────────────────
        let mut filter_options = vec![FileFilter::All];
        filter_options.extend(state.available_files().into_iter().map(FileFilter::File));

        let current_filter = state
            .filter_file
            .as_ref()
            .map(|f| FileFilter::File(f.clone()))
            .unwrap_or(FileFilter::All);

        let file_filter = pick_list(filter_options, Some(current_filter), |sel| {
            let opt = match sel {
                FileFilter::All => None,
                FileFilter::File(f) => Some(f),
            };
            Message::localization(LocalizationMessage::FilterFile(opt))
        })
        .width(Length::Fixed(200.0));

        let untranslated_toggle = row![
            checkbox(state.show_untranslated_only)
                .on_toggle(|_| Message::localization(LocalizationMessage::ToggleUntranslatedOnly)),
            text("Untranslated only").size(12),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let overlong_count = state.overlong_count();
        let overlong_toggle = row![
            checkbox(state.show_overlong_only)
                .on_toggle(|_| Message::localization(LocalizationMessage::ToggleOverlongOnly)),
            text(format!("Overlong ({overlong_count})")).size(12),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let total = state.total_count();
        let done = state.translated_count();
        let progress_val = if total == 0 {
            0.0
        } else {
            done as f32 / total as f32
        };

        let progress = progress_bar(0.0..=1.0, progress_val);
        let progress_label = text(format!("{done} / {total} translated"))
            .size(12)
            .style(style::subtle_text);

        let filter_row = row![
            file_filter,
            untranslated_toggle,
            overlong_toggle,
            progress,
            progress_label
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // ── Entries table ────────────────────────────────────────────────────
        let visible = state.visible_entries();

        let header = row![
            text("File").size(11).width(Length::Fixed(140.0)),
            text("Rec").size(11).width(Length::Fixed(36.0)),
            text("Field").size(11).width(Length::Fixed(100.0)),
            text("Original").size(11).width(Fill),
            text("Translation").size(11).width(Fill),
            text("Bytes").size(11).width(Length::Fixed(72.0)),
        ]
        .spacing(4)
        .padding([2, 0]);

        let rows: Vec<Element<'_, Message>> = visible
            .into_iter()
            .map(|(idx, entry)| {
                let is_overlong = entry.would_truncate();
                let short_file = entry
                    .file_path
                    .split('/')
                    .last()
                    .unwrap_or(&entry.file_path)
                    .to_owned();

                let translation_input = text_input("", &entry.translation)
                    .on_input(move |v| {
                        Message::localization(LocalizationMessage::TranslationChanged {
                            idx,
                            translation: v,
                        })
                    })
                    .width(Fill);

                let translation_cell: Element<'_, Message> = if is_overlong {
                    container(translation_input)
                        .style(|_theme| container::Style {
                            border: Border {
                                color: Color::from_rgb(0.9, 0.2, 0.2),
                                width: 1.5,
                                radius: 3.0.into(),
                            },
                            ..Default::default()
                        })
                        .into()
                } else {
                    translation_input.into()
                };

                // C1: byte counter — show encoded bytes / max_bytes
                let encoded_len = entry.encoded_translation_len();
                let byte_label: Element<'_, Message> = if entry.max_bytes > 0 {
                    let label = format!("{}/{}", encoded_len, entry.max_bytes);
                    if is_overlong {
                        text(label)
                            .size(10)
                            .width(Length::Fixed(72.0))
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgb(0.85, 0.2, 0.2)),
                            })
                            .into()
                    } else {
                        text(label)
                            .size(10)
                            .width(Length::Fixed(72.0))
                            .style(style::subtle_text)
                            .into()
                    }
                } else {
                    text("").size(10).width(Length::Fixed(72.0)).into()
                };

                let original = entry.original.clone();
                row![
                    text(short_file).size(11).width(Length::Fixed(140.0)),
                    text(entry.record_id.to_string())
                        .size(11)
                        .width(Length::Fixed(36.0)),
                    text(entry.field_name).size(11).width(Length::Fixed(100.0)),
                    text(original).size(11).width(Fill),
                    translation_cell,
                    byte_label,
                ]
                .spacing(4)
                .align_y(Alignment::Center)
                .into()
            })
            .collect();

        let table = scrollable(
            column![header]
                .push(column(rows).spacing(2))
                .spacing(4)
                .padding([4, 0])
                .width(Fill),
        )
        .height(Fill);

        // ── Mod metadata + action bar ─────────────────────────────────────────
        let name_input = text_input("Mod name (required)", &state.mod_metadata.name)
            .on_input(|v| Message::localization(LocalizationMessage::ModNameChanged(v)))
            .width(Length::Fixed(180.0));
        let version_input = text_input("1.0.0", &state.mod_metadata.version)
            .on_input(|v| Message::localization(LocalizationMessage::ModVersionChanged(v)))
            .width(Length::Fixed(80.0));
        let author_input = text_input("Author", &state.mod_metadata.author)
            .on_input(|v| Message::localization(LocalizationMessage::ModAuthorChanged(v)))
            .width(Length::Fixed(120.0));
        let apply_btn = button(text("Apply & Package").size(12)).on_press_maybe(if is_loading {
            None
        } else {
            Some(Message::localization(LocalizationMessage::ApplyAndPackage))
        });
        let revert_btn = button(text("Revert").size(12))
            .style(style::browse_button)
            .on_press_maybe(if is_loading || !backup_exists {
                None
            } else {
                Some(Message::localization(LocalizationMessage::Revert))
            });
        let status = text(&state.status_msg).size(11).style(style::subtle_text);

        let action_bar = row![
            name_input,
            version_input,
            author_input,
            apply_btn,
            revert_btn,
            status
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // ── Compose ──────────────────────────────────────────────────────────
        let content = column![toolbar, filter_row, table, action_bar]
            .spacing(8)
            .padding(12)
            .width(Fill)
            .height(Fill);

        container(content).width(Fill).height(Fill).into()
    }
}
