use dispel_core::{Extractor, MagicSpell};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct MagicEditorState {
    pub catalog: Option<Vec<MagicSpell>>,
    pub filtered_spells: Vec<(usize, MagicSpell)>,
    pub selected_idx: Option<usize>,

    pub edit_mana_cost: String,
    pub edit_success_rate: String,
    pub edit_base_damage: String,
    pub edit_range: String,
    pub edit_level_required: String,
    pub edit_effect_value: String,
    pub edit_effect_type: String,
    pub edit_effect_modifier: String,
    pub edit_magic_school: String,
    pub edit_animation_id: String,
    pub edit_visual_id: String,
    pub edit_icon_id: String,
    pub edit_target_type: String,
    pub edit_enabled: String,
    pub edit_flag1: String,
    pub edit_flag2: String,
    pub edit_flag3: String,
    pub edit_constant1: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl MagicEditorState {
    pub fn refresh_spells(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_spells = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_spell(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_spells.get(idx) {
            self.edit_mana_cost = record.mana_cost.to_string();
            self.edit_success_rate = record.success_rate.to_string();
            self.edit_base_damage = record.base_damage.to_string();
            self.edit_range = record.range.to_string();
            self.edit_level_required = record.level_required.to_string();
            self.edit_effect_value = record.effect_value.to_string();
            self.edit_effect_type = record.effect_type.to_string();
            self.edit_effect_modifier = record.effect_modifier.to_string();
            self.edit_magic_school = format!("{:?}", record.magic_school);
            self.edit_animation_id = record.animation_id.to_string();
            self.edit_visual_id = record.visual_id.to_string();
            self.edit_icon_id = record.icon_id.to_string();
            self.edit_target_type = format!("{:?}", record.target_type);
            self.edit_enabled = format!("{:?}", record.enabled);
            self.edit_flag1 = format!("{:?}", record.flag1);
            self.edit_flag2 = format!("{:?}", record.flag2);
            self.edit_flag3 = format!("{:?}", record.flag3);
            self.edit_constant1 = format!("{:?}", record.constant1);
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_spells.get_mut(idx).map(|(_, r)| r) {
            match field {
                "mana_cost" => {
                    self.edit_mana_cost = value.clone();
                    if let Ok(v) = value.parse() {
                        record.mana_cost = v
                    }
                }
                "success_rate" => {
                    self.edit_success_rate = value.clone();
                    if let Ok(v) = value.parse() {
                        record.success_rate = v
                    }
                }
                "base_damage" => {
                    self.edit_base_damage = value.clone();
                    if let Ok(v) = value.parse() {
                        record.base_damage = v
                    }
                }
                "range" => {
                    self.edit_range = value.clone();
                    if let Ok(v) = value.parse() {
                        record.range = v
                    }
                }
                "level_required" => {
                    self.edit_level_required = value.clone();
                    if let Ok(v) = value.parse() {
                        record.level_required = v
                    }
                }
                "effect_value" => {
                    self.edit_effect_value = value.clone();
                    if let Ok(v) = value.parse() {
                        record.effect_value = v
                    }
                }
                "effect_type" => {
                    self.edit_effect_type = value.clone();
                    if let Ok(v) = value.parse() {
                        record.effect_type = v
                    }
                }
                "effect_modifier" => {
                    self.edit_effect_modifier = value.clone();
                    if let Ok(v) = value.parse() {
                        record.effect_modifier = v
                    }
                }
                "animation_id" => {
                    self.edit_animation_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.animation_id = v
                    }
                }
                "visual_id" => {
                    self.edit_visual_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.visual_id = v
                    }
                }
                "icon_id" => {
                    self.edit_icon_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.icon_id = v
                    }
                }
                "enabled" => {
                    self.edit_enabled = value.clone();
                    record.enabled = if value.contains("Enabled") {
                        dispel_core::MagicSpellFlag::Enabled
                    } else {
                        dispel_core::MagicSpellFlag::Disabled
                    };
                }
                "flag1" => {
                    self.edit_flag1 = value.clone();
                    record.flag1 = if value.contains("Enabled") {
                        dispel_core::MagicSpellFlag::Enabled
                    } else {
                        dispel_core::MagicSpellFlag::Disabled
                    };
                }
                "flag2" => {
                    self.edit_flag2 = value.clone();
                    record.flag2 = if value.contains("Enabled") {
                        dispel_core::MagicSpellFlag::Enabled
                    } else {
                        dispel_core::MagicSpellFlag::Disabled
                    };
                }
                "flag3" => {
                    self.edit_flag3 = value.clone();
                    record.flag3 = if value.contains("Enabled") {
                        dispel_core::MagicSpellFlag::Enabled
                    } else {
                        dispel_core::MagicSpellFlag::Disabled
                    };
                }
                "constant1" => {
                    self.edit_constant1 = value.clone();
                    record.constant1 = if value.contains("Standard") {
                        dispel_core::MagicSpellConstant::Standard
                    } else {
                        dispel_core::MagicSpellConstant::Invalid
                    };
                }
                "magic_school" => {
                    self.edit_magic_school = value.clone();
                    if value.contains("School1") {
                        record.magic_school = dispel_core::MagicSchool::School1;
                    } else if value.contains("School2") {
                        record.magic_school = dispel_core::MagicSchool::School2;
                    } else if value.contains("School3") {
                        record.magic_school = dispel_core::MagicSchool::School3;
                    } else if value.contains("School4") {
                        record.magic_school = dispel_core::MagicSchool::School4;
                    } else if value.contains("School5") {
                        record.magic_school = dispel_core::MagicSchool::School5;
                    } else if value.contains("School6") {
                        record.magic_school = dispel_core::MagicSchool::School6;
                    } else {
                        record.magic_school = dispel_core::MagicSchool::Unknown;
                    }
                }
                "target_type" => {
                    self.edit_target_type = value.clone();
                    if value.contains("Single") {
                        record.target_type = dispel_core::SpellTargetType::Single;
                    } else if value.contains("Self") {
                        record.target_type = dispel_core::SpellTargetType::SelfTarget;
                    } else if value.contains("Area") {
                        record.target_type = dispel_core::SpellTargetType::AreaOfEffect;
                    } else if value.contains("Multi") {
                        record.target_type = dispel_core::SpellTargetType::MultiTarget;
                    }
                }
                _ => {}
            }
            self.refresh_spells();
        }
    }

    pub fn save_spells(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("MagicInGame")
            .join("Magic.db");
        if let Some(catalog) = &self.catalog {
            MagicSpell::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save magic spells: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
