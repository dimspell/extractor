use crate::message::{workspace::WorkspaceMessage, Message};
use crate::search_index::SearchIndex;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font, Length};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub catalog_type: String,
    pub record_idx: usize,
    pub display_text: String,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalSearch {
    pub query: String,
    pub pending_query: String,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub is_visible: bool,
}

impl GlobalSearch {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            pending_query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            is_visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
        if !self.is_visible {
            self.query.clear();
            self.pending_query.clear();
            self.results.clear();
        }
    }

    pub fn search(&mut self, index: &SearchIndex) {
        self.results.clear();
        self.selected_index = 0;

        if self.query.is_empty() {
            return;
        }

        let indexed_results = index.search(&self.query);
        for entry in indexed_results {
            self.results.push(SearchResult {
                catalog_type: entry.editor_type.clone(),
                record_idx: entry.record_idx,
                display_text: format!("[{}] {}", entry.editor_type, entry.label),
                source_file: entry.source_file.clone(),
            });
        }
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
            }
        }
    }

    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_index)
    }

    pub fn input_id() -> iced::widget::Id {
        iced::widget::Id::new("global_search_input")
    }

    pub fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search all records...", &self.query)
            .id(Self::input_id())
            .on_input(|s| Message::Workspace(WorkspaceMessage::GlobalSearchInput(s)))
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
                    .on_press(Message::Workspace(WorkspaceMessage::GlobalSearchSelect(
                        idx,
                    )))
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
