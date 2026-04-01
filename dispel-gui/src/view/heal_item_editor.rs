use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{labeled_input, labeled_select};
use dispel_core::HealItemFlag;
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text,
    vertical_space,
};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_heal_item_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.heal_item_editor;

        let controls = row![button(text("Load Catalog").size(13))
            .padding([8, 16])
            .on_press(Message::HealItemOpLoadCatalog)
            .style(style::chip),]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let item_list: Vec<Element<Message>> = editor
            .filtered_items
            .iter()
            .enumerate()
            .map(|(idx, (_, item))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] {} - {}g\n  HP:{}/MP:{}",
                    item.id, item.name, item.base_price, item.health_points, item.mana_points
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::HealItemOpSelectItem(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Heal Item Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _item)) = editor.filtered_items.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::HealItemOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Description:",
                    &editor.edit_description,
                    move |v| Message::HealItemOpFieldChanged(orig, "description".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Base Price (gold):",
                    &editor.edit_base_price,
                    move |v| Message::HealItemOpFieldChanged(orig, "base_price".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Health Points:",
                    &editor.edit_health_points,
                    move |v| Message::HealItemOpFieldChanged(orig, "health_points".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Mana Points:",
                    &editor.edit_mana_points,
                    move |v| Message::HealItemOpFieldChanged(orig, "mana_points".into(), v),
                ));

                let flag_options = vec![HealItemFlag::None, HealItemFlag::FullRestoration];

                let make_flag_select = |label: &'static str,
                                        field: &'static str,
                                        value_str: &str|
                 -> Element<'_, Message> {
                    let flag = if value_str == "FullRestoration" {
                        HealItemFlag::FullRestoration
                    } else {
                        HealItemFlag::None
                    };
                    labeled_select(label, flag, flag_options.clone(), move |v| {
                        let val = if v == HealItemFlag::FullRestoration {
                            "FullRestoration".to_string()
                        } else {
                            "None".to_string()
                        };
                        Message::HealItemOpFieldChanged(orig, field.to_string(), val)
                    })
                };

                detail_content.push(make_flag_select(
                    "Restore Full Health:",
                    "restore_full_health",
                    &editor.edit_restore_full_health,
                ));
                detail_content.push(make_flag_select(
                    "Restore Full Mana:",
                    "restore_full_mana",
                    &editor.edit_restore_full_mana,
                ));
                detail_content.push(make_flag_select(
                    "Poison Heal:",
                    "poison_heal",
                    &editor.edit_poison_heal,
                ));
                detail_content.push(make_flag_select(
                    "Petrify Heal:",
                    "petrif_heal",
                    &editor.edit_petrif_heal,
                ));
                detail_content.push(make_flag_select(
                    "Polymorph Heal:",
                    "polimorph_heal",
                    &editor.edit_polimorph_heal,
                ));
            }
        } else {
            detail_content.push(
                text("No heal item selected")
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
            text("Heal Items").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::HealItemOpScanItems)
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
            container(controls).style(style::toolbar_container),
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
                    button(text("Save Heal Items"))
                        .on_press(Message::HealItemOpSave)
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
