use crate::app::App;
use crate::editors::event_scr::message::EventScrEditorMessage;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

pub fn view(_app: &App) -> Element<'_, EventScrEditorMessage> {
    let sections = ["VAR", "MAP", "CHR", "NPC", "SPR", "WAV", "ACT"];
    
    let tabs = row![
        button("VAR").on_press(EventScrEditorMessage::SectionChanged("VAR".to_string())),
        button("MAP").on_press(EventScrEditorMessage::SectionChanged("MAP".to_string())),
        button("CHR").on_press(EventScrEditorMessage::SectionChanged("CHR".to_string())),
        button("NPC").on_press(EventScrEditorMessage::SectionChanged("NPC".to_string())),
        button("SPR").on_press(EventScrEditorMessage::SectionChanged("SPR".to_string())),
        button("WAV").on_press(EventScrEditorMessage::SectionChanged("WAV".to_string())),
        button("ACT").on_press(EventScrEditorMessage::SectionChanged("ACT".to_string())),
    ].spacing(5);
    
    column![
        text("EventScript Editor").size(20),
        tabs,
        container(text("Section content area").size(16))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ]
    .spacing(10)
    .into()
}
