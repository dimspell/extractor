use crate::message::{Message, MessageExt, workspace::WorkspaceMessage};
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
    /// Editor types this command applies to. Empty means universal (always shown).
    pub applicable_editors: Vec<EditorType>,
}

impl Command {
    pub fn all() -> Vec<Self> {
        vec![
            Command {
                id: "undo",
                label: "Undo",
                shortcut: Some("Ctrl+Z"),
                action: || Message::System(crate::message::system::SystemMessage::Undo),
                applicable_editors: vec![],
            },
            Command {
                id: "redo",
                label: "Redo",
                shortcut: Some("Ctrl+Y"),
                action: || Message::System(crate::message::system::SystemMessage::Redo),
                applicable_editors: vec![],
            },
            Command {
                id: "toggle-history",
                label: "Toggle Edit History",
                shortcut: Some("Ctrl+H"),
                action: || Message::Workspace(WorkspaceMessage::ToggleHistoryPanel),
                applicable_editors: vec![],
            },
            Command {
                id: "toggle-sidebar",
                label: "Toggle Sidebar",
                shortcut: None,
                action: || Message::Workspace(WorkspaceMessage::ToggleSidebar),
                applicable_editors: vec![],
            },
            Command {
                id: "toggle-command-palette",
                label: "Toggle Command Palette",
                shortcut: Some("Ctrl+Shift+P"),
                action: || Message::Workspace(WorkspaceMessage::ToggleCommandPalette),
                applicable_editors: vec![],
            },
            Command {
                id: "toggle-global-search",
                label: "Toggle Global Search",
                shortcut: Some("Ctrl+P"),
                action: || Message::Workspace(WorkspaceMessage::ToggleGlobalSearch),
                applicable_editors: vec![],
            },
            Command {
                id: "rebuild-index",
                label: "Rebuild Search Index",
                shortcut: None,
                action: || Message::System(crate::message::system::SystemMessage::RebuildIndex),
                applicable_editors: vec![],
            },
            // ── Workspace Management ─────────────────────────────────────
            Command {
                id: "clear-workspace",
                label: "Clear: Workspace Tabs & Editors",
                shortcut: None,
                action: || Message::System(crate::message::system::SystemMessage::ClearWorkspace),
                applicable_editors: vec![],
            },
            // ── Tool views ──────────────────────────────────────────────────
            Command {
                id: "open-db-viewer",
                label: "Open: DB Viewer",
                shortcut: None,
                action: || Message::Workspace(WorkspaceMessage::OpenToolTab(EditorType::DbViewer)),
                applicable_editors: vec![],
            },
            Command {
                id: "open-store-editor",
                label: "Open: Store Editor",
                shortcut: None,
                action: || {
                    Message::Workspace(WorkspaceMessage::OpenToolTab(EditorType::StoreEditor))
                },
                applicable_editors: vec![],
            },
            // ── File operations ──────────────────────────────────────────────
            Command {
                id: "open-as-hex",
                label: "Open current file in the hex editor",
                shortcut: Some("Ctrl+Shift+X"),
                action: || Message::Workspace(WorkspaceMessage::ReopenActiveTabAsHex),
                applicable_editors: vec![],
            },
            Command {
                id: "hex-search",
                label: "Hex editor: Search (Find)",
                shortcut: Some("Ctrl+F"),
                action: || Message::hex_editor(hexedit::HexEditorMessage::OpenSearch),
                applicable_editors: vec![EditorType::HexEditor],
            },
            Command {
                id: "hex-goto",
                label: "Hex editor: Go to address",
                shortcut: Some("Ctrl+G"),
                action: || Message::hex_editor(hexedit::HexEditorMessage::OpenGotoDialog),
                applicable_editors: vec![EditorType::HexEditor],
            },
            Command {
                id: "browse-game-path",
                label: "Set Game Path…",
                shortcut: None,
                action: || {
                    Message::System(crate::message::system::SystemMessage::BrowseSharedGamePath)
                },
                applicable_editors: vec![],
            },
            // ── Weapon Editor ────────────────────────────────────────────────
            Command {
                id: "scan-weapons",
                label: "Scan: Load Weapon catalog",
                shortcut: None,
                action: || {
                    Message::weapon(crate::editors::weapon::WeaponEditorMessage::LoadCatalog)
                },
                applicable_editors: vec![EditorType::WeaponEditor],
            },
            Command {
                id: "save-weapons",
                label: "Save: Weapon Editor",
                shortcut: None,
                action: || Message::weapon(crate::editors::weapon::WeaponEditorMessage::Save),
                applicable_editors: vec![EditorType::WeaponEditor],
            },
            // ── Heal Item Editor ─────────────────────────────────────────────
            Command {
                id: "scan-heal-items",
                label: "Scan: Load Heal Item catalog",
                shortcut: None,
                action: || {
                    Message::heal_item(
                        crate::editors::heal_item::HealItemEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::HealItemEditor],
            },
            Command {
                id: "save-heal-items",
                label: "Save: Heal Item Editor",
                shortcut: None,
                action: || {
                    Message::heal_item(crate::editors::heal_item::HealItemEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::HealItemEditor],
            },
            // ── Misc Item Editor ─────────────────────────────────────────────
            Command {
                id: "scan-misc-items",
                label: "Scan: Load Misc Item catalog",
                shortcut: None,
                action: || {
                    Message::misc_item(
                        crate::editors::misc_item::MiscItemEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::MiscItemEditor],
            },
            Command {
                id: "save-misc-items",
                label: "Save: Misc Item Editor",
                shortcut: None,
                action: || {
                    Message::misc_item(crate::editors::misc_item::MiscItemEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::MiscItemEditor],
            },
            // ── Magic Editor ─────────────────────────────────────────────────
            Command {
                id: "scan-magic",
                label: "Scan: Load Magic catalog",
                shortcut: None,
                action: || Message::magic(crate::editors::magic::MagicEditorMessage::LoadCatalog),
                applicable_editors: vec![EditorType::MagicEditor],
            },
            Command {
                id: "save-magic",
                label: "Save: Magic Editor",
                shortcut: None,
                action: || Message::magic(crate::editors::magic::MagicEditorMessage::Save),
                applicable_editors: vec![EditorType::MagicEditor],
            },
            // ── Monster Editor ───────────────────────────────────────────────
            Command {
                id: "scan-monsters",
                label: "Scan: Load Monster catalog",
                shortcut: None,
                action: || {
                    Message::monster(crate::editors::monster::MonsterEditorMessage::LoadCatalog)
                },
                applicable_editors: vec![EditorType::MonsterEditor],
            },
            Command {
                id: "save-monsters",
                label: "Save: Monster Editor",
                shortcut: None,
                action: || Message::monster(crate::editors::monster::MonsterEditorMessage::Save),
                applicable_editors: vec![EditorType::MonsterEditor],
            },
            // ── Party Ref Editor ─────────────────────────────────────────────
            Command {
                id: "scan-party-ref",
                label: "Scan: Load Party Ref catalog",
                shortcut: None,
                action: || {
                    Message::party_ref(
                        crate::editors::party_ref::PartyRefEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::PartyRefEditor],
            },
            Command {
                id: "save-party-ref",
                label: "Save: Party Ref Editor",
                shortcut: None,
                action: || {
                    Message::party_ref(crate::editors::party_ref::PartyRefEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::PartyRefEditor],
            },
            // ── Party Ini Editor ─────────────────────────────────────────────
            Command {
                id: "scan-party-ini",
                label: "Scan: Load Party Ini catalog",
                shortcut: None,
                action: || {
                    Message::party_ini(
                        crate::editors::party_ini::PartyIniEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::PartyIniEditor],
            },
            Command {
                id: "save-party-ini",
                label: "Save: Party Ini Editor",
                shortcut: None,
                action: || {
                    Message::party_ini(crate::editors::party_ini::PartyIniEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::PartyIniEditor],
            },
            // ── ChData Editor ────────────────────────────────────────────────
            Command {
                id: "load-chdata",
                label: "Scan: Load ChData catalog",
                shortcut: None,
                action: || {
                    Message::chdata(crate::editors::chdata::ChDataEditorMessage::LoadCatalog)
                },
                applicable_editors: vec![EditorType::ChDataEditor],
            },
            Command {
                id: "save-chdata",
                label: "Save: ChData Editor",
                shortcut: None,
                action: || Message::chdata(crate::editors::chdata::ChDataEditorMessage::Save),
                applicable_editors: vec![EditorType::ChDataEditor],
            },
            // ── Map Ini Editor ───────────────────────────────────────────────
            Command {
                id: "load-map-ini",
                label: "Scan: Load Map Ini catalog",
                shortcut: None,
                action: || {
                    Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::LoadCatalog)
                },
                applicable_editors: vec![EditorType::MapIniEditor],
            },
            Command {
                id: "save-map-ini",
                label: "Save: Map Ini Editor",
                shortcut: None,
                action: || Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::Save),
                applicable_editors: vec![EditorType::MapIniEditor],
            },
            // ── Wave Ini Editor ──────────────────────────────────────────────
            Command {
                id: "load-wave-ini",
                label: "Scan: Load Wave Ini catalog",
                shortcut: None,
                action: || {
                    Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::LoadCatalog)
                },
                applicable_editors: vec![EditorType::WaveIniEditor],
            },
            Command {
                id: "save-wave-ini",
                label: "Save: Wave Ini Editor",
                shortcut: None,
                action: || Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::Save),
                applicable_editors: vec![EditorType::WaveIniEditor],
            },
            // ── Event Ini Editor ─────────────────────────────────────────────
            Command {
                id: "load-event-ini",
                label: "Scan: Load Event Ini catalog",
                shortcut: None,
                action: || {
                    Message::event_ini(
                        crate::editors::event_ini::EventIniEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::EventIniEditor],
            },
            Command {
                id: "save-event-ini",
                label: "Save: Event Ini Editor",
                shortcut: None,
                action: || {
                    Message::event_ini(crate::editors::event_ini::EventIniEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::EventIniEditor],
            },
            // ── NPC Ini Editor ───────────────────────────────────────────────
            Command {
                id: "scan-npc-ini",
                label: "Scan: Load NPC Ini catalog",
                shortcut: None,
                action: || {
                    Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::LoadCatalog)
                },
                applicable_editors: vec![EditorType::NpcIniEditor],
            },
            Command {
                id: "save-npc-ini",
                label: "Save: NPC Ini Editor",
                shortcut: None,
                action: || Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::Save),
                applicable_editors: vec![EditorType::NpcIniEditor],
            },
            // ── Quest Scr Editor ─────────────────────────────────────────────
            Command {
                id: "load-quest-scr",
                label: "Scan: Load Quest catalog",
                shortcut: None,
                action: || {
                    Message::quest_scr(
                        crate::editors::quest_scr::QuestScrEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::QuestScrEditor],
            },
            Command {
                id: "save-quest-scr",
                label: "Save: Quest Scr Editor",
                shortcut: None,
                action: || {
                    Message::quest_scr(crate::editors::quest_scr::QuestScrEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::QuestScrEditor],
            },
            // ── Message Scr Editor ───────────────────────────────────────────
            Command {
                id: "load-message-scr",
                label: "Scan: Load Message catalog",
                shortcut: None,
                action: || {
                    Message::message_scr(
                        crate::editors::message_scr::MessageScrEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::QuestScrEditor],
            },
            Command {
                id: "save-message-scr",
                label: "Save: Message Scr Editor",
                shortcut: None,
                action: || {
                    Message::message_scr(crate::editors::message_scr::MessageScrEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::MessageScrEditor],
            },
            // ── Extra Ini Editor ─────────────────────────────────────────────
            Command {
                id: "load-extra-ini",
                label: "Scan: Load Extra Ini catalog",
                shortcut: None,
                action: || {
                    Message::extra_ini(
                        crate::editors::extra_ini::ExtraIniEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::ExtraIniEditor],
            },
            Command {
                id: "save-extra-ini",
                label: "Save: Extra Ini Editor",
                shortcut: None,
                action: || {
                    Message::extra_ini(crate::editors::extra_ini::ExtraIniEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::ExtraIniEditor],
            },
            // ── Event NpcRef Editor ──────────────────────────────────────────
            Command {
                id: "load-event-npc-ref",
                label: "Scan: Load Event NPC Ref catalog",
                shortcut: None,
                action: || {
                    Message::event_npc_ref(
                        crate::editors::event_npc_ref::EventNpcRefEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::EventNpcRefEditor],
            },
            Command {
                id: "save-event-npc-ref",
                label: "Save: Event NPC Ref Editor",
                shortcut: None,
                action: || {
                    Message::event_npc_ref(
                        crate::editors::event_npc_ref::EventNpcRefEditorMessage::Save,
                    )
                },
                applicable_editors: vec![EditorType::EventNpcRefEditor],
            },
            // ── All Map Ini Editor ───────────────────────────────────────────
            Command {
                id: "load-all-map-ini",
                label: "Scan: Load All Map Ini catalog",
                shortcut: None,
                action: || {
                    Message::all_map_ini(
                        crate::editors::all_map_ini::AllMapIniEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::MapIniEditor],
            },
            Command {
                id: "save-all-map-ini",
                label: "Save: All Map Ini Editor",
                shortcut: None,
                action: || {
                    Message::all_map_ini(crate::editors::all_map_ini::AllMapIniEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::MapIniEditor],
            },
            // ── Party Level Db Editor ────────────────────────────────────────
            Command {
                id: "load-party-level-db",
                label: "Scan: Load Party Level Db catalog",
                shortcut: None,
                action: || {
                    Message::party_level_db(
                        crate::editors::party_level_db::PartyLevelDbEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::PartyLevelDbEditor],
            },
            Command {
                id: "save-party-level-db",
                label: "Save: Party Level Db Editor",
                shortcut: None,
                action: || {
                    Message::party_level_db(
                        crate::editors::party_level_db::PartyLevelDbEditorMessage::Save,
                    )
                },
                applicable_editors: vec![EditorType::PartyLevelDbEditor],
            },
            // ── Draw Item Editor ─────────────────────────────────────────────
            Command {
                id: "load-draw-item",
                label: "Scan: Load Draw Item catalog",
                shortcut: None,
                action: || {
                    Message::draw_item(
                        crate::editors::draw_item::DrawItemEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::DrawItemEditor],
            },
            Command {
                id: "save-draw-item",
                label: "Save: Draw Item Editor",
                shortcut: None,
                action: || {
                    Message::draw_item(crate::editors::draw_item::DrawItemEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::DrawItemEditor],
            },
            // ── Edit Item Editor ─────────────────────────────────────────────
            Command {
                id: "scan-edit-items",
                label: "Scan: Load Edit Item catalog",
                shortcut: None,
                action: || {
                    Message::edit_item(
                        crate::editors::edit_item::EditItemEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::EditItemEditor],
            },
            Command {
                id: "save-edit-items",
                label: "Save: Edit Item Editor",
                shortcut: None,
                action: || {
                    Message::edit_item(crate::editors::edit_item::EditItemEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::EditItemEditor],
            },
            // ── Event Item Editor ────────────────────────────────────────────
            Command {
                id: "scan-event-items",
                label: "Scan: Load Event Item catalog",
                shortcut: None,
                action: || {
                    Message::event_item(
                        crate::editors::event_item::EventItemEditorMessage::LoadCatalog,
                    )
                },
                applicable_editors: vec![EditorType::EventItemEditor],
            },
            Command {
                id: "save-event-items",
                label: "Save: Event Item Editor",
                shortcut: None,
                action: || {
                    Message::event_item(crate::editors::event_item::EventItemEditorMessage::Save)
                },
                applicable_editors: vec![EditorType::EventItemEditor],
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
    pub active_editor_type: Option<EditorType>,
}

impl CommandPalette {
    pub fn new() -> Self {
        let all_commands = Command::all();
        Self {
            input_value: String::new(),
            filtered_commands: all_commands.clone(),
            selected_index: 0,
            all_commands,
            active_editor_type: None,
        }
    }

    pub fn set_active_editor_type(&mut self, editor_type: Option<EditorType>) {
        self.active_editor_type = editor_type;
        self.update_input(self.input_value.clone());
    }

    pub fn update_input(&mut self, input: String) {
        self.input_value = input.clone();
        self.filter_commands(&input);
        self.selected_index = 0;
    }

    fn filter_commands(&mut self, query: &str) {
        self.filtered_commands = self
            .all_commands
            .iter()
            .filter(|cmd| {
                // If query is empty, show all commands that match the editor scope
                if query.is_empty() {
                    match &self.active_editor_type {
                        Some(editor_type) => {
                            cmd.applicable_editors.is_empty()
                                || cmd.applicable_editors.contains(editor_type)
                        }
                        None => true,
                    }
                } else {
                    // Both query and editor scope must match
                    let query_matches = fuzzy_score(cmd.label, query) > 0;
                    let editor_matches = match &self.active_editor_type {
                        Some(editor_type) => {
                            cmd.applicable_editors.is_empty()
                                || cmd.applicable_editors.contains(editor_type)
                        }
                        None => true,
                    };
                    query_matches && editor_matches
                }
            })
            .cloned()
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

    pub fn scroll_id() -> iced::widget::Id {
        iced::widget::Id::new("command_palette_list")
    }

    /// Calculate scroll Y offset for a given command index.
    /// Item height = text bounds (size × line_height) + vertical padding + spacing.
    pub fn scroll_offset_for_index(&self, index: usize) -> f32 {
        const DEFAULT_TEXT_SIZE: f32 = 16.0; // Iced default text size
        const LINE_HEIGHT: f32 = 1.3; // Iced default LineHeight::Relative(1.3)
        const PADDING_V: f32 = 16.0; // [8, 12] = 8 top + 8 bottom
        const SPACING: f32 = 2.0;
        let text_height = DEFAULT_TEXT_SIZE * LINE_HEIGHT;
        let item_height = text_height + PADDING_V + SPACING; // ~36.8px
        index as f32 * item_height
    }

    pub fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search commands...", &self.input_value)
            .id(Self::input_id())
            .on_input(|s| Message::Workspace(WorkspaceMessage::CommandPaletteInput(s)))
            .padding(12)
            .accessible_label("Command palette");

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

        let list = scrollable(column(commands))
            .spacing(2)
            .id(Self::scroll_id());

        let content = column![input, list].spacing(8).padding(16);

        container(content)
            .width(500)
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
