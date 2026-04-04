use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_wave_ini_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.wave_ini_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_waves
            .iter()
            .enumerate()
            .map(|(idx, (_, wave))| {
                let is_selected = editor.selected_idx == Some(idx);
                let filename = wave.snf_filename.as_deref().unwrap_or("(none)");
                let label = format!("[{}] {}", wave.id, filename);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::WaveIniOpSelectWave(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Wave Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, wave)) = editor.filtered_waves.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::WaveIniOpFieldChanged(orig, "id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "SNF Filename:",
                    &editor.edit_snf_filename,
                    move |v| Message::WaveIniOpFieldChanged(orig, "snf_filename".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Unknown Flag:",
                    &editor.edit_unknown_flag,
                    move |v| Message::WaveIniOpFieldChanged(orig, "unknown_flag".into(), v),
                ));

                if wave.snf_filename.is_some() {
                    detail_content.push(vertical_space().height(8).into());
                    detail_content.push(
                        button(text("Export to WAV").size(13))
                            .on_press(Message::WaveIniOpExportWav(orig))
                            .padding([8, 16])
                            .style(style::commit_button)
                            .into(),
                    );
                }
            }
        } else {
            detail_content.push(
                text("No wave selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);

        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(380)
            .style(style::info_card);

        let item_list_header = row![
            text("Wave Entries").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::WaveIniOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![left_panel, detail_panel.width(Length::FillPortion(2))]
            .spacing(0)
            .height(Length::Fill);

        column![
            horizontal_rule(1),
            main_content,
            container(
                row![
                    text(&editor.status_msg).size(13).style(style::subtle_text),
                    horizontal_space(),
                    if !editor.export_status.is_empty() {
                        Element::from(
                            text(&editor.export_status)
                                .size(12)
                                .style(style::subtle_text),
                        )
                    } else {
                        Element::from(text(""))
                    },
                    if editor.is_loading {
                        Element::from(text("Loading...").size(13))
                    } else {
                        Element::from(text(""))
                    },
                    horizontal_space().width(20),
                    button(text("Save Wave Ini"))
                        .on_press(Message::WaveIniOpSave)
                        .style(style::commit_button),
                ]
                .padding([10, 20])
                .align_y(iced::Alignment::Center),
            )
            .width(Fill)
            .style(style::status_bar),
        ]
        .spacing(0)
        .height(Length::Fill)
        .into()
    }
}
