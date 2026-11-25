//! Right-hand inspector pane — one input widget per field of the
//! currently-selected record.

use std::collections::HashMap;

use iced::widget::{
    Column, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Element, Fill, Length};

use crate::components::composite_item::composite_item_picker;
use crate::components::editable::{EditableRecord, FieldDescriptor, FieldKind};
use crate::components::generic_editor::GenericEditorState;
use crate::components::utils::horizontal_space;
use crate::message::Message;
use crate::style;
use crate::view::editor::spreadsheet::message::SpreadsheetMessage;
use crate::view::editor::spreadsheet::state::SpreadsheetState;
use gui_widgets::lucide::{LUCIDE_FONT, icon_char};
use gui_widgets::{TextAreaContent, textarea};
use lucide_icons::Icon;

pub fn build_inspector_panel<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    field_changed_msg: fn(usize, String, String) -> Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let descriptors = R::field_descriptors();

    let header = container(
        row![
            text("Inspector").size(13),
            horizontal_space(),
            button(text(icon_char(Icon::X)).font(LUCIDE_FONT).size(11))
                .on_press(spreadsheet_msg(SpreadsheetMessage::CloseInspector))
                .style(style::browse_button)
                .padding([2, 6]),
        ]
        .align_y(iced::Alignment::Center)
        .padding([6, 10]),
    )
    .width(Fill)
    .style(style::spreadsheet_header);

    let mut fields: Column<Message> = column![].spacing(6).padding([8, 12]);

    if let Some(orig_idx) = spreadsheet.selected_orig {
        if let Some(record) = editor.catalog.as_ref().and_then(|c| c.get(orig_idx)) {
            for desc in descriptors.iter() {
                let value = record.get_field(desc.name);
                let validation_error = record.validate_field(desc.name, &value);
                fields = fields.push(build_inspector_field(
                    desc,
                    value,
                    orig_idx,
                    lookups,
                    validation_error,
                    field_changed_msg,
                    &spreadsheet.inspector_textarea_contents,
                    spreadsheet_msg,
                ));
            }
        }
    } else {
        fields = fields.push(
            text("Click a row to inspect")
                .size(12)
                .style(style::subtle_text),
        );
    }

    let scroll = scrollable(fields).height(Length::Fill);

    container(column![header, scroll])
        .width(Fill)
        .height(Fill)
        .style(style::sidebar_container)
        .into()
}

