use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::components::loading_state::LoadingState;
use crate::editors::map_editor::{MapEditAction, SelectedEntity};
use crate::message::Message;
use dispel_core::map::EventBlock;
use iced::Task;
use std::sync::Arc;

pub fn field_changed(
    app: &mut App,
    tab_id: usize,
    entity: SelectedEntity,
    field: String,
    value: String,
) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };
    // Capture old value before mutating (for undo).
    let old_value = match entity {
        SelectedEntity::Monster(i) => state
            .data
            .monsters
            .get(i)
            .map(|m| m.get_field(&field))
            .unwrap_or_default(),
        SelectedEntity::Npc(i) => state
            .data
            .npcs
            .get(i)
            .map(|n| n.get_field(&field))
            .unwrap_or_default(),
        SelectedEntity::Extra(i) => state
            .data
            .extra_refs
            .get(i)
            .map(|e| e.get_field(&field))
            .unwrap_or_default(),
        SelectedEntity::EventTile(tx, ty) => state
            .map_data()
            .and_then(|h| h.0.events.get(&(tx, ty)))
            .map(|e| e.event_id.to_string())
            .unwrap_or_default(),
        SelectedEntity::CollisionTile(_, _) => String::new(),
    };
    // Apply the change.
    match entity {
        SelectedEntity::Monster(i) => {
            if let Some(m) = state.data.monsters.get_mut(i) {
                m.set_field(&field, value.clone());
            }
        }
        SelectedEntity::Npc(i) => {
            if let Some(n) = state.data.npcs.get_mut(i) {
                n.set_field(&field, value.clone());
            }
        }
        SelectedEntity::Extra(i) => {
            if let Some(e) = state.data.extra_refs.get_mut(i) {
                e.set_field(&field, value.clone());
            }
        }
        SelectedEntity::EventTile(tx, ty) => {
            if let LoadingState::Loaded(ref mut handle) = state.data.loading_state {
                let map_data = Arc::get_mut(&mut handle.0)
                    .expect("MapData Arc has unexpected shared reference");
                let ev = map_data
                    .events
                    .entry((tx, ty))
                    .or_insert(EventBlock {
                        x: tx,
                        y: ty,
                        _unknown_value: 0,
                        event_id: 0,
                    });
                ev.event_id = value.parse::<i16>().unwrap_or(0);
            }
        }
        SelectedEntity::CollisionTile(_, _) => {}
    }
    if old_value != value {
        state.push_undo(MapEditAction {
            entity,
            field: field.clone(),
            old_value,
            new_value: value,
        });

        // When an NPC's looking_direction changes, re-resolve its sprite
        // so the map canvas reflects the new direction immediately.
        if field == "looking_direction" {
            if let SelectedEntity::Npc(i) = entity {
                if let Some(ref game_path) = app.state.workspace.game_path {
                    state.data.recompute_npc_sprite(i, game_path);
                }
            }
        }

        // Entity positions live on the tile canvas; selection ring on the overlay.
        state.view.tile_layer_cache.clear();
        state.view.overlay_cache.clear();
        super::set_tab_modified(app, tab_id, true);
    }
    Task::none()
}

pub fn undo(app: &mut App, tab_id: usize) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };
    if let Some(action) = state.pop_undo() {
        // Capture NPC index before the field value is moved.
        let npc_idx = if action.field == "looking_direction" {
            match action.entity {
                SelectedEntity::Npc(i) => Some(i),
                _ => None,
            }
        } else {
            None
        };

        match action.entity {
            SelectedEntity::Monster(i) => {
                if let Some(m) = state.data.monsters.get_mut(i) {
                    m.set_field(&action.field, action.old_value);
                }
            }
            SelectedEntity::Npc(i) => {
                if let Some(n) = state.data.npcs.get_mut(i) {
                    n.set_field(&action.field, action.old_value);
                }
            }
            SelectedEntity::Extra(i) => {
                if let Some(e) = state.data.extra_refs.get_mut(i) {
                    e.set_field(&action.field, action.old_value);
                }
            }
            SelectedEntity::CollisionTile(tx, ty) => {
                if let LoadingState::Loaded(ref mut handle) = state.data.loading_state {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.old_value.parse::<bool>().unwrap_or(false);
                    map_data.collisions.insert((tx, ty), val);
                }
            }
            SelectedEntity::EventTile(tx, ty) => {
                if let LoadingState::Loaded(ref mut handle) = state.data.loading_state {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.old_value.parse::<i16>().unwrap_or(0);
                    if let Some(ev) = map_data.events.get_mut(&(tx, ty)) {
                        ev.event_id = val;
                    }
                }
            }
        }

        // Recompute NPC sprite if looking_direction was reverted.
        if let Some(idx) = npc_idx {
            if let Some(ref game_path) = app.state.workspace.game_path {
                state.data.recompute_npc_sprite(idx, game_path);
            }
        }

        state.view.tile_layer_cache.clear();
        state.view.overlay_cache.clear();
        let still_dirty = !state.data.undo_stack.is_empty();
        super::set_tab_modified(app, tab_id, still_dirty);
    }
    Task::none()
}

pub fn redo(app: &mut App, tab_id: usize) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };
    if let Some(action) = state.pop_redo() {
        // Capture NPC index before the field value is moved.
        let npc_idx = if action.field == "looking_direction" {
            match action.entity {
                SelectedEntity::Npc(i) => Some(i),
                _ => None,
            }
        } else {
            None
        };

        match action.entity {
            SelectedEntity::Monster(i) => {
                if let Some(m) = state.data.monsters.get_mut(i) {
                    m.set_field(&action.field, action.new_value);
                }
            }
            SelectedEntity::Npc(i) => {
                if let Some(n) = state.data.npcs.get_mut(i) {
                    n.set_field(&action.field, action.new_value);
                }
            }
            SelectedEntity::Extra(i) => {
                if let Some(e) = state.data.extra_refs.get_mut(i) {
                    e.set_field(&action.field, action.new_value);
                }
            }
            SelectedEntity::CollisionTile(tx, ty) => {
                if let LoadingState::Loaded(ref mut handle) = state.data.loading_state {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.new_value.parse::<bool>().unwrap_or(false);
                    map_data.collisions.insert((tx, ty), val);
                }
            }
            SelectedEntity::EventTile(tx, ty) => {
                if let LoadingState::Loaded(ref mut handle) = state.data.loading_state {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.new_value.parse::<i16>().unwrap_or(0);
                    if let Some(ev) = map_data.events.get_mut(&(tx, ty)) {
                        ev.event_id = val;
                    }
                }
            }
        }

        // Recompute NPC sprite if looking_direction was re-applied.
        if let Some(idx) = npc_idx {
            if let Some(ref game_path) = app.state.workspace.game_path {
                state.data.recompute_npc_sprite(idx, game_path);
            }
        }

        state.view.tile_layer_cache.clear();
        state.view.overlay_cache.clear();
        super::set_tab_modified(app, tab_id, true);
    }
    Task::none()
}
