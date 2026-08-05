use crate::editors::save_file_viewer::helpers::{label_row, section_header};
use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;
use dispel_core::references::save_file::PartyMember;
use iced::widget::{container, scrollable, text, Column};
use iced::Element;

/// Player party member section
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    let identity = &sf.character_identity;
    let party_members = &sf.character_identity.party_members;

    scrollable(
        Column::new()
            .push(section_header("Player Identity"))
            .push(label_row(
                "Party Members Count",
                &identity.party_members_count.to_string(),
            ))
            .push(Column::new().spacing(4).push(match party_members.get(0) {
                Some(member) => party_member_block(member),
                None => text("No party member in slot 1").into(),
            }))
            .push(Column::new().spacing(4).push(match party_members.get(1) {
                Some(member) => party_member_block(member),
                None => text("No party member in slot 2").into(),
            }))
            .spacing(8)
            .padding(16),
    )
    .into()
}

fn party_member_block(member: &PartyMember) -> Element<'static, Message> {
    container(text(member.name.to_string()).size(11))
        .padding(8)
        .into()
}
