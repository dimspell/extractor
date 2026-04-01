use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::labeled_input;
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, image, row, scrollable, text,
    vertical_space,
};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_heal_item_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.heal_item_editor;

        let sprite_path_row = row![
            text("Sprites: ")
                .size(12)
                .width(60)
                .style(style::subtle_text),
            container(
                text(if editor.sprite_base_path.is_empty() {
                    "No sprite path selected"
                } else {
                    &editor.sprite_base_path
                })
                .size(11)
                .font(Font::MONOSPACE)
            )
            .padding([4, 10])
            .width(Fill)
            .style(style::sql_editor_container),
            button(text("Browse").size(11))
                .on_press(Message::HealItemOpBrowseSpritePath)
                .padding([5, 10])
                .style(style::browse_button),
            horizontal_space().width(20),
            button(text("Load Catalog").size(11))
                .on_press(Message::HealItemOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

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
                button(text("Save Heal Items"))
                    .on_press(Message::HealItemOpSave)
                    .style(style::commit_button),
            ]
            .padding([10, 20])
            .align_y(iced::Alignment::Center),
        )
        .width(Fill)
        .style(style::status_bar);

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
                detail_content.push(labeled_input(
                    "Restore Full Health (Full/None):",
                    &editor.edit_restore_full_health,
                    move |v| Message::HealItemOpFieldChanged(orig, "restore_full_health".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Restore Full Mana (Full/None):",
                    &editor.edit_restore_full_mana,
                    move |v| Message::HealItemOpFieldChanged(orig, "restore_full_mana".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Poison Heal (Active/None):",
                    &editor.edit_poison_heal,
                    move |v| Message::HealItemOpFieldChanged(orig, "poison_heal".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Petrify Heal (Active/None):",
                    &editor.edit_petrif_heal,
                    move |v| Message::HealItemOpFieldChanged(orig, "petrif_heal".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Polymorph Heal (Active/None):",
                    &editor.edit_polimorph_heal,
                    move |v| Message::HealItemOpFieldChanged(orig, "polimorph_heal".into(), v),
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

        // Add sprite display if available
        let sprite_display: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            if let Some((_, item)) = editor.filtered_items.get(idx) {
                if let Some(sprite_path) = editor.get_sprite_path_for_item(item.id) {
                    let img: iced::widget::Image<iced::widget::image::Handle> =
                        image::Image::new(sprite_path.clone())
                            .width(Length::Fixed(64.0))
                            .height(Length::Fixed(64.0));
                    let img_container: iced::widget::Container<'_, Message> = container(img)
                        .padding(8)
                        .width(Length::Fixed(80.0))
                        .center_x(Length::Fixed(80.0));
                    img_container.into()
                } else {
                    let no_sprite_container: iced::widget::Container<'_, Message> =
                        container(text("No sprite found").size(12).style(style::subtle_text))
                            .padding(8)
                            .width(Length::Fixed(80.0))
                            .center_x(Length::Fixed(80.0));
                    no_sprite_container.into()
                }
            } else {
                container(text("").size(12))
                    .width(Length::Fixed(80.0))
                    .into()
            }
        } else {
            container(text("").size(12))
                .width(Length::Fixed(80.0))
                .into()
        };

        let detail_panel = container(
            row![sprite_display, detail_scroll]
                .spacing(16)
                .padding(16)
                .width(380),
        )
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
            container(sprite_path_row).style(style::toolbar_container),
            horizontal_rule(1),
            main_content,
            status_row,
        ]
        .spacing(0)
        .height(Length::Fill)
        .into()
    }
}
