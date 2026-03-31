use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::types::MapOp;
use crate::utils::{labeled_file_row, labeled_input};
use iced::widget::{button, column, row, text, toggler, vertical_space};
use iced::Element;

impl App {
    pub fn view_map_tab(&self) -> Element<'_, Message> {
        let op_buttons: Vec<Element<Message>> = MapOp::ALL
            .iter()
            .map(|op| {
                let is_active = self.map_op == Some(*op);
                let btn = button(text(op.label()).size(12))
                    .padding([6, 12])
                    .on_press(Message::MapOpSelected(*op));
                if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                }
                .into()
            })
            .collect();
        let fields: Element<Message> = match self.map_op {
            Some(MapOp::Tiles) | Some(MapOp::Sprites) => column![
                labeled_file_row(
                    "Input:",
                    &self.map_input,
                    Message::MapInputChanged,
                    Message::BrowseMapInput
                ),
                labeled_input("Output dir:", &self.map_output, Message::MapOutputChanged),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::Atlas) => column![
                labeled_file_row(
                    "Input:",
                    &self.map_input,
                    Message::MapInputChanged,
                    Message::BrowseMapInput
                ),
                labeled_input("Output PNG:", &self.map_output, Message::MapOutputChanged),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::Render) => column![
                labeled_file_row(
                    "MAP file:",
                    &self.map_map_path,
                    Message::MapMapPathChanged,
                    Message::BrowseMapMapPath
                ),
                labeled_file_row(
                    "BTL file:",
                    &self.map_btl_path,
                    Message::MapBtlPathChanged,
                    Message::BrowseMapBtlPath
                ),
                labeled_file_row(
                    "GTL file:",
                    &self.map_gtl_path,
                    Message::MapGtlPathChanged,
                    Message::BrowseMapGtlPath
                ),
                labeled_input("Output PNG:", &self.map_output, Message::MapOutputChanged),
                toggler(self.map_save_sprites)
                    .label("Save sprites")
                    .on_toggle(Message::MapSaveSpritesToggled)
                    .size(18)
                    .spacing(8),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::FromDb) => column![
                labeled_input("Database:", &self.map_database, Message::MapDatabaseChanged),
                labeled_input("Map ID:", &self.map_map_id, Message::MapMapIdChanged),
                labeled_file_row(
                    "GTL Atlas:",
                    &self.map_gtl_atlas,
                    Message::MapGtlAtlasChanged,
                    Message::BrowseMapGtlAtlas
                ),
                labeled_file_row(
                    "BTL Atlas:",
                    &self.map_btl_atlas,
                    Message::MapBtlAtlasChanged,
                    Message::BrowseMapBtlAtlas
                ),
                labeled_input(
                    "Columns:",
                    &self.map_atlas_columns,
                    Message::MapAtlasColumnsChanged
                ),
                labeled_input("Output:", &self.map_output, Message::MapOutputChanged),
                labeled_file_row(
                    "Game Path:",
                    &self.map_game_path,
                    Message::MapGamePathChanged,
                    Message::BrowseMapGamePath
                ),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::ToDb) => column![
                labeled_input("Database:", &self.map_database, Message::MapDatabaseChanged),
                labeled_file_row(
                    "MAP file:",
                    &self.map_map_path,
                    Message::MapMapPathChanged,
                    Message::BrowseMapMapPath
                ),
            ]
            .spacing(10)
            .into(),
            None => text("Select an operation above.").into(),
        };
        column![
            row(op_buttons).spacing(6).wrap(),
            vertical_space().height(12),
            fields
        ]
        .spacing(4)
        .into()
    }
}
