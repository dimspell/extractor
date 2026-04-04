use crate::message::Message;
use crate::state::AppState;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use dispel_core::references::editable::EditableRecord;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font, Length};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub catalog_type: String,
    pub record_idx: usize,
    pub display_text: String,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalSearch {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub is_visible: bool,
}

impl GlobalSearch {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            is_visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
        if !self.is_visible {
            self.query.clear();
            self.results.clear();
        }
    }

    pub fn search(&mut self, state: &AppState) {
        self.results.clear();
        self.selected_index = 0;

        if self.query.is_empty() {
            return;
        }

        let query_lower = self.query.to_lowercase();

        if let Some(cat) = &state.chest_editor.catalog {
            search_catalog(&mut self.results, &cat.weapons, "Weapon", &query_lower);
            search_catalog(&mut self.results, &cat.healing, "HealItem", &query_lower);
            search_catalog(&mut self.results, &cat.misc, "MiscItem", &query_lower);
            search_catalog(&mut self.results, &cat.edit, "EditItem", &query_lower);
            search_catalog(&mut self.results, &cat.event, "EventItem", &query_lower);
        }

        search_editor_catalog(
            &mut self.results,
            &state.weapon_editor.catalog,
            "Weapon",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.heal_item_editor.catalog,
            "HealItem",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.misc_item_editor.catalog,
            "MiscItem",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.edit_item_editor.catalog,
            "EditItem",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.event_item_editor.catalog,
            "EventItem",
            &query_lower,
        );
        search_monster_catalog(
            &mut self.results,
            &state.monster_editor.catalog,
            &query_lower,
        );
        search_npc_ini_catalog(
            &mut self.results,
            &state.npc_ini_editor.catalog,
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.magic_editor.catalog,
            "MagicSpell",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.dialog_editor.catalog,
            "Dialog",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.dialogue_text_editor.catalog,
            "DialogueText",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.store_editor.catalog,
            "Store",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.party_ref_editor.catalog,
            "PartyRef",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.party_ini_editor.catalog,
            "PartyIni",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.draw_item_editor.catalog,
            "DrawItem",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.event_ini_editor.catalog,
            "Event",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.event_npc_ref_editor.catalog,
            "EventNpcRef",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.extra_ini_editor.catalog,
            "Extra",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.extra_ref_editor.catalog,
            "ExtraRef",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.map_ini_editor.catalog,
            "MapIni",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.message_scr_editor.catalog,
            "Message",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.npc_ref_editor.editor.catalog,
            "NpcRef",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.party_level_db_editor.catalog,
            "PartyLevel",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.quest_scr_editor.catalog,
            "Quest",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.wave_ini_editor.catalog,
            "WaveIni",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.chdata_editor.catalog,
            "ChData",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.all_map_ini_editor.catalog,
            "Map",
            &query_lower,
        );
        search_editor_catalog(
            &mut self.results,
            &state.monster_ref_editor.editor.catalog,
            "MonsterRef",
            &query_lower,
        );
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.results.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.results.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_index)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search all records...", &self.query)
            .on_input(Message::GlobalSearchInput)
            .padding(12);

        let results_list: Vec<Element<_>> = self
            .results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let is_selected = idx == self.selected_index;
                let label = text(&result.display_text).size(12).font(Font::MONOSPACE);

                button(label)
                    .width(Length::Fill)
                    .padding([8, 12])
                    .on_press(Message::GlobalSearchSelect(idx))
                    .style(if is_selected {
                        style::selected_button
                    } else {
                        style::chip
                    })
                    .into()
            })
            .collect();

        let list = column(results_list).spacing(2);
        let scroll = scrollable(list).height(Length::Fill);

        let count = if self.results.is_empty() && !self.query.is_empty() {
            text("No results").size(12).style(style::subtle_text)
        } else {
            text(format!("{} results", self.results.len()))
                .size(11)
                .style(style::subtle_text)
        };

        let content = column![
            input,
            horizontal_rule(1),
            row![count, horizontal_space()]
                .padding([8, 12])
                .align_y(iced::Alignment::Center),
            scroll
        ]
        .spacing(0)
        .height(Fill);

        container(content)
            .max_width(400)
            .max_height(500)
            .style(style::modal_container)
            .into()
    }
}

fn matches_search(label: &str, query: &str) -> bool {
    label.to_lowercase().contains(query)
}

fn search_catalog<R: EditableRecord>(
    results: &mut Vec<SearchResult>,
    catalog: &[R],
    type_name: &str,
    query: &str,
) {
    for (idx, record) in catalog.iter().enumerate() {
        let label = record.list_label();
        if matches_search(&label, query) {
            results.push(SearchResult {
                catalog_type: type_name.to_string(),
                record_idx: idx,
                display_text: format!("[{}] {}", type_name, label),
            });
        }
    }
}

fn search_editor_catalog<R: EditableRecord>(
    results: &mut Vec<SearchResult>,
    catalog: &Option<Vec<R>>,
    type_name: &str,
    query: &str,
) {
    if let Some(cat) = catalog {
        search_catalog(results, cat, type_name, query);
    }
}

fn search_npc_ini_catalog(
    results: &mut Vec<SearchResult>,
    catalog: &Option<Vec<dispel_core::NpcIni>>,
    query: &str,
) {
    if let Some(cat) = catalog {
        for (idx, npc) in cat.iter().enumerate() {
            let label = format!("#{} {}", npc.id, npc.description);
            if matches_search(&label, query) {
                results.push(SearchResult {
                    catalog_type: "NpcIni".to_string(),
                    record_idx: idx,
                    display_text: format!("[NpcIni] {}", label),
                });
            }
        }
    }
}

fn search_monster_catalog(
    results: &mut Vec<SearchResult>,
    catalog: &Option<Vec<dispel_core::Monster>>,
    query: &str,
) {
    if let Some(cat) = catalog {
        for (idx, monster) in cat.iter().enumerate() {
            let label = format!("#{} {}", monster.id, monster.name);
            if matches_search(&label, query) {
                results.push(SearchResult {
                    catalog_type: "Monster".to_string(),
                    record_idx: idx,
                    display_text: format!("[Monster] {}", label),
                });
            }
        }
    }
}
