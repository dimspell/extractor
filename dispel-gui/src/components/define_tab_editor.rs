/// Generates state type alias, message enum, and `handle_core` for a tab editor.
///
/// Always creates: `AddEntry`, `RemoveEntry(usize)`, `Saved(Result<(), String>)` in
/// the message enum and matching arms in `handle_core`. Unused variants are harmless.
///
/// The caller still writes `component.rs`, `mod.rs`, `view.rs`, and a thin
/// `handle()` wrapper in their `mod.rs`.
///
/// # Example
/// ```ignore
/// define_tab_editor! {
///     name: extra_ref,
///     name_pascal: ExtraRef,
///     record: ExtraRef,
///     field: extra_ref_editor,
///     empty_text: "Extra ref file not loaded",
///     save_success_msg: "Extra refs saved successfully.",
///     save_error_msg: "Error saving extra refs",
///     extra_variants: {
///         NpcNamesLoaded(Result<Vec<(String, String)>, String>),
///     },
/// }
/// ```
///
/// `extra_variants` is optional — extra enum variants injected as-is.
#[macro_export]
macro_rules! define_tab_editor {
    (
        name: $name:ident,
        name_pascal: $Name:ident,
        record: $Record:ty,
        field: $field:ident,
        empty_text: $empty_text:expr,
        save_success_msg: $save_success_msg:expr,
        save_error_msg: $save_error_msg:expr,
        $(extra_variants: { $($extra_variants:tt)* })?
        $(,)?
    ) => {
        ::paste::paste! {

            // ── State ──
            pub type [<$Name EditorState>] =
                $crate::components::generic_editor::MultiFileEditorState<$Record>;

            // ── Message ──
            #[derive(Debug, Clone)]
            pub enum [<$Name EditorMessage>] {
                Select(usize),
                FieldChanged(usize, String, String),
                Spreadsheet($crate::view::editor::SpreadsheetMessage),
                PaneResized(iced::widget::pane_grid::ResizeEvent),
                PaneClicked(iced::widget::pane_grid::Pane),
                Save,
                AddEntry,
                RemoveEntry(usize),
                Saved(Result<(), String>),
                $($($extra_variants)*)?
            }

            // ── handle_core ──
            pub fn handle_core(
                msg: [<$Name EditorMessage>],
                app: &mut $crate::app::App,
                tab_id: usize,
            ) -> ::iced::Task<$crate::message::Message> {
                use $crate::message::MessageExt;

                match msg {
                    [<$Name EditorMessage>]::Select(index) => {
                        $crate::update::editor::tab::select(
                            &mut app.state.editors.$field, tab_id, index,
                        )
                    }
                    [<$Name EditorMessage>]::FieldChanged(index, field, value) => {
                        let captured = $crate::editors::mod_packager::recording::capture_field_recording_context(
                            app.state.editors.$field.editors.get(&tab_id),
                            index, &field, &app.state.shared_game_path,
                        );
                        let new_value = value.clone();
                        let task = $crate::update::editor::tab::field_changed(
                            &mut app.state.editors.$field, tab_id, index, field.clone(), value,
                        );
                        let observe = match captured {
                            Some((old_value, orig_idx, file_path)) if old_value != new_value => {
                                $crate::editors::mod_packager::recording::observe_field_change(
                                    app, file_path, orig_idx, &field, old_value, new_value,
                                )
                            }
                            _ => ::iced::Task::none(),
                        };
                        observe.chain(task)
                    }
                    [<$Name EditorMessage>]::Save => {
                        $crate::update::editor::tab::save(
                            &mut app.state.editors.$field, tab_id,
                            $save_success_msg, $save_error_msg,
                        )
                    }
                    [<$Name EditorMessage>]::Spreadsheet(msg) => {
                        $crate::handle_spreadsheet_messages_tab!(
                            app,
                            $field,
                            &tab_id,
                            |index, field, value| $crate::message::Message::$name(
                                [<$Name EditorMessage>]::FieldChanged(index, field, value)
                            ),
                            msg
                        );
                        ::iced::Task::none()
                    }
                    [<$Name EditorMessage>]::PaneResized(event) => {
                        $crate::update::editor::tab::pane_resized(
                            &mut app.state.editors.$field, tab_id, event,
                        )
                    }
                    [<$Name EditorMessage>]::PaneClicked(pane) => {
                        $crate::update::editor::tab::pane_clicked(
                            &mut app.state.editors.$field, tab_id, pane,
                        )
                    }
                    [<$Name EditorMessage>]::AddEntry => {
                        $crate::update::editor::tab::add_entry(
                            &mut app.state.editors.$field, tab_id,
                        )
                    }
                    [<$Name EditorMessage>]::RemoveEntry(index) => {
                        $crate::update::editor::tab::remove_entry(
                            &mut app.state.editors.$field, tab_id, index,
                        )
                    }
                    [<$Name EditorMessage>]::Saved(_) => {
                        ::iced::Task::none()
                    }
                    _ => ::core::panic!("handle_core: unhandled message variant"),
                }
            }
        }
    };
}
