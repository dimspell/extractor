use crate::message::{workspace::WorkspaceMessage, Message, MessageExt};
use crate::style;
use crate::workspace::EditorType;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub struct Command {
    pub id: &'static str,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: fn() -> Message,
}

impl Command {
    pub fn all() -> Vec<Self> {
        vec![
            Command {
                id: "undo",
                label: "Undo",
                shortcut: Some("Ctrl+Z"),
                action: || Message::System(crate::message::system::SystemMessage::Undo),
            },
            Command {
                id: "redo",
                label: "Redo",
                shortcut: Some("Ctrl+Y"),
                action: || Message::System(crate::message::system::SystemMessage::Redo),
            },
            Command {
                id: "toggle-history",
                label: "Toggle Edit History",
                shortcut: Some("Ctrl+H"),
                action: || {
                    Message::Workspace(
                        crate::message::workspace::WorkspaceMessage::ToggleHistoryPanel,
                    )
                },
            },
            Command {
                id: "toggle-sidebar",
                label: "Toggle Sidebar",
                shortcut: None,
                action: || {
                    Message::Workspace(crate::message::workspace::WorkspaceMessage::ToggleSidebar)
                },
            },
            Command {
                id: "toggle-command-palette",
                label: "Toggle Command Palette",
                shortcut: Some("Ctrl+P"),
                action: || {
                    Message::Workspace(
                        crate::message::workspace::WorkspaceMessage::ToggleCommandPalette,
                    )
                },
            },
            Command {
                id: "toggle-global-search",
                label: "Toggle Global Search",
                shortcut: Some("Ctrl+F"),
                action: || {
                    Message::Workspace(
                        crate::message::workspace::WorkspaceMessage::ToggleGlobalSearch,
                    )
                },
            },
            Command {
                id: "rebuild-index",
                label: "Rebuild Search Index",
                shortcut: None,
                action: || Message::System(crate::message::system::SystemMessage::RebuildIndex),
            },
            // ── Workspace Management ─────────────────────────────────────
            Command {
                id: "clear-workspace",
                label: "Clear: Workspace Tabs & Editors",
                shortcut: None,
                action: || Message::System(crate::message::system::SystemMessage::ClearWorkspace),
            },
            // ── Tool views ──────────────────────────────────────────────────
            Command {
                id: "open-db-viewer",
                label: "Open: DB Viewer",
                shortcut: None,
                action: || Message::Workspace(WorkspaceMessage::OpenToolTab(EditorType::DbViewer)),
            },
            Command {
                id: "open-chest-editor",
                label: "Open: Chest Editor",
                shortcut: None,
                action: || {
                    Message::Workspace(WorkspaceMessage::OpenToolTab(EditorType::ChestEditor))
                },
            },
            Command {
                id: "open-store-editor",
                label: "Open: Store Editor",
                shortcut: None,
                action: || {
                    Message::Workspace(WorkspaceMessage::OpenToolTab(EditorType::StoreEditor))
                },
            },
            // ── File operations ──────────────────────────────────────────────
            Command {
                id: "browse-game-path",
                label: "Set Game Path…",
                shortcut: None,
                action: || {
                    Message::System(crate::message::system::SystemMessage::BrowseSharedGamePath)
                },
            },
            // ── Weapon Editor ────────────────────────────────────────────────
            Command {
                id: "scan-weapons",
                label: "Scan: Load Weapon catalog",
                shortcut: None,
                action: || {
                    Message::weapon(
                        crate::message::editor::weapon::WeaponEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-weapons",
                label: "Save: Weapon Editor",
                shortcut: None,
                action: || {
                    Message::weapon(crate::message::editor::weapon::WeaponEditorMessage::Save)
                },
            },
            // ── Heal Item Editor ─────────────────────────────────────────────
            Command {
                id: "scan-heal-items",
                label: "Scan: Load Heal Item catalog",
                shortcut: None,
                action: || {
                    Message::heal_item(
                        crate::message::editor::healitem::HealItemEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-heal-items",
                label: "Save: Heal Item Editor",
                shortcut: None,
                action: || {
                    Message::heal_item(
                        crate::message::editor::healitem::HealItemEditorMessage::Save,
                    )
                },
            },
            // ── Misc Item Editor ─────────────────────────────────────────────
            Command {
                id: "scan-misc-items",
                label: "Scan: Load Misc Item catalog",
                shortcut: None,
                action: || {
                    Message::misc_item(
                        crate::message::editor::miscitem::MiscItemEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-misc-items",
                label: "Save: Misc Item Editor",
                shortcut: None,
                action: || {
                    Message::misc_item(
                        crate::message::editor::miscitem::MiscItemEditorMessage::Save,
                    )
                },
            },
            // ── Magic Editor ─────────────────────────────────────────────────
            Command {
                id: "scan-magic",
                label: "Scan: Load Magic catalog",
                shortcut: None,
                action: || {
                    Message::magic(crate::message::editor::magic::MagicEditorMessage::LoadCatalog)
                },
            },
            Command {
                id: "save-magic",
                label: "Save: Magic Editor",
                shortcut: None,
                action: || Message::magic(crate::message::editor::magic::MagicEditorMessage::Save),
            },
            // ── Monster Editor ───────────────────────────────────────────────
            Command {
                id: "scan-monsters",
                label: "Scan: Load Monster catalog",
                shortcut: None,
                action: || {
                    Message::monster(
                        crate::message::editor::monster::MonsterEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-monsters",
                label: "Save: Monster Editor",
                shortcut: None,
                action: || {
                    Message::monster(crate::message::editor::monster::MonsterEditorMessage::Save)
                },
            },
            // ── Party Ref Editor ─────────────────────────────────────────────
            Command {
                id: "scan-party-ref",
                label: "Scan: Load Party Ref catalog",
                shortcut: None,
                action: || {
                    Message::party_ref(
                        crate::message::editor::partyref::PartyRefEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-party-ref",
                label: "Save: Party Ref Editor",
                shortcut: None,
                action: || {
                    Message::party_ref(
                        crate::message::editor::partyref::PartyRefEditorMessage::Save,
                    )
                },
            },
            // ── Party Ini Editor ─────────────────────────────────────────────
            Command {
                id: "scan-party-ini",
                label: "Scan: Load Party Ini catalog",
                shortcut: None,
                action: || {
                    Message::party_ini(
                        crate::message::editor::partyini::PartyIniEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-party-ini",
                label: "Save: Party Ini Editor",
                shortcut: None,
                action: || {
                    Message::party_ini(
                        crate::message::editor::partyini::PartyIniEditorMessage::Save,
                    )
                },
            },
            // ── ChData Editor ────────────────────────────────────────────────
            Command {
                id: "load-chdata",
                label: "Scan: Load ChData catalog",
                shortcut: None,
                action: || {
                    Message::ch_data(
                        crate::message::editor::chdata::ChDataEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-chdata",
                label: "Save: ChData Editor",
                shortcut: None,
                action: || {
                    Message::ch_data(crate::message::editor::chdata::ChDataEditorMessage::Save)
                },
            },
            // ── Map Ini Editor ───────────────────────────────────────────────
            Command {
                id: "load-map-ini",
                label: "Scan: Load Map Ini catalog",
                shortcut: None,
                action: || {
                    Message::map_ini(
                        crate::message::editor::mapini::MapIniEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-map-ini",
                label: "Save: Map Ini Editor",
                shortcut: None,
                action: || {
                    Message::map_ini(crate::message::editor::mapini::MapIniEditorMessage::Save)
                },
            },
            // ── Wave Ini Editor ──────────────────────────────────────────────
            Command {
                id: "load-wave-ini",
                label: "Scan: Load Wave Ini catalog",
                shortcut: None,
                action: || {
                    Message::wave_ini(
                        crate::message::editor::waveini::WaveIniEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-wave-ini",
                label: "Save: Wave Ini Editor",
                shortcut: None,
                action: || {
                    Message::wave_ini(crate::message::editor::waveini::WaveIniEditorMessage::Save)
                },
            },
            // ── Event Ini Editor ─────────────────────────────────────────────
            Command {
                id: "load-event-ini",
                label: "Scan: Load Event Ini catalog",
                shortcut: None,
                action: || {
                    Message::event_ini(
                        crate::message::editor::eventini::EventIniEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-event-ini",
                label: "Save: Event Ini Editor",
                shortcut: None,
                action: || {
                    Message::event_ini(
                        crate::message::editor::eventini::EventIniEditorMessage::Save,
                    )
                },
            },
            // ── NPC Ini Editor ───────────────────────────────────────────────
            Command {
                id: "scan-npc-ini",
                label: "Scan: Load NPC Ini catalog",
                shortcut: None,
                action: || {
                    Message::npc_ini(crate::message::editor::npcini::NpcIniEditorMessage::LoadCatalog)
                },
            },
            Command {
                id: "save-npc-ini",
                label: "Save: NPC Ini Editor",
                shortcut: None,
                action: || {
                    Message::npc_ini(crate::message::editor::npcini::NpcIniEditorMessage::Save)
                },
            },
            // ── Quest Scr Editor ─────────────────────────────────────────────
            Command {
                id: "load-quest-scr",
                label: "Scan: Load Quest catalog",
                shortcut: None,
                action: || {
                    Message::quest_scr(
                        crate::message::editor::questscr::QuestScrEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-quest-scr",
                label: "Save: Quest Scr Editor",
                shortcut: None,
                action: || {
                    Message::quest_scr(
                        crate::message::editor::questscr::QuestScrEditorMessage::Save,
                    )
                },
            },
            // ── Message Scr Editor ───────────────────────────────────────────
            Command {
                id: "load-message-scr",
                label: "Scan: Load Message catalog",
                shortcut: None,
                action: || {
                    Message::message_scr(
                        crate::message::editor::messagescr::MessageScrEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-message-scr",
                label: "Save: Message Scr Editor",
                shortcut: None,
                action: || {
                    Message::message_scr(
                        crate::message::editor::messagescr::MessageScrEditorMessage::Save,
                    )
                },
            },
            // ── Extra Ini Editor ─────────────────────────────────────────────
            Command {
                id: "load-extra-ini",
                label: "Scan: Load Extra Ini catalog",
                shortcut: None,
                action: || {
                    Message::extra_ini(
                        crate::message::editor::extraini::ExtraIniEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-extra-ini",
                label: "Save: Extra Ini Editor",
                shortcut: None,
                action: || {
                    Message::extra_ini(
                        crate::message::editor::extraini::ExtraIniEditorMessage::Save,
                    )
                },
            },
            // ── Event NpcRef Editor ──────────────────────────────────────────
            Command {
                id: "load-event-npc-ref",
                label: "Scan: Load Event NPC Ref catalog",
                shortcut: None,
                action: || {
                    Message::event_npc_ref(
                        crate::message::editor::eventnpcref::EventNpcRefEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-event-npc-ref",
                label: "Save: Event NPC Ref Editor",
                shortcut: None,
                action: || {
                    Message::event_npc_ref(
                        crate::message::editor::eventnpcref::EventNpcRefEditorMessage::Save,
                    )
                },
            },
            // ── All Map Ini Editor ───────────────────────────────────────────
            Command {
                id: "load-all-map-ini",
                label: "Scan: Load All Map Ini catalog",
                shortcut: None,
                action: || {
                    Message::all_map_ini(
                        crate::message::editor::allmapini::AllMapIniEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-all-map-ini",
                label: "Save: All Map Ini Editor",
                shortcut: None,
                action: || {
                    Message::all_map_ini(
                        crate::message::editor::allmapini::AllMapIniEditorMessage::Save,
                    )
                },
            },
            // ── Party Level Db Editor ────────────────────────────────────────
            Command {
                id: "load-party-level-db",
                label: "Scan: Load Party Level Db catalog",
                shortcut: None,
                action: || {
                    Message::party_level_db(
                    crate::message::editor::partyleveldb::PartyLevelDbEditorMessage::LoadCatalog
                )
                },
            },
            Command {
                id: "save-party-level-db",
                label: "Save: Party Level Db Editor",
                shortcut: None,
                action: || {
                    Message::party_level_db(
                        crate::message::editor::partyleveldb::PartyLevelDbEditorMessage::Save,
                    )
                },
            },
            // ── Draw Item Editor ─────────────────────────────────────────────
            Command {
                id: "load-draw-item",
                label: "Scan: Load Draw Item catalog",
                shortcut: None,
                action: || {
                    Message::draw_item(
                        crate::message::editor::drawitem::DrawItemEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-draw-item",
                label: "Save: Draw Item Editor",
                shortcut: None,
                action: || {
                    Message::draw_item(
                        crate::message::editor::drawitem::DrawItemEditorMessage::Save,
                    )
                },
            },
            // ── Edit Item Editor ─────────────────────────────────────────────
            Command {
                id: "scan-edit-items",
                label: "Scan: Load Edit Item catalog",
                shortcut: None,
                action: || {
                    Message::edit_item(
                        crate::message::editor::edititem::EditItemEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-edit-items",
                label: "Save: Edit Item Editor",
                shortcut: None,
                action: || {
                    Message::edit_item(
                        crate::message::editor::edititem::EditItemEditorMessage::Save,
                    )
                },
            },
            // ── Event Item Editor ────────────────────────────────────────────
            Command {
                id: "scan-event-items",
                label: "Scan: Load Event Item catalog",
                shortcut: None,
                action: || {
                    Message::event_item(
                        crate::message::editor::eventitem::EventItemEditorMessage::LoadCatalog,
                    )
                },
            },
            Command {
                id: "save-event-items",
                label: "Save: Event Item Editor",
                shortcut: None,
                action: || {
                    Message::event_item(
                        crate::message::editor::eventitem::EventItemEditorMessage::Save,
                    )
                },
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct CommandPalette {
    pub input_value: String,
    pub filtered_commands: Vec<Command>,
    pub selected_index: usize,
    pub all_commands: Vec<Command>,
}

impl CommandPalette {
    pub fn new() -> Self {
        let all_commands = Command::all();
        Self {
            input_value: String::new(),
            filtered_commands: all_commands.clone(),
            selected_index: 0,
            all_commands,
        }
    }

    pub fn update_input(&mut self, input: String) {
        self.input_value = input.clone();
        self.filter_commands(&input);
        self.selected_index = 0;
    }

    fn filter_commands(&mut self, query: &str) {
        if query.is_empty() {
            self.filtered_commands = self.all_commands.clone();
            return;
        }

        let mut scored: Vec<(u32, usize)> = self
            .all_commands
            .iter()
            .enumerate()
            .filter_map(|(idx, cmd)| {
                let score = fuzzy_score(cmd.label, query);
                if score > 0 {
                    Some((score, idx))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        self.filtered_commands = scored
            .into_iter()
            .map(|(_, idx)| self.all_commands[idx].clone())
            .collect();

        if self.selected_index >= self.filtered_commands.len() {
            self.selected_index = self.filtered_commands.len().saturating_sub(1);
        }
    }

    pub fn select_next(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_commands.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_commands.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered_commands.get(self.selected_index)
    }

    pub fn input_id() -> iced::widget::Id {
        iced::widget::Id::new("command_palette_input")
    }

    pub fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search commands...", &self.input_value)
            .id(Self::input_id())
            .on_input(|s| Message::Workspace(WorkspaceMessage::CommandPaletteInput(s)))
            .padding(12);

        let commands: Vec<Element<_>> = self
            .filtered_commands
            .iter()
            .enumerate()
            .map(|(idx, cmd)| {
                let is_selected = idx == self.selected_index;
                let label = if let Some(shortcut) = cmd.shortcut {
                    row![
                        text(cmd.label).width(Length::Fill),
                        text(shortcut)
                            .size(11)
                            .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                    ]
                } else {
                    row![text(cmd.label).width(Length::Fill)]
                };

                button(label)
                    .width(Length::Fill)
                    .padding([8, 12])
                    .on_press(Message::Workspace(WorkspaceMessage::CommandPaletteSelect(
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

        let list = scrollable(column(commands)).spacing(2);

        let content = column![input, list].spacing(8).padding(16);

        container(content)
            .max_width(500)
            .style(style::modal_container)
            .into()
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

/// Fuzzy-match scoring for command palette filtering.
///
/// Returns 0 if the query doesn't match, otherwise returns a score
/// where higher is better. Scoring considers:
/// - Consecutive character matches (bonus)
/// - Matches at word boundaries (bonus)
/// - Matches at the start of the string (bonus)
/// - Overall match density
fn fuzzy_score(text: &str, query: &str) -> u32 {
    if query.is_empty() {
        return 1;
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    let query_chars: Vec<char> = query_lower.chars().collect();
    let text_chars: Vec<char> = text_lower.chars().collect();
    let qlen = query_chars.len();
    let tlen = text_chars.len();

    if qlen > tlen {
        return 0;
    }

    // Greedy match: find query chars in text in order
    let mut matches: Vec<usize> = Vec::with_capacity(qlen);
    let mut qi = 0;
    let mut ti = 0;

    while qi < qlen && ti < tlen {
        if query_chars[qi] == text_chars[ti] {
            matches.push(ti);
            qi += 1;
        }
        ti += 1;
    }

    if matches.len() < qlen {
        return 0;
    }

    // Calculate score
    let mut score: u32 = 0;

    // Base score: match density
    score += (matches.len() as u32) * 10;

    // Consecutive match bonus
    for i in 1..matches.len() {
        if matches[i] == matches[i - 1] + 1 {
            score += 5;
        }
    }

    // Word boundary bonus
    for (i, &pos) in matches.iter().enumerate() {
        if pos == 0 {
            score += 15; // Start of string
        } else if text_chars[pos - 1] == ' ' || text_chars[pos - 1] == '-' {
            score += 10; // After space or hyphen
        } else if (i == 0
            || (pos > 0 && text_chars[pos].is_uppercase() && !text_chars[pos - 1].is_uppercase()))
            && text_chars[pos].is_uppercase()
        {
            score += 8; // CamelCase boundary
        }
    }

    // First character match bonus
    if matches[0] == 0 {
        score += 20;
    }

    // Penalize gaps (prefer tighter matches)
    let total_gap: u32 = if matches.len() > 1 {
        (matches[matches.len() - 1] - matches[0]) as u32
    } else {
        0
    };
    score = score.saturating_sub(total_gap);

    score
}
