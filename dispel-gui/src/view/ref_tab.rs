use crate::app::App;
use crate::message::Message;
use crate::types::RefOp;
use crate::utils::labeled_file_row;
use iced::widget::{column, pick_list, row, text};
use iced::Element;

impl App {
    pub fn view_ref_tab(&self) -> Element<'_, Message> {
        let picker = pick_list(RefOp::ALL, self.state.ref_op, Message::RefOpSelected)
            .placeholder("Select…")
            .padding(8);
        column![
            row![text("Ref type:").size(13).width(140), picker]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            labeled_file_row(
                "Input file:",
                &self.state.ref_input,
                Message::RefInputChanged,
                Message::BrowseRefInput
            ),
        ]
        .spacing(12)
        .into()
    }
}
