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
        SelectedEntity::DrawItem(i) => state
            .data
            .draw_items
            .get(i)
            .map(|d| d.get_field(&field))
            .unwrap_or_default(),
        SelectedEntity::EventTile(tx, ty) => state
            .map_data()
            .and_then(|h| h.0.events.get(&(tx, ty)))
            .map(|e| e.event_id.to_string())
            .unwrap_or_default(),
        SelectedEntity::CollisionTile(_, _) => String::new(),
    };
    // Apply the change; track whether the entity was actually found/mutated
    // so we don't push an undo action for a no-op (out-of-bounds index).
    let mut entity_mutated = true;
    match entity {
        SelectedEntity::Monster(i) => {
            if let Some(m) = state.data.monsters.get_mut(i) {
                m.set_field(&field, value.clone());
            } else {
                entity_mutated = false;
            }
        }
        SelectedEntity::Npc(i) => {
            if let Some(n) = state.data.npcs.get_mut(i) {
                n.set_field(&field, value.clone());
            } else {
                entity_mutated = false;
            }
        }
        SelectedEntity::Extra(i) => {
            if let Some(e) = state.data.extra_refs.get_mut(i) {
                e.set_field(&field, value.clone());
            } else {
                entity_mutated = false;
            }
        }
        SelectedEntity::DrawItem(i) => {
            if let Some(d) = state.data.draw_items.get_mut(i) {
                d.set_field(&field, value.clone());
            } else {
                entity_mutated = false;
            }
        }
        SelectedEntity::EventTile(tx, ty) => {
            if !state.data.can_mutate_map_data() {
                state.data.status_msg =
                    Some("Cannot edit events while save/export is in progress".into());
                entity_mutated = false;
            } else if let LoadingState::Loaded(ref mut handle) = state.data.loading_state {
                let map_data = Arc::get_mut(&mut handle.0)
                    .expect("MapData Arc has unexpected shared reference");
                let ev = map_data.events.entry((tx, ty)).or_insert(EventBlock {
                    x: tx,
                    y: ty,
                    _unknown_value: 0,
                    event_id: 0,
                });
                ev.event_id = value.parse::<i16>().unwrap_or(0);
            } else {
                entity_mutated = false;
            }
        }
        SelectedEntity::CollisionTile(_, _) => {
            entity_mutated = false;
        }
    }
    if entity_mutated && old_value != value {
        state.push_undo(MapEditAction {
            entity,
            field: field.clone(),
            old_value,
            new_value: value,
        });

        // When an NPC's waypoint1_facing_direction or npc_id changes, re-resolve its
        // sprite so the map canvas reflects the new state immediately.
        if (field == "waypoint1_facing_direction" || field == "npc_ini_id")
            && let SelectedEntity::Npc(i) = entity
            && let Some(ref game_path) = app.state.workspace.game_path
        {
            state.data.recompute_npc_sprite(i, game_path);
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
        let needs_sprite_update =
            action.field == "waypoint1_facing_direction" || action.field == "npc_ini_id";
        let npc_idx = if needs_sprite_update {
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
            SelectedEntity::DrawItem(i) => {
                if let Some(d) = state.data.draw_items.get_mut(i) {
                    d.set_field(&action.field, action.old_value);
                }
            }
            SelectedEntity::CollisionTile(tx, ty) => {
                if state.data.can_mutate_map_data()
                    && let LoadingState::Loaded(ref mut handle) = state.data.loading_state
                {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.old_value.parse::<bool>().unwrap_or(false);
                    map_data.collisions.insert((tx, ty), val);
                }
            }
            SelectedEntity::EventTile(tx, ty) => {
                if state.data.can_mutate_map_data()
                    && let LoadingState::Loaded(ref mut handle) = state.data.loading_state
                {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.old_value.parse::<i16>().unwrap_or(0);
                    // The event may have been removed from the map since
                    // the undo was recorded (e.g. the user saved the map
                    // and the entry was culled). Re-insert it so undo
                    // always succeeds.
                    let ev =
                        map_data
                            .events
                            .entry((tx, ty))
                            .or_insert(dispel_core::map::EventBlock {
                                x: tx,
                                y: ty,
                                _unknown_value: 0,
                                event_id: 0,
                            });
                    ev.event_id = val;
                }
            }
        }
        if let Some(idx) = npc_idx
            && let Some(ref game_path) = app.state.workspace.game_path
        {
            state.data.recompute_npc_sprite(idx, game_path);
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
        let needs_sprite_update =
            action.field == "waypoint1_facing_direction" || action.field == "npc_ini_id";
        let npc_idx = if needs_sprite_update {
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
            SelectedEntity::DrawItem(i) => {
                if let Some(d) = state.data.draw_items.get_mut(i) {
                    d.set_field(&action.field, action.new_value);
                }
            }
            SelectedEntity::CollisionTile(tx, ty) => {
                if state.data.can_mutate_map_data()
                    && let LoadingState::Loaded(ref mut handle) = state.data.loading_state
                {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.new_value.parse::<bool>().unwrap_or(false);
                    map_data.collisions.insert((tx, ty), val);
                }
            }
            SelectedEntity::EventTile(tx, ty) => {
                if state.data.can_mutate_map_data()
                    && let LoadingState::Loaded(ref mut handle) = state.data.loading_state
                {
                    let map_data = Arc::get_mut(&mut handle.0)
                        .expect("MapData Arc has unexpected shared reference");
                    let val = action.new_value.parse::<i16>().unwrap_or(0);
                    if let Some(ev) = map_data.events.get_mut(&(tx, ty)) {
                        ev.event_id = val;
                    }
                }
            }
        }

        // Recompute the NPC sprite if waypoint1_facing_direction was re-applied.
        if let Some(idx) = npc_idx
            && let Some(ref game_path) = app.state.workspace.game_path
        {
            state.data.recompute_npc_sprite(idx, game_path);
        }

        state.view.tile_layer_cache.clear();
        state.view.overlay_cache.clear();
        super::set_tab_modified(app, tab_id, true);
    }
    Task::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    /// Create an App with a map editor tab that has one DrawItem.
    fn app_with_draw_item() -> (App, usize) {
        let tab_id = 42;
        let mut app = App::new().0;
        let state = app.state.editors.map_editors.entry(tab_id).or_default();

        state.data.all_map_id = Some(0);
        state.data.draw_items = vec![dispel_core::DrawItem {
            map_id: 0,
            x_coord: 10,
            y_coord: 20,
            item: dispel_core::InventoryItem::new(
                dispel_core::references::enums::ItemTypeId::Event,
                5,
            ),
        }];
        // Also need a tab in workspace so set_tab_modified doesn't silently skip
        app.state
            .workspace
            .tabs
            .push(crate::workspace::WorkspaceTab {
                id: tab_id,
                label: "test".into(),
                path: None,
                editor_type: crate::workspace::EditorType::MapEditor,
                modified: false,
                pinned: false,
            });
        app.state.workspace.active_tab = Some(0);
        (app, tab_id)
    }

    // ── DrawItem field_changed ───────────────────────────────────────────────────

    #[test]
    fn test_draw_item_field_changed_updates_state() {
        let (mut app, tab_id) = app_with_draw_item();

        let _task = field_changed(
            &mut app,
            tab_id,
            SelectedEntity::DrawItem(0),
            "x_coord".into(),
            "99".into(),
        );

        let state = app.state.editors.map_editors.get(&tab_id).unwrap();
        assert_eq!(state.data.draw_items[0].x_coord, 99);
        assert!(state.data.dirty, "mark dirty after edit");
        assert_eq!(state.data.undo_stack.len(), 1, "undo created");
    }

    #[test]
    fn test_draw_item_field_changed_undo_restores_old_value() {
        let (mut app, tab_id) = app_with_draw_item();

        let _task = field_changed(
            &mut app,
            tab_id,
            SelectedEntity::DrawItem(0),
            "x_coord".into(),
            "99".into(),
        );

        let _task = undo(&mut app, tab_id);

        let state = app.state.editors.map_editors.get(&tab_id).unwrap();
        assert_eq!(state.data.draw_items[0].x_coord, 10);
        assert_eq!(state.data.undo_stack.len(), 0, "undo stack drained");
        assert_eq!(state.data.redo_stack.len(), 1, "redo stack populated");
    }

    #[test]
    fn test_draw_item_field_changed_redo_reapplies_new_value() {
        let (mut app, tab_id) = app_with_draw_item();

        let _task = field_changed(
            &mut app,
            tab_id,
            SelectedEntity::DrawItem(0),
            "x_coord".into(),
            "99".into(),
        );
        let _task = undo(&mut app, tab_id);
        let _task = redo(&mut app, tab_id);

        let state = app.state.editors.map_editors.get(&tab_id).unwrap();
        assert_eq!(state.data.draw_items[0].x_coord, 99);
        assert_eq!(state.data.undo_stack.len(), 1, "back on undo stack");
        assert_eq!(state.data.redo_stack.len(), 0, "redo drained");
    }

    #[test]
    fn test_draw_item_field_changed_out_of_bounds_noop() {
        let (mut app, tab_id) = app_with_draw_item();

        let _task = field_changed(
            &mut app,
            tab_id,
            SelectedEntity::DrawItem(999),
            "x_coord".into(),
            "99".into(),
        );

        let state = app.state.editors.map_editors.get(&tab_id).unwrap();
        assert!(!state.data.dirty);
        assert_eq!(state.data.undo_stack.len(), 0);
    }

    #[test]
    fn test_draw_item_field_changed_same_value_noop() {
        let (mut app, tab_id) = app_with_draw_item();

        let _task = field_changed(
            &mut app,
            tab_id,
            SelectedEntity::DrawItem(0),
            "x_coord".into(),
            "10".into(),
        );

        let state = app.state.editors.map_editors.get(&tab_id).unwrap();
        assert!(!state.data.dirty, "no undo when value unchanged");
        assert_eq!(state.data.undo_stack.len(), 0);
    }

    #[test]
    fn test_draw_item_field_changed_y_coord() {
        let (mut app, tab_id) = app_with_draw_item();

        let _task = field_changed(
            &mut app,
            tab_id,
            SelectedEntity::DrawItem(0),
            "y_coord".into(),
            "77".into(),
        );

        let state = app.state.editors.map_editors.get(&tab_id).unwrap();
        assert_eq!(state.data.draw_items[0].y_coord, 77);
        assert_eq!(state.data.undo_stack.len(), 1);
    }
}
