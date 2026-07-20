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

pub struct EntityRenderData<'a> {
    pub tile_x: i32,
    pub tile_y: i32,
    pub sort_key: i32,
    pub sprite: Option<&'a EntitySpriteHandle>,
    pub kind: EntityKind,
    pub visible: bool,
}

pub trait MapRenderSource {
    fn map_data(&self) -> Option<&MapDataHandle>;
    fn gtl_handles(&self) -> &HashMap<i32, Handle>;
    fn btl_handles(&self) -> &HashMap<i32, Handle>;
    fn tiles_ready(&self) -> bool;
    fn view(&self) -> &MapViewState;
    fn internal_sprite_handles(&self) -> &[InternalSpriteHandle];
    fn entity_count(&self) -> usize;
    fn entity_data(&self, idx: usize) -> Option<EntityRenderData<'_>>;
}
