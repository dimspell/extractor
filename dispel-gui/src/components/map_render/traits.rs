use crate::components::map_render::{EntitySpriteHandle, InternalSpriteHandle, MapViewState};
use crate::editors::map_editor::message::MapDataHandle;
use iced::widget::image::Handle;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Monster,
    Npc,
    Extra,
    DrawItem,
}

impl EntityKind {
    /// Category tiebreaker for the interlaced Y-sort (lower draws first).
    ///
    /// Rungs: buildings (0) < internal sprites (1) < extras/props (2) <
    /// draw items (3) < monsters (4) < NPCs (5).
    ///
    /// When entities share a tile their Y keys and tile X are equal, so this
    /// rung decides: e.g. an NPC sitting on a chair draws over it, and a drop
    /// item lying on a prop draws over the prop.
    pub fn type_order(self) -> i32 {
        match self {
            EntityKind::Extra => 2,
            EntityKind::DrawItem => 3,
            EntityKind::Monster => 4,
            EntityKind::Npc => 5,
        }
    }
}

pub struct EntityRenderData<'a> {
    pub tile_x: i32,
    pub tile_y: i32,
    pub sort_key: i32,
    pub sprite: Option<&'a EntitySpriteHandle>,
    pub kind: EntityKind,
    pub visible: bool,
}

pub trait MapRenderSource {
    /// Lighting fade tables for the shadow pass; `None` disables shadows.
    fn shadow_data(&self) -> Option<&std::sync::Arc<dispel_core::map::render::FogData>> {
        None
    }

    fn map_data(&self) -> Option<&MapDataHandle>;
    fn gtl_handles(&self) -> &HashMap<i32, Handle>;
    fn btl_handles(&self) -> &HashMap<i32, Handle>;
    fn tiles_ready(&self) -> bool;
    fn view(&self) -> &MapViewState;
    fn internal_sprite_handles(&self) -> &[InternalSpriteHandle];
    fn entity_count(&self) -> usize;
    fn entity_data(&self, idx: usize) -> Option<EntityRenderData<'_>>;
}

#[cfg(test)]
mod tests {
    use super::EntityKind;

    /// Regression: an NPC placed on an Extra (sitting on a chair) must render
    /// over it. Both tie on Y key and tile X, so the kind rung decides.
    #[test]
    fn test_entity_type_order_npc_draws_over_extra() {
        assert!(EntityKind::Npc.type_order() > EntityKind::Extra.type_order());
    }

    /// Regression: a drop item lying on a prop must render over the prop.
    #[test]
    fn test_entity_type_order_draw_item_draws_over_extra() {
        assert!(EntityKind::DrawItem.type_order() > EntityKind::Extra.type_order());
    }

    #[test]
    fn test_entity_type_order_ladder_is_total() {
        let order = [
            EntityKind::Extra,
            EntityKind::DrawItem,
            EntityKind::Monster,
            EntityKind::Npc,
        ];
        for pair in order.windows(2) {
            assert!(
                pair[0].type_order() < pair[1].type_order(),
                "{:?} must draw before {:?}",
                pair[0],
                pair[1]
            );
        }
        // Entities stay above buildings (0) and internal sprites (1).
        assert!(order[0].type_order() > 1);
    }
}
