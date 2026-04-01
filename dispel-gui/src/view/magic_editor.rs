use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_magic_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.magic_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_spells
            .iter()
            .enumerate()
            .map(|(idx, (_, spell))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] DMG:{} RNG:{} MP:{}",
                    spell.id, spell.base_damage, spell.range, spell.mana_cost
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::MagicOpSelectSpell(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Spell Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _spell)) = editor.filtered_spells.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input(
                    "Mana Cost:",
                    &editor.edit_mana_cost,
                    move |v| Message::MagicOpFieldChanged(orig, "mana_cost".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Success Rate:",
                    &editor.edit_success_rate,
                    move |v| Message::MagicOpFieldChanged(orig, "success_rate".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Base Damage:",
                    &editor.edit_base_damage,
                    move |v| Message::MagicOpFieldChanged(orig, "base_damage".into(), v),
                ));
                detail_content.push(labeled_input("Range:", &editor.edit_range, move |v| {
                    Message::MagicOpFieldChanged(orig, "range".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Level Required:",
                    &editor.edit_level_required,
                    move |v| Message::MagicOpFieldChanged(orig, "level_required".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Effect Type:",
                    &editor.edit_effect_type,
                    move |v| Message::MagicOpFieldChanged(orig, "effect_type".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Magic School:",
                    &editor.edit_magic_school,
                    move |v| Message::MagicOpFieldChanged(orig, "magic_school".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Target Type:",
                    &editor.edit_target_type,
                    move |v| Message::MagicOpFieldChanged(orig, "target_type".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No spell selected")
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
            text("Spells").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::MagicOpScanSpells)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![left_panel, detail_panel.width(Length::FillPortion(2)),]
            .spacing(0)
            .height(Length::Fill);

        column![
            horizontal_rule(1),
            main_content,
            container(
                row![
                    text(&editor.status_msg).size(13).style(style::subtle_text),
                    horizontal_space(),
                    if editor.is_loading {
                        Element::from(text("Loading...").size(13))
                    } else {
                        Element::from(text(""))
                    },
                    horizontal_space().width(20),
                    button(text("Save Spells"))
                        .on_press(Message::MagicOpSave)
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