#[allow(clippy::too_many_arguments)]
fn build_inspector_field<'a>(
    descriptor: &'a FieldDescriptor,
    value: String,
    orig_idx: usize,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    validation_error: Option<String>,
    field_changed_msg: fn(usize, String, String) -> Message,
    textarea_contents: &'a HashMap<String, TextAreaContent>,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let label = text(descriptor.label).size(11).style(style::subtle_text);

    let body: Element<'a, Message> = match &descriptor.kind {
        FieldKind::TextArea => {
            let field_name = descriptor.name.to_string();
            if let Some(tc) = textarea_contents.get(descriptor.name) {
                textarea(&tc.0, move |action| {
                    spreadsheet_msg(SpreadsheetMessage::TextAreaChanged(
                        orig_idx,
                        field_name.clone(),
                        action,
                    ))
                })
            } else {
                text_input("", value.clone())
                    .on_input(move |v| field_changed_msg(orig_idx, field_name.clone(), v))
                    .padding(4)
                    .size(11)
                    .width(Length::Fill)
                    .into()
            }
        }
        FieldKind::Lookup(key) => {
            if let Some(options) = lookups.get(*key) {
                let field_name = descriptor.name.to_string();
                let selected = options
                    .iter()
                    .find(|(id, _)| id == &value)
                    .map(|(_, name)| name.clone());

                let options_vec: Vec<String> =
                    options.iter().map(|(_, name)| name.clone()).collect();

                // Build name→id lookup for the closure
                let name_to_id: HashMap<String, String> = options
                    .iter()
                    .map(|(id, name)| (name.clone(), id.clone()))
                    .collect();
                pick_list(selected, options_vec, String::clone)
                    .on_select(move |selected_name| {
                        let selected_id = name_to_id
                            .get(selected_name.as_str())
                            .cloned()
                            .unwrap_or_default();
                        spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                            orig_idx,
                            field_name.clone(),
                            selected_id,
                        ))
                    })
                    .width(Length::Fill)
                    .into()
            } else {
                let field_name = descriptor.name.to_string();
                text_input("", value.clone())
                    .on_input(move |v| field_changed_msg(orig_idx, field_name.clone(), v))
                    .padding(4)
                    .size(11)
                    .width(Length::Fill)
                    .into()
            }
        }
        FieldKind::Enum { variants } => {
            let field_name = descriptor.name.to_string();
            let selected = variants.iter().find(|&&v| v == value).copied();
            pick_list(selected, *variants, |v| v.to_string())
                .on_select(move |selected_variant| {
                    spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                        orig_idx,
                        field_name.clone(),
                        selected_variant.to_string(),
                    ))
                })
                .width(Length::Fill)
                .into()
        }
        FieldKind::CompositeItem { lookup_key } => {
            let entries = lookups.get(*lookup_key).map(|v| v.as_slice());
            let field_name = descriptor.name.to_string();
            composite_item_picker(descriptor.label, &value, entries, false, move |v| {
                spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                    orig_idx,
                    field_name.clone(),
                    v,
                ))
            })
        }
        _ => {
            let field_name = descriptor.name.to_string();
            text_input("", value.clone())
                .on_input(move |v| {
                    spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                        orig_idx,
                        field_name.clone(),
                        v,
                    ))
                })
                .padding(4)
                .size(11)
                .width(Length::Fill)
                .into()
        }
    };

    let mut field_column = column![label, body].spacing(4);
    if let Some(err) = validation_error {
        field_column =
            field_column.push(
                text(err)
                    .size(10)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(iced::color!(0xff5252)),
                    }),
            );
    }
    field_column.into()
}

#[cfg(all(test, feature = "iced_test"))]
mod tests {
    use super::*;
    use iced_test::selector::{Candidate, Target};
    use iced_test::simulator;

    fn field_changed(idx: usize, name: String, value: String) -> Message {
        Message::Editor(crate::message::editor::EditorMessage::Weapon(
            crate::editors::weapon::WeaponEditorMessage::FieldChanged(idx, name, value),
        ))
    }

    fn spreadsheet_msg(_: SpreadsheetMessage) -> Message {
        Message::System(crate::message::SystemMessage::DismissError)
    }

