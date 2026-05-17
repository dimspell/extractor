/// Generates the boilerplate for a "standard" single-file catalog editor.
///
/// A standard editor is one whose state is `StandardEditor<T>` (which bundles
/// `GenericEditorState<T>` + `SpreadsheetState`) and whose message is
/// `StandardEditorMessage<T>` — i.e. the data lives in one file, rendered in a
/// spreadsheet, with no per-tab multiplicity.
///
/// Generates:
/// - `pub type {Name}EditorState = StandardEditor<T>`
/// - `pub type {Name}EditorMessage = StandardEditorMessage<T>`
/// - `pub fn handle(msg, app) -> Task<Message>`
/// - `pub fn view(app) -> Element<Message>`
///
/// The caller still needs to:
/// - Define `component.rs` with the `EditableRecord` impl for `T`
/// - Add the `EditorMessage::{Variant}` arm + `MessageExt::{name}` shortcut
/// - Add the `{field}` field to `AppState` with type `Box<{Name}EditorState>`
/// - Wire the dispatch in `update/editor/mod.rs` and `view/mod.rs`
///
/// # Example
/// ```ignore
/// define_standard_editor! {
///     name: weapon,
///     name_pascal: Weapon,
///     record: dispel_core::WeaponItem,
///     field: weapon_editor,
///     file: "CharacterInGame/weaponItem.db",
/// }
/// ```
#[macro_export]
macro_rules! define_standard_editor {
    (
        name: $name:ident,
        name_pascal: $Name:ident,
        record: $Record:path,
        field: $field:ident,
        file: $file:expr $(,)?
    ) => {
        ::paste::paste! {
            pub type [<$Name EditorState>] =
                $crate::components::standard::state::StandardEditor<$Record>;

            pub type [<$Name EditorMessage>] =
                $crate::components::standard::message::StandardEditorMessage<$Record>;

            pub fn handle(
                msg: [<$Name EditorMessage>],
                app: &mut $crate::app::App,
            ) -> ::iced::Task<$crate::message::Message> {
                use $crate::message::MessageExt;
                match msg {
                    [<$Name EditorMessage>]::Spreadsheet(sm) => {
                        $crate::handle_spreadsheet_messages!(
                            app,
                            $field,
                            |index, field, value| $crate::message::Message::$name(
                                [<$Name EditorMessage>]::FieldChanged(index, field, value)
                            ),
                            sm
                        );
                        ::iced::Task::none()
                    }
                    [<$Name EditorMessage>]::FieldChanged(idx, field, value) => {
                        // idx is the catalog index — search filtered by matching it.
                        let (old_value, orig_idx_u32) = app
                            .state
                            .$field
                            .state
                            .filtered
                            .iter()
                            .find(|(i, _)| *i == idx)
                            .map(|(i, r)| {
                                use $crate::components::editable::EditableRecord;
                                (r.get_field(&field), *i as u32)
                            })
                            .unwrap_or_default();
                        let new_value = value.clone();
                        let task = $crate::components::standard::update::handle(
                            [<$Name EditorMessage>]::FieldChanged(idx, field.clone(), value),
                            &mut app.state.$field,
                            &app.state.shared_game_path.clone(),
                            $file,
                            $crate::message::Message::$name,
                        );
                        // Only record if the edit actually changed the value
                        // (avoids no-op observations).
                        let observe = if old_value != new_value {
                            $crate::editors::mod_packager::recording::observe_field_change(
                                app, $file, orig_idx_u32, &field, old_value, new_value,
                            )
                        } else {
                            ::iced::Task::none()
                        };
                        observe.chain(task)
                    }
                    msg => $crate::components::standard::update::handle(
                        msg,
                        &mut app.state.$field,
                        &app.state.shared_game_path.clone(),
                        $file,
                        $crate::message::Message::$name,
                    ),
                }
            }

            pub fn view(app: &$crate::app::App) -> ::iced::Element<'_, $crate::message::Message> {
                use $crate::message::MessageExt;
                $crate::view::editor::view_spreadsheet(
                    &app.state.$field.state,
                    &app.state.$field.spreadsheet,
                    $crate::message::Message::$name([<$Name EditorMessage>]::LoadCatalog),
                    $crate::message::Message::$name([<$Name EditorMessage>]::Save),
                    |idx| $crate::message::Message::$name([<$Name EditorMessage>]::Select(idx)),
                    |idx, field, val| {
                        $crate::message::Message::$name(
                            [<$Name EditorMessage>]::FieldChanged(idx, field, val),
                        )
                    },
                    |msg| $crate::message::Message::$name([<$Name EditorMessage>]::Spreadsheet(msg)),
                    &app.state.lookups,
                    |event| $crate::message::Message::$name([<$Name EditorMessage>]::PaneResized(event)),
                    |pane| $crate::message::Message::$name([<$Name EditorMessage>]::PaneClicked(pane)),
                )
            }
        }
    };
}
