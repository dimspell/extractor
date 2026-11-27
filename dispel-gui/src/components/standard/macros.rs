/// Resolves the spreadsheet title: an explicit `title:` wins, otherwise the
/// editor name is converted to a human-readable label.
#[macro_export]
macro_rules! standard_title {
    ($name:ident) => {
        concat!(stringify!($name), " editor")
    };
    ($name:ident, $title:expr) => {
        $title
    };
}

/// Generates the shared spreadsheet `view()` for a standard editor.
///
/// Kept separate so `define_standard_editor!` can reuse it for both the plain
/// and the extension-enabled variants without duplicating the view body.
#[macro_export]
macro_rules! define_standard_view {
    (
        name: $name:ident,
        field: $field:ident,
        message: $Msg:ty,
        title: $title:expr $(,)?
    ) => {
        pub fn view(app: &$crate::app::App) -> ::iced::Element<'_, $crate::message::Message> {
            use $crate::message::MessageExt;
            $crate::view::editor::view_spreadsheet(
                &app.state.editors.$field.state,
                &app.state.editors.$field.spreadsheet,
                $crate::message::Message::$name(<$Msg>::LoadCatalog),
                $crate::message::Message::$name(<$Msg>::Save),
                |idx| $crate::message::Message::$name(<$Msg>::Select(idx)),
                |idx, field, val| {
                    $crate::message::Message::$name(<$Msg>::FieldChanged(idx, field, val))
                },
                |msg| $crate::message::Message::$name(<$Msg>::Spreadsheet(msg)),
                &app.state.lookups,
                &app.state.composite_item_options.state,
                |event| $crate::message::Message::$name(<$Msg>::PaneResized(event)),
                |pane| $crate::message::Message::$name(<$Msg>::PaneClicked(pane)),
                None,
                None,
                $title,
            )
        }
    };
}

