//! Compile-time / API-surface test for the crate-root re-exports.
//!
//! The point of this module is the *compilation*, not the runtime assertion:
//! if any intended public item stops resolving from `gui_widgets::`, this file
//! fails to build. It also documents the supported surface in one place.

use super::*;

/// All re-exported widget types must be nameable from the crate root.
#[allow(dead_code)]
fn assert_types_resolve_from_root() {
    let _: Option<TreeNode<u8>> = None;
    let _: Option<CollapsibleTree<'static, u8, ()>> = None;
    let _: Option<RenderContext<'static, u8>> = None;
    let _: Option<ContextMenu<'static, ()>> = None;
    let _: Option<Entry<()>> = None;
    let _: Option<Modal<'static, (), iced::Theme, iced::Renderer>> = None;
    let _: Option<Popover<'static, (), iced::Theme, iced::Renderer>> = None;
    let _: Option<ParagraphCache> = None;
    let _: Option<Tab> = None;
    let _: Option<TabBar<'static, ()>> = None;
    let _: Option<TabBarEvent> = None;
    let _: Option<TableWidget<'static, ()>> = None;
    let _: Option<TableColumn> = None;
    let _: Option<TableState> = None;
    let _: Option<RowFlags> = None;
    let _: Option<TextAreaContent> = None;
    let _: Option<Toast> = None;
    let _: Option<ToastStatus> = None;
    let _: Option<ToastManager<'static, ()>> = None;
    let _: Option<u64> = Some(DEFAULT_TIMEOUT);
}

/// All re-exported functions must resolve from the crate root too. Concrete
/// `fn` pointer types pin down the signatures and the default theme/renderer.
#[allow(dead_code, clippy::type_complexity)]
fn assert_functions_resolve_from_root() {
    let _: fn(
        iced::Element<'static, ()>,
        iced::Element<'static, ()>,
        fn() -> (),
        f32,
    ) -> iced::Element<'static, ()> = modal;
    let _: fn(
        iced::Element<'static, ()>,
        iced::Element<'static, ()>,
        bool,
        fn() -> (),
        fn() -> (),
    ) -> iced::Element<'static, ()> = popover;
    let _: fn(
        &'static iced::widget::text_editor::Content,
        fn(iced::widget::text_editor::Action) -> (),
    ) -> iced::Element<'static, ()> = textarea;
    let _: fn(
        &'static iced::widget::text_editor::Content,
        fn(iced::widget::text_editor::Action) -> (),
    ) -> iced::Element<'static, ()> = textarea_fill;
    let _: fn(&iced::Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style =
        textarea_style;
}

#[test]
fn test_public_api_resolves_from_crate_root() {
    assert_types_resolve_from_root();
    assert_functions_resolve_from_root();

    // The root re-export and the fully-qualified module path name the same item.
    let root_toast = Toast::success("t", "b");
    let module_toast = crate::components::toast::Toast::success("t", "b");
    assert_eq!(root_toast.title, module_toast.title);
    assert_eq!(root_toast.status, module_toast.status);
}
