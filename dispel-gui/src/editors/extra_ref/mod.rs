mod component;

crate::define_tab_editor! {
    name: extra_ref,
    name_pascal: ExtraRef,
    record: dispel_core::ExtraRef,
    field: extra_ref_editor,
    loaded_prefix: "Extra ref catalog loaded: ",
    error_prefix: "Error loading extra ref catalog: ",
    save_success_msg: "Extra refs saved successfully.",
    save_error_msg: "Error saving extra refs",
}

mod view;
pub use view::view;
