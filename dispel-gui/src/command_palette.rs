use crate::message::Message;
use crate::style;
use crate::types::Tab;
use iced::widget::{button, column, container, row, text, text_input};
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
                action: || Message::Undo,
            },
            Command {
                id: "redo",
                label: "Redo",
                shortcut: Some("Ctrl+Y"),
                action: || Message::Redo,
            },
            Command {
                id: "toggle-history",
                label: "Toggle Edit History",
                shortcut: Some("Ctrl+H"),
                action: || Message::ToggleHistoryPanel,
            },
            Command {
                id: "toggle-command-palette",
                label: "Toggle Command Palette",
                shortcut: Some("Ctrl+P"),
                action: || Message::ToggleCommandPalette,
            },
            Command {
                id: "goto-map",
                label: "Go to Map",
                shortcut: None,
                action: || Message::TabSelected(Tab::Map),
            },
            Command {
                id: "goto-ref",
                label: "Go to Ref",
                shortcut: None,
                action: || Message::TabSelected(Tab::Ref),
            },
            Command {
                id: "goto-database",
                label: "Go to Database",
                shortcut: None,
                action: || Message::TabSelected(Tab::Database),
            },
            Command {
                id: "goto-dbviewer",
                label: "Go to DbViewer",
                shortcut: None,
                action: || Message::TabSelected(Tab::DbViewer),
            },
            Command {
                id: "goto-chest-editor",
                label: "Go to Chest Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::ChestEditor),
            },
            Command {
                id: "goto-weapon-editor",
                label: "Go to Weapon Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::WeaponEditor),
            },
            Command {
                id: "goto-heal-item-editor",
                label: "Go to Heal Item Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::HealItemEditor),
            },
            Command {
                id: "goto-misc-item-editor",
                label: "Go to Misc Item Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::MiscItemEditor),
            },
            Command {
                id: "goto-edit-item-editor",
                label: "Go to Edit Item Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::EditItemEditor),
            },
            Command {
                id: "goto-event-item-editor",
                label: "Go to Event Item Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::EventItemEditor),
            },
            Command {
                id: "goto-monster-editor",
                label: "Go to Monster Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::MonsterEditor),
            },
            Command {
                id: "goto-npc-ini-editor",
                label: "Go to NPC Ini Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::NpcIniEditor),
            },
            Command {
                id: "goto-magic-editor",
                label: "Go to Magic Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::MagicEditor),
            },
            Command {
                id: "goto-store-editor",
                label: "Go to Store Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::StoreEditor),
            },
            Command {
                id: "goto-party-ref-editor",
                label: "Go to Party Ref Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::PartyRefEditor),
            },
            Command {
                id: "goto-party-ini-editor",
                label: "Go to Party Ini Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::PartyIniEditor),
            },
            Command {
                id: "goto-sprite-browser",
                label: "Go to Sprite Browser",
                shortcut: None,
                action: || Message::TabSelected(Tab::SpriteBrowser),
            },
            Command {
                id: "goto-monster-ref-editor",
                label: "Go to Monster Ref Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::MonsterRefEditor),
            },
            Command {
                id: "goto-all-map-ini-editor",
                label: "Go to All Map Ini Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::AllMapIniEditor),
            },
            Command {
                id: "goto-dialog-editor",
                label: "Go to Dialog Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::DialogEditor),
            },
            Command {
                id: "goto-dialogue-text-editor",
                label: "Go to Dialogue Text Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::DialogueTextEditor),
            },
            Command {
                id: "goto-draw-item-editor",
                label: "Go to Draw Item Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::DrawItemEditor),
            },
            Command {
                id: "goto-event-ini-editor",
                label: "Go to Event Ini Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::EventIniEditor),
            },
            Command {
                id: "goto-event-npc-ref-editor",
                label: "Go to Event Npc Ref Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::EventNpcRefEditor),
            },
            Command {
                id: "goto-extra-ini-editor",
                label: "Go to Extra Ini Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::ExtraIniEditor),
            },
            Command {
                id: "goto-extra-ref-editor",
                label: "Go to Extra Ref Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::ExtraRefEditor),
            },
            Command {
                id: "goto-map-ini-editor",
                label: "Go to Map Ini Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::MapIniEditor),
            },
            Command {
                id: "goto-message-scr-editor",
                label: "Go to Message Scr Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::MessageScrEditor),
            },
            Command {
                id: "goto-npc-ref-editor",
                label: "Go to Npc Ref Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::NpcRefEditor),
            },
            Command {
                id: "goto-party-level-db-editor",
                label: "Go to Party Level Db Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::PartyLevelDbEditor),
            },
            Command {
                id: "goto-quest-scr-editor",
                label: "Go to Quest Scr Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::QuestScrEditor),
            },
            Command {
                id: "goto-wave-ini-editor",
                label: "Go to Wave Ini Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::WaveIniEditor),
            },
            Command {
                id: "goto-chdata-editor",
                label: "Go to ChData Editor",
                shortcut: None,
                action: || Message::TabSelected(Tab::ChDataEditor),
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

        let query_lower = query.to_lowercase();
        self.filtered_commands = self
            .all_commands
            .iter()
            .filter(|cmd| {
                cmd.label.to_lowercase().contains(&query_lower) || cmd.id.contains(&query_lower)
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

    pub fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search commands...", &self.input_value)
            .on_input(Message::CommandPaletteInput)
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
                    .on_press(Message::CommandPaletteSelect(idx))
                    .style(if is_selected {
                        style::selected_button
                    } else {
                        style::chip
                    })
                    .into()
            })
            .collect();

        let list = column(commands).spacing(2);

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
