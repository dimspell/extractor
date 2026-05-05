use crate::app::App;
use crate::components::textarea::textarea;
use crate::loading_state::LoadingState;
use crate::message::editor::localization::LocalizationMessage;
use crate::message::{Message, MessageExt};
use crate::style;
use iced::widget::{
    button, checkbox, column, container, pick_list, progress_bar, row, scrollable, text, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Fill, Length};

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
        .width(Length::Fixed(160.0));

        let search_input = text_input("Search…", &state.search_query)
            .on_input(|v| Message::localization(LocalizationMessage::SearchChanged(v)))
            .width(Length::Fixed(160.0))
            .size(12);

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

        let progress = container(progress_bar(0.0..=1.0, progress_val)).width(Length::Fixed(80.0));
        let progress_label = text(format!("{done} / {total}"))
            .size(12)
            .style(style::subtle_text);

        let filter_row = row![
            file_filter,
            search_input,
            untranslated_toggle,
            overlong_toggle,
            progress,
            progress_label
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // ── Master-detail split ──────────────────────────────────────────────
        const PAGE_SIZE: usize = 250;
        let visible = state.visible_entries();
        let visible_total = visible.len();
        let page = state.page;
        let total_pages = visible_total.div_ceil(PAGE_SIZE).max(1);
        let page_start = page * PAGE_SIZE;
        let page_end = (page_start + PAGE_SIZE).min(visible_total);
        let page_visible = &visible[page_start..page_end];

        let entry_rows: Vec<Element<'_, Message>> = page_visible
            .iter()
            .map(|(idx, entry)| {
                let idx = *idx;
                let is_selected = state.selected_idx == Some(idx);
                let is_overlong = entry.would_truncate();
                let is_translated = entry.is_translated();

                // Status indicator
                let indicator = if is_overlong {
                    text("✗").size(11).style(|_t| iced::widget::text::Style {
                        color: Some(Color::from_rgb(0.85, 0.2, 0.2)),
                    })
                } else if is_translated {
                    text("✓").size(11).style(|_t| iced::widget::text::Style {
                        color: Some(Color::from_rgb(0.3, 0.75, 0.3)),
                    })
                } else {
                    text("●").size(11).style(style::subtle_text)
                };

                let short_file = entry
                    .file_path
                    .split('/')
                    .last()
                    .unwrap_or(&entry.file_path);

                // Truncate original for list display
                let original_preview: String = entry.original.chars().take(60).collect();
                let original_preview = if entry.original.len() > 60 {
                    format!("{original_preview}…")
                } else {
                    original_preview
                };

                let row_content = row![
                    indicator,
                    text(entry.field_name)
                        .size(11)
                        .width(Length::Fixed(90.0))
                        .style(style::subtle_text),
                    text(format!("#{}", entry.record_id))
                        .size(10)
                        .width(Length::Fixed(32.0))
                        .style(style::subtle_text),
                    column![
                        text(short_file).size(10).style(style::subtle_text),
                        text(original_preview).size(11),
                    ]
                    .spacing(1)
                ]
                .spacing(6)
                .align_y(Alignment::Center);

                button(row_content)
                    .on_press(Message::localization(LocalizationMessage::SelectEntry(idx)))
                    .width(Fill)
                    .style(if is_selected {
                        style::selected_row_button
                    } else {
                        style::normal_row_button
                    })
                    .into()
            })
            .collect();

        let entry_list =
            scrollable(column(entry_rows).spacing(1).padding([4, 4]).width(Fill)).height(Fill);

        let prev_page_btn = button(text("‹").size(12)).on_press_maybe(if page > 0 {
            Some(Message::localization(LocalizationMessage::PagePrev))
        } else {
            None
        });
        let next_page_btn = button(text("›").size(12)).on_press_maybe(if page + 1 < total_pages {
            Some(Message::localization(LocalizationMessage::PageNext))
        } else {
            None
        });
        let page_label = text(format!(
            "{} – {} of {visible_total}",
            page_start + 1,
            page_end
        ))
        .size(11)
        .style(style::subtle_text);

        let pagination = row![prev_page_btn, page_label, next_page_btn]
            .spacing(6)
            .align_y(Alignment::Center);

        let list_with_pager = column![entry_list, pagination]
            .spacing(4)
            .padding([0, 4])
            .width(Fill)
            .height(Fill);

        let list_pane = container(list_with_pager)
            .width(Length::FillPortion(4))
            .height(Fill)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.25, 0.18, 0.1),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        // ── Editor panel (right) ─────────────────────────────────────────────
        let editor_pane: Element<'_, Message> = if let Some(idx) = state.selected_idx {
            if let Some(entry) = state.entries.get(idx) {
                let short_file = entry
                    .file_path
                    .split('/')
                    .last()
                    .unwrap_or(&entry.file_path);

                let context_line = text(format!(
                    "{short_file}  ›  #{id}  ›  {field}",
                    id = entry.record_id,
                    field = entry.field_name,
                ))
                .size(11)
                .style(style::subtle_text);

                let encoding_line = if entry.max_bytes > 0 {
                    text(format!(
                        "Encoding: {:?}   Max: {} bytes",
                        entry.encoding, entry.max_bytes
                    ))
                    .size(10)
                    .style(style::subtle_text)
                } else {
                    text(format!("Encoding: {:?}", entry.encoding))
                        .size(10)
                        .style(style::subtle_text)
                };

                let original_label = text("Original").size(11).style(style::subtle_text);
                let original_text = container(text(entry.original.as_str()).size(12))
                    .padding([8, 10])
                    .width(Fill)
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.2))),
                        border: Border {
                            color: Color::from_rgb(0.25, 0.18, 0.1),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    });

                let translation_label = text("Translation").size(11).style(style::subtle_text);
                let translation_editor = textarea(&state.translation_content.0, |action| {
                    Message::localization(LocalizationMessage::TranslationAction(action))
                });

                // Byte counter
                let encoded_len = entry.encoded_translation_len();
                let is_overlong = entry.would_truncate();
                let byte_counter: Element<'_, Message> = if entry.max_bytes > 0 {
                    let label = format!("{encoded_len} / {} bytes", entry.max_bytes);
                    if is_overlong {
                        text(label)
                            .size(11)
                            .style(|_t| iced::widget::text::Style {
                                color: Some(Color::from_rgb(0.85, 0.2, 0.2)),
                            })
                            .into()
                    } else {
                        text(label).size(11).style(style::subtle_text).into()
                    }
                } else {
                    text("").size(11).into()
                };

                // Navigation buttons
                let prev_btn = button(text("← Prev untranslated").size(11))
                    .on_press(Message::localization(LocalizationMessage::NavigatePrev));
                let next_btn = button(text("Next untranslated →").size(11))
                    .on_press(Message::localization(LocalizationMessage::NavigateNext));

                let nav_row = row![prev_btn, next_btn, byte_counter]
                    .spacing(8)
                    .align_y(Alignment::Center);

                column![
                    context_line,
                    encoding_line,
                    original_label,
                    original_text,
                    translation_label,
                    translation_editor,
                    nav_row,
                ]
                .spacing(6)
                .padding([8, 12])
                .width(Fill)
                .height(Fill)
                .into()
            } else {
                placeholder_panel()
            }
        } else {
            placeholder_panel()
        };

        let editor_container = container(editor_pane)
            .width(Length::FillPortion(6))
            .height(Fill)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.25, 0.18, 0.1),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        let main_area = row![list_pane, editor_container].spacing(8).height(Fill);

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
        let content = column![toolbar, filter_row, main_area, action_bar]
            .spacing(8)
            .padding(12)
            .width(Fill)
            .height(Fill);

        container(content).width(Fill).height(Fill).into()
    }
}

fn placeholder_panel<'a>() -> Element<'a, Message> {
    container(
        text("Select an entry to start translating")
            .size(13)
            .style(style::subtle_text),
    )
    .width(Fill)
    .height(Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}
