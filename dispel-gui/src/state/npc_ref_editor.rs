use crate::generic_editor::MultiFileEditorState;
use dispel_core::NPC;

pub type NpcRefEditorState = MultiFileEditorState<NPC>;

impl NpcRefEditorState {
    pub fn refresh_npcs(&mut self) {
        self.editor.refresh();
    }

    pub fn select_npc(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_npcs(&self) -> Result<(), String> {
        self.save()
    }
}