/// Generates the boilerplate for a "standard" single-file catalog editor.
///
/// A standard editor is one whose state is `StandardEditor<T>` (which bundles
/// `GenericEditorState<T>` + `SpreadsheetState`) and whose data lives in one
/// file, rendered in a spreadsheet.
///
/// Generates:
/// - `pub type {Name}EditorState = StandardEditor<T>`
/// - either a `type {Name}EditorMessage = StandardEditorMessage<T>` alias or,
///   with an `extension` block, a dedicated enum with an `Extension` variant
/// - `pub fn handle(msg, app) -> Task<Message>`
/// - `pub fn view(app) -> Element<Message>` (via `define_standard_view!`)
///
/// The caller still needs to:
/// - Define `component.rs` with the `EditableRecord` impl for `T`
/// - Add the `EditorMessage::{Variant}` arm + `MessageExt::{name}` shortcut
/// - Add the `{field}` field to `AppState` with type `Box<{Name}EditorState>`
/// - Wire the dispatch in `update/editor/mod.rs` and `view/mod.rs`
///
/// # Plain editor
/// ```ignore
/// define_standard_editor! {
///     name: weapon,
///     name_pascal: Weapon,
///     record: dispel_core::WeaponItem,
///     field: weapon_editor,
///     file: "CharacterInGame/weaponItem.db",
/// }
/// ```
///
/// # Editor with extra messages (no copied handler / view)
/// ```ignore
/// define_standard_editor! {
///     name: wave_ini,
///     name_pascal: WaveIni,
///     record: dispel_core::WaveIni,
///     field: wave_ini_editor,
///     file: "Wave.ini",
///     extension: {
///         message: WaveIniExtension,
///         handle: handle_extension,
///     },
/// }
/// ```
#[macro_export]
macro_rules! define_standard_editor {
    // ── Plain editor: message is a type alias for StandardEditorMessage<T> ──
    (
        name: $name:ident,
        name_pascal: $Name:ident,
        record: $Record:path,
        field: $field:ident,
        file: $file:expr
        $(, title: $title:expr)?
        $(,)?
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
                let tab_id = app.state.workspace.active_tab_id().as_usize();
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
                            .editors
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
                        let (task, applied) = $crate::components::standard::update::handle(
                            [<$Name EditorMessage>]::FieldChanged(idx, field.clone(), value),
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            $crate::message::Message::$name,
                        );
                        if applied {
                            app.state.workspace.mark_modified();
                        }
                        // Only record if the edit actually applied and changed
                        // the value (avoids no-op observations). The field
                        // identity is one canonical FieldKey, shared with the
                        // core conflict/pin/apply pipeline.
                        let context =
                            $crate::editors::mod_packager::recording::FieldEditContext {
                                old: old_value,
                                key: ::dispel_core::modding::FieldKey::new(
                                    $file, orig_idx_u32, &field,
                                ),
                            };
                        let observe = if applied && context.old != new_value {
                            $crate::editors::mod_packager::recording::observe_field_edit(
                                app, context, new_value,
                            )
                        } else {
                            ::iced::Task::none()
                        };
                        observe.chain(task)
                    }
                    [<$Name EditorMessage>]::Saved(saved_tab_id, result) => {
                        let succeeded = result.is_ok();
                        let (task, _) = $crate::components::standard::update::handle(
                            [<$Name EditorMessage>]::Saved(saved_tab_id, result),
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            saved_tab_id,
                            $crate::message::Message::$name,
                        );
                        if succeeded {
                            app.state.workspace.clear_modified_for(saved_tab_id);
                        }
                        task
                    }
                    [<$Name EditorMessage>]::LoadCatalog => {
                        // Ensure item lookups are populated for CompositeItem
                        // fields used in some standard editors.
                        $crate::components::item_catalog::ensure_item_lookups(
                            &app.state.shared_game_path,
                            &mut app.state.lookups,
                        );
                        let (task, _) = $crate::components::standard::update::handle(
                            msg,
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            $crate::message::Message::$name,
                        );
                        task
                    }
                    [<$Name EditorMessage>]::CatalogLoaded(result) => {
                        let succeeded = result.is_ok();
                        let (task, _) = $crate::components::standard::update::handle(
                            [<$Name EditorMessage>]::CatalogLoaded(result),
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            $crate::message::Message::$name,
                        );
                        if succeeded {
                            app.state.workspace.clear_modified_for(tab_id);
                        }
                        task
                    }
                    msg => {
                        let (task, _) = $crate::components::standard::update::handle(
                            msg,
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            $crate::message::Message::$name,
                        );
                        task
                    }
                }
            }

            $crate::define_standard_view! {
                name: $name,
                field: $field,
                message: [<$Name EditorMessage>],
                title: $crate::standard_title!($name $(, $title)?),
            }
        }
    };

    // ── Editor with an extension enum for extra messages ───────────────────
    (
        name: $name:ident,
        name_pascal: $Name:ident,
        record: $Record:path,
        field: $field:ident,
        file: $file:expr,
        extension: {
            message: $Ext:path,
            handle: $ext_handle:path,
        }
        $(, title: $title:expr)?
        $(,)?
    ) => {
        ::paste::paste! {
            pub type [<$Name EditorState>] =
                $crate::components::standard::state::StandardEditor<$Record>;

            #[derive(Debug, Clone)]
            pub enum [<$Name EditorMessage>] {
                LoadCatalog,
                CatalogLoaded(Result<Vec<$Record>, String>),
                Select(usize),
                FieldChanged(usize, String, String),
                Spreadsheet($crate::view::editor::SpreadsheetMessage),
                PaneResized(iced::widget::pane_grid::ResizeEvent),
                PaneClicked(iced::widget::pane_grid::Pane),
                Save,
                /// First field is the id of the tab that issued the save.
                Saved(usize, Result<(), String>),
                /// Editor-specific messages (e.g. WAV export).
                Extension($Ext),
            }

            /// Map a generic standard message back into this editor's message
            /// enum. Exhaustive over `StandardEditorMessage`, so no
            /// `unreachable!()` is needed.
            fn [<wrap_ $name _std>](
                msg: $crate::components::standard::message::StandardEditorMessage<$Record>,
            ) -> $crate::message::Message {
                use $crate::message::MessageExt;
                $crate::message::Message::$name(match msg {
                    $crate::components::standard::message::StandardEditorMessage::LoadCatalog =>
                        [<$Name EditorMessage>]::LoadCatalog,
                    $crate::components::standard::message::StandardEditorMessage::CatalogLoaded(r) =>
                        [<$Name EditorMessage>]::CatalogLoaded(r),
                    $crate::components::standard::message::StandardEditorMessage::Select(i) =>
                        [<$Name EditorMessage>]::Select(i),
                    $crate::components::standard::message::StandardEditorMessage::FieldChanged(i, f, v) =>
                        [<$Name EditorMessage>]::FieldChanged(i, f, v),
                    $crate::components::standard::message::StandardEditorMessage::Spreadsheet(s) =>
                        [<$Name EditorMessage>]::Spreadsheet(s),
                    $crate::components::standard::message::StandardEditorMessage::PaneResized(e) =>
                        [<$Name EditorMessage>]::PaneResized(e),
                    $crate::components::standard::message::StandardEditorMessage::PaneClicked(p) =>
                        [<$Name EditorMessage>]::PaneClicked(p),
                    $crate::components::standard::message::StandardEditorMessage::Save =>
                        [<$Name EditorMessage>]::Save,
                    $crate::components::standard::message::StandardEditorMessage::Saved(t, r) =>
                        [<$Name EditorMessage>]::Saved(t, r),
                })
            }

            pub fn handle(
                msg: [<$Name EditorMessage>],
                app: &mut $crate::app::App,
            ) -> ::iced::Task<$crate::message::Message> {
                use $crate::message::MessageExt;
                let tab_id = app.state.workspace.active_tab_id().as_usize();
                match msg {
                    [<$Name EditorMessage>]::Extension(ext) => $ext_handle(ext, app),
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
                        let (old_value, orig_idx_u32) = app
                            .state
                            .editors
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
                        let (task, applied) = $crate::components::standard::update::handle(
                            $crate::components::standard::message::StandardEditorMessage::FieldChanged(idx, field.clone(), value),
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            [<wrap_ $name _std>],
                        );
                        if applied {
                            app.state.workspace.mark_modified();
                        }
                        let context =
                            $crate::editors::mod_packager::recording::FieldEditContext {
                                old: old_value,
                                key: ::dispel_core::modding::FieldKey::new(
                                    $file, orig_idx_u32, &field,
                                ),
                            };
                        let observe = if applied && context.old != new_value {
                            $crate::editors::mod_packager::recording::observe_field_edit(
                                app, context, new_value,
                            )
                        } else {
                            ::iced::Task::none()
                        };
                        observe.chain(task)
                    }
                    [<$Name EditorMessage>]::Saved(saved_tab_id, result) => {
                        let succeeded = result.is_ok();
                        let (task, _) = $crate::components::standard::update::handle(
                            $crate::components::standard::message::StandardEditorMessage::Saved(saved_tab_id, result),
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            saved_tab_id,
                            [<wrap_ $name _std>],
                        );
                        if succeeded {
                            app.state.workspace.clear_modified_for(saved_tab_id);
                        }
                        task
                    }
                    [<$Name EditorMessage>]::LoadCatalog => {
                        $crate::components::item_catalog::ensure_item_lookups(
                            &app.state.shared_game_path,
                            &mut app.state.lookups,
                        );
                        let (task, _) = $crate::components::standard::update::handle(
                            $crate::components::standard::message::StandardEditorMessage::LoadCatalog,
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            [<wrap_ $name _std>],
                        );
                        task
                    }
                    [<$Name EditorMessage>]::CatalogLoaded(result) => {
                        let succeeded = result.is_ok();
                        let (task, _) = $crate::components::standard::update::handle(
                            $crate::components::standard::message::StandardEditorMessage::CatalogLoaded(result),
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            [<wrap_ $name _std>],
                        );
                        if succeeded {
                            app.state.workspace.clear_modified_for(tab_id);
                        }
                        task
                    }
                    other => {
                        use $crate::components::standard::message::StandardEditorMessage as Std;
                        let std = match other {
                            [<$Name EditorMessage>]::Select(i) => Std::Select(i),
                            [<$Name EditorMessage>]::CatalogLoaded(r) => Std::CatalogLoaded(r),
                            [<$Name EditorMessage>]::PaneResized(e) => Std::PaneResized(e),
                            [<$Name EditorMessage>]::PaneClicked(p) => Std::PaneClicked(p),
                            [<$Name EditorMessage>]::Save => Std::Save,
                            [<$Name EditorMessage>]::Saved(t, r) => Std::Saved(t, r),
                            // Handled by the dedicated arms above; a stray
                            // extension message is a safe no-op.
                            [<$Name EditorMessage>]::Extension(_)
                            | [<$Name EditorMessage>]::Spreadsheet(_)
                            | [<$Name EditorMessage>]::FieldChanged(..)
                            | [<$Name EditorMessage>]::LoadCatalog => {
                                return ::iced::Task::none();
                            }
                        };
                        let (task, _) = $crate::components::standard::update::handle(
                            std,
                            &mut app.state.editors.$field,
                            &app.state.shared_game_path.clone(),
                            &app.state.lookups,
                            $file,
                            tab_id,
                            [<wrap_ $name _std>],
                        );
                        task
                    }
                }
            }

            $crate::define_standard_view! {
                name: $name,
                field: $field,
                message: [<$Name EditorMessage>],
                title: $crate::standard_title!($name $(, $title)?),
            }
        }
    };
}
