use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, image, row, scrollable, text,
};
use iced::{Element, Length};

impl App {
    pub fn view_sprite_browser_tab(&self) -> Element<'_, Message> {
        let browser = &self.sprite_browser;

        // Header
        let header = row![
            text("Sprite Browser").size(20),
            text(format!(" - {} sprites", browser.sprites.len()))
                .size(14)
                .style(style::subtle_text),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // Controls
        let controls = row![
            button(text("Scan Game Path").size(13))
                .padding([8, 16])
                .on_press(Message::SpriteBrowserOpScan)
                .style(style::chip),
            horizontal_space(),
            text(&self.shared_game_path)
                .size(11)
                .style(style::subtle_text),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        // Status
        let status = text(&browser.status_msg).size(12).style(style::subtle_text);

        // Sprite list items - fixed: no Fill on buttons inside scrollable
        let item_list: Vec<Element<'_, Message>> = browser
            .sprites
            .iter()
            .enumerate()
            .map(|(i, sprite)| {
                let is_selected = browser.selected_sprite_idx == Some(i);
                let btn = button(
                    row![
                        text(format!("{:04}", i)).size(11).width(40),
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
                .padding([8, 12])
                .on_press(Message::SpriteBrowserOpSelectSprite(i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        // Sequence selector buttons
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

        // Main image display - explicit fixed dimensions to avoid scrollable panic
        let main_display: Element<'_, Message> =
            if let Some(frame) = browser.frames.get(browser.selected_frame) {
                let img = image(frame.image.clone())
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(300.0));
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
                .width(Length::Shrink)
                .height(Length::Shrink)
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
                .width(Length::Shrink)
                .height(Length::Shrink)
                .padding(16)
                .into()
            };

        // Left panel: header + scrollable sprite list with fixed height
        let left_panel = column![
            container(
                row![
                    text("Sprites").size(13),
                    horizontal_space(),
                    text(format!("{} found", browser.sprites.len()))
                        .size(11)
                        .style(style::subtle_text),
                ]
                .padding([8, 12])
                .align_y(iced::Alignment::Center)
            )
            .style(style::grid_header_cell),
            scrollable(column(item_list).spacing(2)).height(Length::Fixed(400.0)),
        ]
        .width(Length::FillPortion(2));

        // Right panel: no scrollable, just vertical layout
        // All scrollable content should be in the left panel sprite list
        let right_content =
            column![sequence_row, frame_strip, horizontal_rule(1), main_display,].spacing(8);

        let right_panel = container(right_content)
            .padding(16)
            .width(Length::FillPortion(3))
            .style(style::info_card);

        // Main content
        let main_content = row![left_panel, right_panel]
            .spacing(12)
            .height(Length::Fill);

        column![header, controls, horizontal_rule(1), status, main_content,]
            .spacing(10)
            .padding(16)
            .height(Length::Fill)
            .into()
    }
}
