use crate::app::App;
use crate::message::Message;
use crate::view::editor::view_spreadsheet;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_heal_item_editor_tab(&self) -> Element<'_, Message> {
        if self.state.heal_item_spreadsheet.show_inspector
            || self.state.heal_item_spreadsheet.sort_column.is_some()
            || !self.state.heal_item_spreadsheet.filter_query.is_empty()
        {
            return view_spreadsheet(
                &self.state.heal_item_editor,
                &self.state.heal_item_spreadsheet,
                Message::HealItemOpScanItems,
                Message::HealItemOpSave,
                Message::HealItemOpSelectItem,
                Message::HealItemOpFieldChanged,
                Message::HealItemSpreadsheet,
                &self.state.lookups,
            );
        }
        build_editor_view(
            self,
            &self.state.heal_item_editor,
            Message::HealItemOpScanItems,
            Message::HealItemOpSave,
            Message::HealItemOpSelectItem,
            Message::HealItemOpFieldChanged,
            &self.state.lookups,
        )
    }
}
//         build_editor_view(
//             self,
//             &self.state.heal_item_editor,
//             Message::HealItemOpScanItems,
//             Message::HealItemOpSave,
//             Message::HealItemOpSelectItem,
//             Message::HealItemOpFieldChanged,
//             &self.state.lookups,
//         )
//     }
// }
//         }
//         view_spreadsheet(
//             &self.state.heal_item_editor,
//             &self.state.heal_item_spreadsheet,
//             Message::HealItemOpScanItems,
//             Message::HealItemOpSave,
//             Message::HealItemOpSelectItem,
//             Message::HealItemOpFieldChanged,
//             Message::HealItemSpreadsheet,
//             &self.state.lookups,
//         )
//     }
// }
