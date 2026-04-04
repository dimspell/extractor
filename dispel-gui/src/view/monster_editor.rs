use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_monster_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.monster_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_monsters
            .iter()
            .enumerate()
            .map(|(idx, (_, monster))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] {} - HP:{}/MP:{}",
                    monster.id, monster.name, monster.health_points_max, monster.mana_points_max
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::MonsterOpSelectMonster(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Monster Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _monster)) = editor.filtered_monsters.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::MonsterOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input("HP Max:", &editor.edit_hp_max, move |v| {
                    Message::MonsterOpFieldChanged(orig, "health_points_max".into(), v)
                }));
                detail_content.push(labeled_input("HP Min:", &editor.edit_hp_min, move |v| {
                    Message::MonsterOpFieldChanged(orig, "health_points_min".into(), v)
                }));
                detail_content.push(labeled_input("MP Max:", &editor.edit_mp_max, move |v| {
                    Message::MonsterOpFieldChanged(orig, "mana_points_max".into(), v)
                }));
                detail_content.push(labeled_input("MP Min:", &editor.edit_mp_min, move |v| {
                    Message::MonsterOpFieldChanged(orig, "mana_points_min".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Walk Speed:",
                    &editor.edit_walk_speed,
                    move |v| Message::MonsterOpFieldChanged(orig, "walk_speed".into(), v),
                ));
                detail_content.push(labeled_input(
                    "To Hit Max:",
                    &editor.edit_to_hit_max,
                    move |v| Message::MonsterOpFieldChanged(orig, "to_hit_max".into(), v),
                ));
                detail_content.push(labeled_input(
                    "To Hit Min:",
                    &editor.edit_to_hit_min,
                    move |v| Message::MonsterOpFieldChanged(orig, "to_hit_min".into(), v),
                ));
                detail_content.push(labeled_input(
                    "To Dodge Max:",
                    &editor.edit_to_dodge_max,
                    move |v| Message::MonsterOpFieldChanged(orig, "to_dodge_max".into(), v),
                ));
                detail_content.push(labeled_input(
                    "To Dodge Min:",
                    &editor.edit_to_dodge_min,
                    move |v| Message::MonsterOpFieldChanged(orig, "to_dodge_min".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Offense Max:",
                    &editor.edit_offense_max,
                    move |v| Message::MonsterOpFieldChanged(orig, "offense_max".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Offense Min:",
                    &editor.edit_offense_min,
                    move |v| Message::MonsterOpFieldChanged(orig, "offense_min".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Defense Max:",
                    &editor.edit_defense_max,
                    move |v| Message::MonsterOpFieldChanged(orig, "defense_max".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Defense Min:",
                    &editor.edit_defense_min,
                    move |v| Message::MonsterOpFieldChanged(orig, "defense_min".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Magic Attack Max:",
                    &editor.edit_magic_attack_max,
                    move |v| Message::MonsterOpFieldChanged(orig, "magic_attack_max".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Magic Attack Min:",
                    &editor.edit_magic_attack_min,
                    move |v| Message::MonsterOpFieldChanged(orig, "magic_attack_min".into(), v),
                ));
                detail_content.push(labeled_input("AI Type:", &editor.edit_ai_type, move |v| {
                    Message::MonsterOpFieldChanged(orig, "ai_type".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Exp Gain Max:",
                    &editor.edit_exp_gain_max,
                    move |v| Message::MonsterOpFieldChanged(orig, "exp_gain_max".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Exp Gain Min:",
                    &editor.edit_exp_gain_min,
                    move |v| Message::MonsterOpFieldChanged(orig, "exp_gain_min".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Gold Drop Max:",
                    &editor.edit_gold_drop_max,
                    move |v| Message::MonsterOpFieldChanged(orig, "gold_drop_max".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Gold Drop Min:",
                    &editor.edit_gold_drop_min,
                    move |v| Message::MonsterOpFieldChanged(orig, "gold_drop_min".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Detection Sight:",
                    &editor.edit_detection_sight_size,
                    move |v| Message::MonsterOpFieldChanged(orig, "detection_sight_size".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Distance Range:",
                    &editor.edit_distance_range_size,
                    move |v| Message::MonsterOpFieldChanged(orig, "distance_range_size".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Boldness:",
                    &editor.edit_boldness,
                    move |v| Message::MonsterOpFieldChanged(orig, "boldness".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Attack Speed:",
                    &editor.edit_attack_speed,
                    move |v| Message::MonsterOpFieldChanged(orig, "attack_speed".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No monster selected")
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
            text("Monsters").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::MonsterOpScanMonsters)
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
                    button(text("Save Monsters"))
                        .on_press(Message::MonsterOpSave)
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
