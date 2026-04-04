use crate::chest_editor::ItemCatalog;
use crate::generic_editor::GenericEditorState;
use crate::message::Message;
use crate::state::AppState;
use crate::style;
use crate::types::Tab;
use crate::utils::{horizontal_rule, horizontal_space};
use dispel_core::references::editable::EditableRecord;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font, Length};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub catalog_type: String,
    pub record_idx: usize,
    pub display_text: String,
    pub field_matches: Vec<String>,
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

    pub fn search(&mut self, state: &AppState, catalog: &Option<ItemCatalog>) {
        self.results.clear();
        self.selected_index = 0;

        if self.query.is_empty() {
            return;
        }

        let query_lower = self.query.to_lowercase();

        if let Some(cat) = catalog {
            for (idx, item) in cat.weapons.iter().enumerate() {
                if matches_search(&item.list_label(), &query_lower) {
                    self.results.push(SearchResult {
                        catalog_type: "Weapon".to_string(),
                        record_idx: idx,
                        display_text: format!("[Weapon] {}", item.list_label()),
                        field_matches: vec![],
                    });
                }
            }
            for (idx, item) in cat.healing.iter().enumerate() {
                if matches_search(&item.list_label(), &query_lower) {
                    self.results.push(SearchResult {
                        catalog_type: "HealItem".to_string(),
                        record_idx: idx,
                        display_text: format!("[Heal] {}", item.list_label()),
                        field_matches: vec![],
                    });
                }
            }
            for (idx, item) in cat.misc.iter().enumerate() {
                if matches_search(&item.list_label(), &query_lower) {
                    self.results.push(SearchResult {
                        catalog_type: "MiscItem".to_string(),
                        record_idx: idx,
                        display_text: format!("[Misc] {}", item.list_label()),
                        field_matches: vec![],
                    });
                }
            }
            for (idx, item) in cat.edit.iter().enumerate() {
                if matches_search(&item.list_label(), &query_lower) {
                    self.results.push(SearchResult {
                        catalog_type: "EditItem".to_string(),
                        record_idx: idx,
                        display_text: format!("[Edit] {}", item.list_label()),
                        field_matches: vec![],
                    });
                }
            }
            for (idx, item) in cat.event.iter().enumerate() {
                if matches_search(&item.list_label(), &query_lower) {
                    self.results.push(SearchResult {
                        catalog_type: "EventItem".to_string(),
                        record_idx: idx,
                        display_text: format!("[Event] {}", item.list_label()),
                        field_matches: vec![],
                    });
                }
            }
        }

        search_generic_editor(
            &mut self.results,
            &state.monster_editor,
            "Monster",
            &query_lower,
        );
        search_generic_editor(
            &mut self.results,
            &state.npc_ini_editor,
            "NpcIni",
            &query_lower,
        );
        search_generic_editor(
            &mut self.results,
            &state.magic_editor,
            "MagicSpell",
            &query_lower,
        );
        search_generic_editor(
            &mut self.results,
            &state.dialog_editor,
            "Dialog",
            &query_lower,
        );
        search_generic_editor(
            &mut self.results,
            &state.dialogue_text_editor,
            "DialogueText",
            &query_lower,
        );
        search_generic_editor(
            &mut self.results,
            &state.store_editor,
            "Store",
            &query_lower,
        );
        search_generic_editor(
            &mut self.results,
            &state.party_ref_editor,
            "PartyRef",
            &query_lower,
        );
        search_generic_editor(
            &mut self.results,
            &state.party_ini_editor,
            "PartyIni",
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

impl Default for GlobalSearch {
    fn default() -> Self {
        Self::new()
    }
}

fn matches_search(label: &str, query: &str) -> bool {
    label.to_lowercase().contains(query)
}

fn search_generic_editor<R: EditableRecord>(
    results: &mut Vec<SearchResult>,
    editor: &GenericEditorState<R>,
    type_name: &str,
    query: &str,
) {
    if let Some(catalog) = &editor.catalog {
        for (idx, record) in catalog.iter().enumerate() {
            if matches_search(&record.list_label(), query) {
                results.push(SearchResult {
                    catalog_type: type_name.to_string(),
                    record_idx: idx,
                    display_text: format!("[{}] {}", type_name, record.list_label()),
                    field_matches: vec![],
                });
            }
        }
    }
}
