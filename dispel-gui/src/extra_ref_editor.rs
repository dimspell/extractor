use dispel_core::ExtraRef;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ExtraRefEditorState {
    pub catalog: Option<Vec<ExtraRef>>,
    pub filtered_items: Vec<(usize, ExtraRef)>,
    pub selected_idx: Option<usize>,
    pub current_map_file: String,
    pub map_files: Vec<PathBuf>,

    pub edit_id: String,
    pub edit_name: String,
    pub edit_ext_id: String,
    pub edit_object_type: String,
    pub edit_x_pos: String,
    pub edit_y_pos: String,
    pub edit_rotation: String,
    pub edit_closed: String,
    pub edit_required_item_id: String,
    pub edit_required_item_type_id: String,
    pub edit_required_item_id2: String,
    pub edit_required_item_type_id2: String,
    pub edit_gold_amount: String,
    pub edit_item_id: String,
    pub edit_item_type_id: String,
    pub edit_item_count: String,
    pub edit_event_id: String,
    pub edit_message_id: String,
    pub edit_visibility: String,
    pub edit_interactive_element_type: String,
    pub edit_is_quest_element: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl ExtraRefEditorState {
    pub fn refresh_items(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_items = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_item(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_items.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_name = record.name.clone();
            self.edit_ext_id = record.ext_id.to_string();
            self.edit_object_type = format!("{:?}", record.object_type);
            self.edit_x_pos = record.x_pos.to_string();
            self.edit_y_pos = record.y_pos.to_string();
            self.edit_rotation = record.rotation.to_string();
            self.edit_closed = record.closed.to_string();
            self.edit_required_item_id = record.required_item_id.to_string();
            self.edit_required_item_type_id = format!("{:?}", record.required_item_type_id);
            self.edit_required_item_id2 = record.required_item_id2.to_string();
            self.edit_required_item_type_id2 = format!("{:?}", record.required_item_type_id2);
            self.edit_gold_amount = record.gold_amount.to_string();
            self.edit_item_id = record.item_id.to_string();
            self.edit_item_type_id = format!("{:?}", record.item_type_id);
            self.edit_item_count = record.item_count.to_string();
            self.edit_event_id = record.event_id.to_string();
            self.edit_message_id = record.message_id.to_string();
            self.edit_visibility = format!("{:?}", record.visibility);
            self.edit_interactive_element_type = record.interactive_element_type.to_string();
            self.edit_is_quest_element = record.is_quest_element.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_items.get_mut(idx).map(|(_, r)| r) {
            match field {
                "name" => {
                    self.edit_name = value.clone();
                    record.name = value;
                }
                "ext_id" => {
                    self.edit_ext_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.ext_id = v
                    }
                }
                "object_type" => {
                    self.edit_object_type = value.clone();
                    record.object_type = match value.as_str() {
                        "Chest" => dispel_core::ExtraObjectType::Chest,
                        "Door" => dispel_core::ExtraObjectType::Door,
                        "Sign" => dispel_core::ExtraObjectType::Sign,
                        "Altar" => dispel_core::ExtraObjectType::Altar,
                        "Interactive" => dispel_core::ExtraObjectType::Interactive,
                        "Magic" => dispel_core::ExtraObjectType::Magic,
                        _ => dispel_core::ExtraObjectType::Unknown,
                    };
                }
                "x_pos" => {
                    self.edit_x_pos = value.clone();
                    if let Ok(v) = value.parse() {
                        record.x_pos = v
                    }
                }
                "y_pos" => {
                    self.edit_y_pos = value.clone();
                    if let Ok(v) = value.parse() {
                        record.y_pos = v
                    }
                }
                "rotation" => {
                    self.edit_rotation = value.clone();
                    if let Ok(v) = value.parse() {
                        record.rotation = v
                    }
                }
                "closed" => {
                    self.edit_closed = value.clone();
                    if let Ok(v) = value.parse() {
                        record.closed = v
                    }
                }
                "required_item_id" => {
                    self.edit_required_item_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.required_item_id = v
                    }
                }
                "required_item_type_id" => {
                    self.edit_required_item_type_id = value.clone();
                    if let Some(v) = dispel_core::ItemTypeId::from_name(&value) {
                        record.required_item_type_id = v;
                    }
                }
                "required_item_id2" => {
                    self.edit_required_item_id2 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.required_item_id2 = v
                    }
                }
                "required_item_type_id2" => {
                    self.edit_required_item_type_id2 = value.clone();
                    if let Some(v) = dispel_core::ItemTypeId::from_name(&value) {
                        record.required_item_type_id2 = v;
                    }
                }
                "gold_amount" => {
                    self.edit_gold_amount = value.clone();
                    if let Ok(v) = value.parse() {
                        record.gold_amount = v
                    }
                }
                "item_id" => {
                    self.edit_item_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.item_id = v
                    }
                }
                "item_type_id" => {
                    self.edit_item_type_id = value.clone();
                    if let Some(v) = dispel_core::ItemTypeId::from_name(&value) {
                        record.item_type_id = v;
                    }
                }
                "item_count" => {
                    self.edit_item_count = value.clone();
                    if let Ok(v) = value.parse() {
                        record.item_count = v
                    }
                }
                "event_id" => {
                    self.edit_event_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.event_id = v
                    }
                }
                "message_id" => {
                    self.edit_message_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.message_id = v
                    }
                }
                "visibility" => {
                    self.edit_visibility = value.clone();
                    record.visibility = if value.contains("Visible10") {
                        dispel_core::VisibilityType::Visible10
                    } else if value.contains("Visible0") {
                        dispel_core::VisibilityType::Visible0
                    } else {
                        dispel_core::VisibilityType::Unknown
                    };
                }
                "interactive_element_type" => {
                    self.edit_interactive_element_type = value.clone();
                    if let Ok(v) = value.parse() {
                        record.interactive_element_type = v
                    }
                }
                "is_quest_element" => {
                    self.edit_is_quest_element = value.clone();
                    if let Ok(v) = value.parse() {
                        record.is_quest_element = v
                    }
                }
                _ => {}
            }
            self.refresh_items();
        }
    }

    pub fn save_items(&self) -> Result<(), String> {
        if self.current_map_file.is_empty() {
            return Err("No map file selected".to_string());
        }
        let path = PathBuf::from(&self.current_map_file);
        if let Some(catalog) = &self.catalog {
            ExtraRef::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save extra refs: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
