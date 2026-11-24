mod component;

crate::define_tab_editor! {
    name: dialogue_script,
    name_pascal: DialogueScript,
    record: dispel_core::DialogueScript,
    field: dialogue_script_editor,
    loaded_prefix: "DialogueScript catalog loaded: ",
    error_prefix: "Error loading dialogue script catalog: ",
    save_success_msg: "DialogueScripts saved successfully.",
    save_error_msg: "Error saving dialogue scripts",
}

mod view;
pub use view::view;
