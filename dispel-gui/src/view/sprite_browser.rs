use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use iced::widget::{
    button, column, container, image, progress_bar, row, scrollable, text, text_input,
};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_sprite_browser_tab(&self) -> Element<'_, Message> {
        let browser = &self.sprite_browser;

        // Sprite list items
        let item_list: Vec<Element<'_, Message>> = if browser.is_loading {
            vec![container(
                column![
                    text("Scanning for sprites...").size(13),
                    progress_bar(0.0..=1.0, 0.5).style(style::loading_progress_bar),
                ]
                .spacing(12)
                .padding([16, 20]),
            )
            .width(Fill)
            .into()]
        } else if browser.filtered_sprites.is_empty() && !browser.search_query.is_empty() {
            vec![container(
                text(format!("No matches for \"{}\"", browser.search_query))
                    .size(12)
                    .style(style::subtle_text),
            )
            .width(Fill)
            .padding([16, 20])
            .into()]
        } else {
            browser
                .filtered_sprites
                .iter()
                .enumerate()
                .map(|(filtered_i, (orig_i, sprite))| {
                    let is_selected = browser.selected_sprite_idx == Some(*orig_i);
                    let btn = button(
                        row![
                            text(format!("{:04}", orig_i)).size(11).width(40),
                            text(&sprite.name).size(12),
                            text(format!(
                                "{} seq, {} frames",
                                sprite.sequence_count,
                                sprite.frame_counts.iter().sum::<usize>()
                            ))
                            .size(10)
                            .style(style::subtle_text),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                    )
                    .width(Fill)
                    .padding([8, 12])
                    .on_press(Message::SpriteBrowserOpSelectSprite(filtered_i));
                    if is_selected {
                        btn.style(style::active_tab_button).into()
                    } else {
                        btn.style(style::tab_button).into()
                    }
                })
                .collect()
        };

        // Search bar
        let search_bar = text_input("Filter sprites...", &browser.search_query)
            .on_input(Message::SpriteBrowserOpSearch)
            .padding([6, 10])
            .size(12);

        // Left panel: header + search + scrollable sprite list
        let sprite_list_header = row![
            text("Sprites").size(13),
            horizontal_space(),
            if browser.is_loading {
                Element::from(
                    text(format!(
                        "Scanning... {} found so far",
                        browser.sprites.len()
                    ))
                    .size(11)
                    .style(style::subtle_text),
                )
            } else {
                Element::from(
                    text(format!(
                        "{}/{}",
                        browser.filtered_sprites.len(),
                        browser.sprites.len()
                    ))
                    .size(11)
                    .style(style::subtle_text),
                )
            },
            horizontal_space().width(12),
            if browser.is_loading {
                Element::from(
                    button(text("Scanning...").size(11))
                        .padding([4, 10])
                        .style(style::run_button_disabled),
                )
            } else {
                Element::from(
                    button(text("Scan").size(11))
                        .on_press(Message::SpriteBrowserOpScan)
                        .padding([4, 10])
                        .style(style::run_button),
                )
            },
        ]
        .padding([8, 12])
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(sprite_list_header).style(style::grid_header_cell),
            container(search_bar).padding([4, 8]).width(Fill),
            scrollable(column(item_list).spacing(2)).height(Length::Fill),
        ];

        // Right panel: sequence selector, frame strip, main display
        let sequence_row: Element<'_, Message> = if let Some(idx) = browser.selected_sprite_idx {
            if let Some(sprite) = browser.sprites.get(idx) {
                let seq_buttons: Vec<Element<'_, Message>> = (0..sprite.sequence_count)
                    .map(|seq_idx| {
                        let is_selected = browser.selected_sequence == seq_idx;
                        let frame_count = sprite.frame_counts.get(seq_idx).copied().unwrap_or(0);
                        let btn =
                            button(text(format!("Seq {} ({})", seq_idx, frame_count)).size(11))
                                .padding([4, 8])
                                .on_press(Message::SpriteBrowserOpSelectSequence(seq_idx));
                        if is_selected {
                            btn.style(style::active_chip).into()
                        } else {
                            btn.style(style::chip).into()
                        }
                    })
                    .collect();
                row(seq_buttons).spacing(4).padding([4, 8]).into()
            } else {
                text("Select a sprite")
                    .size(12)
                    .style(style::subtle_text)
                    .into()
            }
        } else {
            text("Select a sprite")
                .size(12)
                .style(style::subtle_text)
                .into()
        };

        // Frame thumbnails - horizontal scrollable with fixed height
        let frame_strip: Element<'_, Message> = {
            let thumbnails: Vec<Element<'_, Message>> = browser
                .frames
                .iter()
                .enumerate()
                .map(|(i, frame)| {
                    let is_selected = browser.selected_frame == i;
                    let btn = button(
                        image(frame.image.clone())
                            .width(Length::Fixed(48.0))
                            .height(Length::Fixed(48.0)),
                    )
                    .padding(2)
                    .on_press(Message::SpriteBrowserOpSelectFrame(i));
                    if is_selected {
                        btn.style(style::active_chip).into()
                    } else {
                        btn.style(style::chip).into()
                    }
                })
                .collect();

            scrollable(row(thumbnails).spacing(4).padding(8))
                .width(Length::Fill)
                .height(Length::Fixed(70.0))
                .into()
        };

        // Main image display
        let main_display: Element<'_, Message> =
            if let Some(frame) = browser.frames.get(browser.selected_frame) {
                let img = image(frame.image.clone())
                    .width(Length::Fill)
                    .height(Length::Fill);
                container(
                    column![
                        img,
                        text(format!(
                            "Sequence: {}, Frame: {} of {}",
                            frame.sequence_idx,
                            frame.frame_idx,
                            browser.frames.len()
                        ))
                        .size(11)
                        .style(style::subtle_text),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(16)
                .into()
            } else {
                container(
                    column![
                        text("No frames loaded").size(14),
                        text("Select a sprite to view its frames")
                            .size(12)
                            .style(style::subtle_text),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(16)
                .into()
            };

        let right_content =
            column![sequence_row, frame_strip, horizontal_rule(1), main_display,].spacing(8);

        let right_panel = container(scrollable(right_content).height(Length::Fill))
            .padding(0)
            .width(Length::FillPortion(2))
            .style(style::info_card);

        // Main content
        let main_content = row![left_panel, right_panel,]
            .spacing(0)
            .height(Length::Fill);

        column![horizontal_rule(1), main_content,]
            .spacing(0)
            .height(Length::Fill)
            .into()
    }
}
