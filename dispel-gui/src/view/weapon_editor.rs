use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::labeled_input;
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text,
    vertical_space,
};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_weapon_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.weapon_editor;

        let status_row = container(
            row![
                text(&editor.status_msg).size(13).style(style::subtle_text),
                horizontal_space(),
                if editor.is_loading {
                    Element::from(text("Loading...").size(13))
                } else {
                    Element::from(text(""))
                },
                horizontal_space().width(20),
                button(text("Save Weapons"))
                    .on_press(Message::WeaponOpSave)
                    .style(style::commit_button),
            ]
            .padding([10, 20])
            .align_y(iced::Alignment::Center),
        )
        .width(Fill)
        .style(style::status_bar);

        let weapon_list: Vec<Element<Message>> = editor
            .filtered_weapons
            .iter()
            .enumerate()
            .map(|(idx, (_, weapon))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] {} - {}g\n  ATK:{}/DEF:{}/MAG:{}\n  STR:{}/AGI:{}/WIS:{}",
                    weapon.id,
                    weapon.name,
                    weapon.base_price,
                    weapon.attack,
                    weapon.defense,
                    weapon.magical_strength,
                    weapon.req_strength,
                    weapon.req_agility,
                    weapon.req_wisdom
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::WeaponOpSelectWeapon(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let weapon_scroll = scrollable(column(weapon_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Weapon Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _weapon)) = editor.filtered_weapons.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::WeaponOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Description:",
                    &editor.edit_description,
                    move |v| Message::WeaponOpFieldChanged(orig, "description".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Base Price (gold):",
                    &editor.edit_base_price,
                    move |v| Message::WeaponOpFieldChanged(orig, "base_price".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Health Points:",
                    &editor.edit_health_points,
                    move |v| Message::WeaponOpFieldChanged(orig, "health_points".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Mana Points:",
                    &editor.edit_mana_points,
                    move |v| Message::WeaponOpFieldChanged(orig, "mana_points".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Strength:",
                    &editor.edit_strength,
                    move |v| Message::WeaponOpFieldChanged(orig, "strength".into(), v),
                ));
                detail_content.push(labeled_input("Agility:", &editor.edit_agility, move |v| {
                    Message::WeaponOpFieldChanged(orig, "agility".into(), v)
                }));
                detail_content.push(labeled_input("Wisdom:", &editor.edit_wisdom, move |v| {
                    Message::WeaponOpFieldChanged(orig, "wisdom".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Constitution:",
                    &editor.edit_constitution,
                    move |v| Message::WeaponOpFieldChanged(orig, "constitution".into(), v),
                ));
                detail_content.push(labeled_input(
                    "To Dodge:",
                    &editor.edit_to_dodge,
                    move |v| Message::WeaponOpFieldChanged(orig, "to_dodge".into(), v),
                ));
                detail_content.push(labeled_input("To Hit:", &editor.edit_to_hit, move |v| {
                    Message::WeaponOpFieldChanged(orig, "to_hit".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Attack Power:",
                    &editor.edit_attack,
                    move |v| Message::WeaponOpFieldChanged(orig, "attack".into(), v),
                ));
                detail_content.push(labeled_input("Defense:", &editor.edit_defense, move |v| {
                    Message::WeaponOpFieldChanged(orig, "defense".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Magical Strength:",
                    &editor.edit_magical_strength,
                    move |v| Message::WeaponOpFieldChanged(orig, "magical_strength".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Durability:",
                    &editor.edit_durability,
                    move |v| Message::WeaponOpFieldChanged(orig, "durability".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Required Strength:",
                    &editor.edit_req_strength,
                    move |v| Message::WeaponOpFieldChanged(orig, "req_strength".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Required Agility:",
                    &editor.edit_req_agility,
                    move |v| Message::WeaponOpFieldChanged(orig, "req_agility".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Required Wisdom:",
                    &editor.edit_req_wisdom,
                    move |v| Message::WeaponOpFieldChanged(orig, "req_wisdom".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No weapon selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);
        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(280)
            .style(style::info_card);

        let _list_header: iced::widget::Row<'_, Message, iced::Theme, iced::Renderer> = row![
            text("Weapons").size(14),
            horizontal_space(),
            text(format!("{} found", editor.filtered_weapons.len()))
                .size(12)
                .style(style::subtle_text)
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let weapon_list_header = row![
            text("Weapon List").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::WeaponOpScanWeapons)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(weapon_list_header).style(style::grid_header_cell),
            weapon_scroll,
        ];

        let main_content = row![left_panel, detail_panel.width(Length::FillPortion(2)),]
            .spacing(0)
            .height(Length::Fill);

        column![horizontal_rule(1), main_content, status_row,]
            .spacing(0)
            .height(Length::Fill)
            .into()
    }
}
