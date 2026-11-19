mod component;

crate::define_tab_editor! {
    name: dialogue_paragraph,
    name_pascal: DialogueParagraph,
    record: dispel_core::DialogueParagraph,
    field: dialogue_paragraph_editor,
    loaded_prefix: "Text catalog loaded: ",
    error_prefix: "Error loading text catalog: ",
    save_success_msg: "Texts saved successfully.",
    save_error_msg: "Error saving texts",
}

mod view;
pub use view::view;
