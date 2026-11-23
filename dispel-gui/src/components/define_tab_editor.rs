/// Generates the state type, message enum and message routing for a tabbed
/// (multi-file) spreadsheet editor.
///
/// All five tabbed editors share the *same* catalog-load implementation: the
/// macro generates `LoadCatalog(PathBuf)` / `CatalogLoaded(tab_id, result)`
/// variants and routes them through
/// [`$crate::update::editor::tab::load_catalog`] and
/// [`$crate::update::editor::tab::catalog_loaded`].  The tab id is captured
/// when the async read is spawned and re-validated on completion, so switching
/// tabs mid-load cannot misdirect the catalog.
///
/// Generates:
/// - `pub type {Name}EditorState = MultiFileEditorState<T>`
/// - `pub enum {Name}EditorMessage` (standard variants + catalog load)
/// - `pub fn handle(msg, app) -> Task<Message>`
/// - `pub fn handle_core(msg, app, tab_id) -> Task<Message>`
///
/// An optional `extension` block adds one extra enum variant and routes it to
/// a caller-supplied handler.  Use it for editor-specific messages such as
/// monster/NPC name lookups; the catalog-load flow itself is never duplicated.
///
/// # Example
/// ```ignore
/// define_tab_editor! {
///     name: extra_ref,
///     name_pascal: ExtraRef,
///     record: ExtraRef,
///     field: extra_ref_editor,
///     loaded_prefix: "Extra ref catalog loaded: ",
///     error_prefix: "Error loading extra ref catalog: ",
///     save_success_msg: "Extra refs saved successfully.",
///     save_error_msg: "Error saving extra refs",
///     extension: {                       // optional
///         message: MyExtraMsg,
///         handle: handle_extra,
///         after_load: after_load,
///     },
/// }
/// ```
#[macro_export]
macro_rules! define_tab_editor {
    (
        name: $name:ident,
        name_pascal: $Name:ident,
        record: $Record:ty,
        field: $field:ident,
        loaded_prefix: $loaded_prefix:expr,
        error_prefix: $error_prefix:expr,
        save_success_msg: $save_success_msg:expr,
        save_error_msg: $save_error_msg:expr,
        $( extension: {
            message: $Ext:path,
            handle: $ext_handle:path,
            after_load: $after_load:path,
        } )?
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
                /// Start a catalog read. Carries the path so the async result
                /// can be tied back to the tab that requested it.
                LoadCatalog(std::path::PathBuf),
                /// Completion of a catalog read. The tab id is captured when
                /// the task is spawned, never read from the active tab here.
                CatalogLoaded(usize, Result<Vec<$Record>, String>),
                $( Extension($Ext), )?
            }

            // ── handle ──
            //
            // Routes the catalog-load variants through the single shared
            // implementation, lets an editor extension handle its own extra
            // messages, and delegates everything else to `handle_core`.
            pub fn handle(
                msg: [<$Name EditorMessage>],
                app: &mut $crate::app::App,
            ) -> ::iced::Task<$crate::message::Message> {
                use $crate::message::MessageExt;

                let tab_id = $crate::update::editor::tab::get_tab_id(&app.state.workspace)
                    .as_usize();

                match msg {
                    [<$Name EditorMessage>]::LoadCatalog(path) => {
                        let task = $crate::update::editor::tab::load_catalog(
                            path,
                            &mut app.state.editors.$field,
                            tab_id,
                            &app.state.shared_game_path,
                            &mut app.state.lookups,
                            |id, result| $crate::message::Message::$name(
                                [<$Name EditorMessage>]::CatalogLoaded(id, result)
                            ),
                        );
                        $(
                            let task = task.chain($after_load(app));
                        )?
                        task
                    }
                    [<$Name EditorMessage>]::CatalogLoaded(id, result) => {
                        $crate::update::editor::tab::catalog_loaded(
                            id,
                            result,
                            &mut app.state.editors.$field,
                            &app.state.lookups,
                            $loaded_prefix,
                            $error_prefix,
                        );
                        ::iced::Task::none()
                    }
                    $(
                        [<$Name EditorMessage>]::Extension(ext) => $ext_handle(ext, app, tab_id),
                    )?
                    other => handle_core(other, app, tab_id),
                }
            }

            // ── handle_core ──
            //
            // Standard editor interactions (selection, field edits, saving,
            // pane layout, spreadsheet messages). An unrecognised variant is
            // ignored rather than panicking — a message must never be able to
            // crash the UI.
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
                        app.state.workspace.mark_modified();
                        let observe = match captured {
                            Some(ctx)
                                if ctx.old != new_value =>
                            {
                                $crate::editors::mod_packager::recording::observe_field_edit(
                                    app, ctx, new_value,
                                )
                            }
                            _ => ::iced::Task::none(),
                        };
                        observe.chain(task)
                    }
                    [<$Name EditorMessage>]::Save => {
                        let result = $crate::update::editor::tab::save(
                            &mut app.state.editors.$field, tab_id,
                            $save_success_msg, $save_error_msg,
                        );
                        ::iced::Task::done($crate::message::Message::$name(
                            [<$Name EditorMessage>]::Saved(result),
                        ))
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
                    [<$Name EditorMessage>]::Saved(result) => {
                        if result.is_ok() {
                            app.state.workspace.clear_modified();
                        }
                        ::iced::Task::none()
                    }
                    // LoadCatalog / CatalogLoaded / Extension are handled by
                    // `handle`; anything else is a no-op, never a panic.
                    _ => ::iced::Task::none(),
                }
            }
        }
    };
}
