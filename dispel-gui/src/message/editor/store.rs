use dispel_core::{EditItem, HealItem, MiscItem, Store, WeaponItem};
use iced::widget::{pane_grid, text_editor};

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub enum StoreEditorMessage {
    LoadCatalog,
    Scanned(
        Result<
            (
                Option<Vec<WeaponItem>>,
                Option<Vec<HealItem>>,
                Option<Vec<MiscItem>>,
                Option<Vec<EditItem>>,
                Vec<Store>,
            ),
            String,
        >,
    ),
    SelectStore(usize),
    FieldChanged(usize, String, String),
    SelectProduct(usize),
    AddProduct,
    RemoveProduct(usize),
    ProductFieldChanged(usize, String, String),
    Save,
    Saved(Result<(), String>),
    InvitationChanged(text_editor::Action),
    HaggleSuccessChanged(text_editor::Action),
    HaggleFailChanged(text_editor::Action),
    PaneResized(pane_grid::ResizeEvent),
    OpenProductModal(Option<usize>),
    CloseProductModal,
    ModalTypeChanged(i16),
    ModalItemIdChanged(String),
    SaveModalProduct,
}
