/// Generates the boilerplate for a "standard" single-file catalog editor.
///
/// A standard editor is one whose state is `GenericEditorState<T>` and whose
/// message is `StandardEditorMessage<T>` — i.e. the data lives in one file,
/// rendered in a spreadsheet, with no per-tab multiplicity.
///
/// Generates:
/// - `pub type {Name}EditorState = GenericEditorState<T>`
/// - `pub type {Name}EditorMessage = StandardEditorMessage<T>`
/// - `pub fn handle(msg, app) -> Task<Message>`
/// - `pub fn view(app) -> Element<Message>`
///
/// The caller still needs to:
/// - Define `component.rs` with the `EditableRecord` impl for `T`
/// - Add the `EditorMessage::{Variant}` arm + `MessageExt::{name}` shortcut
/// - Add the `{name}_editor` and `{name}_spreadsheet` fields to `AppState`
/// - Wire the dispatch in `update/editor/mod.rs` and `view/mod.rs`
///
/// # Example
/// ```ignore
/// define_standard_editor! {
///     name: weapon,
///     name_pascal: Weapon,
///     record: dispel_core::WeaponItem,
///     state_field: weapon_editor,
///     sheet_field: weapon_spreadsheet,
///     file: "CharacterInGame/weaponItem.db",
/// }
/// ```
#[macro_export]
macro_rules! define_standard_editor {
    (
        name: $name:ident,
        name_pascal: $Name:ident,
        record: $Record:path,
        state_field: $state_field:ident,
        sheet_field: $sheet_field:ident,
        file: $file:expr $(,)?
    ) => {
        ::paste::paste! {
            pub type [<$Name EditorState>] =
                $crate::generic_editor::GenericEditorState<$Record>;

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
                            $sheet_field,
                            $state_field,
                            |index, field, value| $crate::message::Message::$name(
                                [<$Name EditorMessage>]::FieldChanged(index, field, value)
                            ),
                            sm
                        );
                        ::iced::Task::none()
                    }
                    msg => $crate::components::standard::update::handle(
                        msg,
                        &mut app.state.$state_field,
                        &mut app.state.$sheet_field,
                        &app.state.shared_game_path.clone(),
                        $file,
                        $crate::message::Message::$name,
                    ),
                }
            }

            pub fn view(app: &$crate::app::App) -> ::iced::Element<'_, $crate::message::Message> {
                use $crate::message::MessageExt;
                $crate::view::editor::view_spreadsheet(
                    &app.state.$state_field,
                    &app.state.$sheet_field,
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
