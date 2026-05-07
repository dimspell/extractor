use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length};

use dispel_core::modding::{Conflict, ConflictKind, FieldKey};

use crate::app::App;
use crate::components::utils::horizontal_space;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

pub fn view(app: &App) -> Element<'_, Message> {
    let state = &app.state.mod_packager_editor;

    if state.workspace_root.is_none() {
        return container(text("Open a workspace to see conflicts.").size(12))
            .padding(20)
            .into();
    }

    if state.conflicts.is_empty() {
        return container(
            column![
                text("No conflicts.").size(14),
                text("Enable two or more mods that touch the same field, file, or sprite to surface them here.")
                    .size(12),
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .padding(40)
        .center_x(Fill)
        .center_y(Fill)
        .into();
    }

    let (soft, hard): (Vec<&Conflict>, Vec<&Conflict>) =
        state.conflicts.iter().partition(|c| !c.is_hard());

    let mut col = column![header_summary(soft.len(), hard.len())]
        .spacing(12)
        .padding(4)
        .width(Fill);

    if !hard.is_empty() {
        col = col.push(text("Hard conflicts (load-order only)").size(13));
        for c in &hard {
            col = col.push(conflict_card(c));
        }
    }
    if !soft.is_empty() {
        col = col.push(text("Field conflicts").size(13));
        for c in &soft {
            col = col.push(conflict_card(c));
        }
    }

    scrollable(col).height(Length::Fill).width(Fill).into()
}

fn header_summary(soft: usize, hard: usize) -> Element<'static, Message> {
    text(format!(
        "{} field conflict(s), {} hard conflict(s). Resolve by reordering load order in the Library tab.",
        soft, hard
    ))
    .size(12)
    .into()
}

fn conflict_card(c: &Conflict) -> Element<'_, Message> {
    let title = match &c.kind {
        ConflictKind::Field { record_id, field } => {
            format!("{}  •  record #{} field `{}`", c.file_path, record_id, field)
        }
        ConflictKind::Binary => format!("{}  •  binary delta overlap", c.file_path),
        ConflictKind::FileWhole => format!("{}  •  whole-file overlap", c.file_path),
    };
    let winner = c.winner().to_owned();
    let winner_label = if c.pinned_to.as_deref() == Some(winner.as_str()) {
        format!("📌 pinned to {winner}")
    } else {
        format!("winner: {winner}")
    };

    let mut rows = column![].spacing(2);
    for (i, p) in c.participants.iter().enumerate() {
        let is_winner = p.mod_id == winner;
        let marker = if is_winner { "▶" } else { " " };
        let value = match &p.field_new {
            Some(v) => format!("{:?}", v),
            None => format!("({})", p.op),
        };
        let line = format!("  {} [{}] {} → {}", marker, i, p.mod_id, value);
        rows = rows.push(text(line).size(11));
    }

    // Per-field pin controls only apply to soft (Field) conflicts.
    let action_row: Element<'_, Message> = match &c.kind {
        ConflictKind::Field { record_id, field } => {
            let key = FieldKey {
                file_path: c.file_path.clone(),
                record_id: *record_id,
                field: field.clone(),
            };
            let mut buttons = row![].spacing(6).align_y(Alignment::Center);
            for p in &c.participants {
                let pinned_here = c.pinned_to.as_deref() == Some(p.mod_id.as_str());
                let label = if pinned_here {
                    format!("✓ {}", p.mod_id)
                } else {
                    format!("Pin {}", p.mod_id)
                };
                let mut btn = button(text(label).size(11)).padding([2, 8]);
                if !pinned_here {
                    btn = btn.on_press(Message::mod_packager(
                        ModPackagerMessage::PinConflict {
                            key: key.clone(),
                            mod_slug: p.mod_id.clone(),
                        },
                    ));
                }
                buttons = buttons.push(btn);
            }
            if c.pinned_to.is_some() {
                buttons = buttons.push(
                    button(text("Unpin").size(11))
                        .padding([2, 8])
                        .style(button::secondary)
                        .on_press(Message::mod_packager(
                            ModPackagerMessage::UnpinConflict { key: key.clone() },
                        )),
                );
            }
            buttons.into()
        }
        ConflictKind::Binary | ConflictKind::FileWhole => {
            text("Reorder load order in the Library tab to resolve.")
                .size(11)
                .into()
        }
    };

    container(
        column![
            row![
                text(title).size(12),
                horizontal_space().width(Length::Fill),
                text(winner_label).size(11),
            ]
            .align_y(Alignment::Center),
            rows,
            action_row,
        ]
        .spacing(6),
    )
    .padding(8)
    .style(container::bordered_box)
    .into()
}
