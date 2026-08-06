// ── MapCanvasTilesLayer — delegates to shared GenericTilesLayer ───────────────

pub type MapCanvasTilesLayer<'a> = crate::components::map_render::GenericTilesLayer<
    'a,
    crate::editors::map_editor::state::MapEditorState,
>;