    fn count_label_text(element: Element<'_, Message>, label: &str) -> usize {
        let mut ui = simulator(element);
        let mut count = 0usize;
        let mut selector = |candidate: Candidate<'_>| -> Option<Target> {
            if let Candidate::Text { content, .. } = candidate
                && content == label
            {
                count += 1;
            }
            None
        };
        let _ = ui.find(&mut selector);
        count
    }

    #[test]
    fn test_composite_field_label_appears_once() {
        let descriptor = FieldDescriptor {
            name: "item_type",
            label: "Invented Item:",
            kind: FieldKind::CompositeItem {
                lookup_key: "invented_items",
            },
        };
        let empty_lookups = HashMap::new();
        let contents = HashMap::new();

        // Fallback path (no lookups): the inspector wraps the body with one label.
        let element = build_inspector_field(
            &descriptor,
            "1:0".into(),
            0,
            &empty_lookups,
            None,
            field_changed,
            &contents,
            spreadsheet_msg,
        );
        assert_eq!(
            count_label_text(element, "Invented Item:"),
            1,
            "composite field must not render its label twice without lookups"
        );

        // Picker path (lookups loaded): same single-label guarantee.
        let lookups = HashMap::from([(
            "invented_items".to_string(),
            vec![("1:0".to_string(), "Invented Sword".to_string())],
        )]);
        let element = build_inspector_field(
            &descriptor,
            "1:0".into(),
            0,
            &lookups,
            None,
            field_changed,
            &contents,
            spreadsheet_msg,
        );
        assert_eq!(
            count_label_text(element, "Invented Item:"),
            1,
            "composite field must not render its label twice with lookups"
        );
    }

    #[test]
    fn test_npc_facing_directions_offer_left_in_inspector() {
        use crate::components::editable::EditableRecord;
        use dispel_core::{EventNpcRef, NPC, references::enums::NpcLookingDirection};

        let _ = NpcLookingDirection::Left;

        fn check<R: EditableRecord>(field: &'static str) {
            let mut record = R::default();
            assert!(
                record.set_field(field, "Left".to_string()),
                "{field} must accept Left"
            );
            assert_eq!(record.get_field(field), "Left", "{field} must display Left");
            let desc = R::field_descriptors()
                .iter()
                .find(|d| d.name == field)
                .expect("descriptor exists");
            let FieldKind::Enum { variants } = &desc.kind else {
                panic!("{field} must be an enum");
            };
            assert!(
                variants.contains(&"Left"),
                "{field} descriptor must offer Left"
            );

            let value = record.get_field(field);
            let contents = HashMap::new();
            let lookups = HashMap::new();
            let element = build_inspector_field(
                desc,
                value,
                0,
                &lookups,
                None,
                field_changed,
                &contents,
                spreadsheet_msg,
            );
            assert!(
                crate::tests::common::select_shows(element, "Left"),
                "inspector for {field} must render the selected Left label"
            );
        }

        for field in [
            "waypoint1_facing_direction",
            "waypoint2_facing_direction",
            "waypoint3_facing_direction",
            "waypoint4_facing_direction",
        ] {
            check::<NPC>(field);
            check::<EventNpcRef>(field);
        }
    }

    #[test]
    fn test_optional_enum_uses_display_names() {
        use crate::components::editable::EditableRecord;
        use dispel_core::DialogueScript;

        let mut record = DialogueScript::default();
        assert!(record.set_field("dialog_type", "Choice".to_string()));
        assert_eq!(record.get_field("dialog_type"), "Choice");
        assert!(record.set_field("dialog_owner", "Npc".to_string()));
        assert_eq!(record.get_field("dialog_owner"), "Npc");

        for name in ["dialog_type", "dialog_owner"] {
            let desc = DialogueScript::field_descriptors()
                .iter()
                .find(|d| d.name == name)
                .expect("descriptor exists");
            let value = record.get_field(name);
            let lookups = HashMap::new();
            let contents = HashMap::new();
            let element = build_inspector_field(
                desc,
                value.clone(),
                0,
                &lookups,
                None,
                field_changed,
                &contents,
                spreadsheet_msg,
            );
            assert!(
                crate::tests::common::select_shows(element, &value),
                "optional enum {name} must render its display name {value}"
            );
        }
    }

    #[test]
    fn test_lookup_fallback_input_emits_field_changed() {
        let descriptor = FieldDescriptor {
            name: "monster_db_id",
            label: "Monster ID:",
            kind: FieldKind::Lookup("monster_names"),
        };
        let lookups = HashMap::new();
        let contents = HashMap::new();
        let element: Element<'_, Message> = build_inspector_field(
            &descriptor,
            String::new(),
            0,
            &lookups,
            None,
            field_changed,
            &contents,
            spreadsheet_msg,
        );
        let mut ui = simulator(element);
        let mut pick_input = |candidate: Candidate<'_>| match candidate {
            Candidate::TextInput { .. } => Some(Target::from(candidate)),
            _ => None,
        };
        ui.click(&mut pick_input)
            .expect("fallback text input must be present");
        ui.typewrite("42");
        let messages: Vec<Message> = ui.into_messages().collect();
        assert!(
            messages.iter().any(|m| matches!(
                m,
                Message::Editor(crate::message::editor::EditorMessage::Weapon(
                    crate::editors::weapon::WeaponEditorMessage::FieldChanged(_, _, _)
                ))
            )),
            "fallback text input must emit a FieldChanged message, got {messages:?}"
        );
    }
}
