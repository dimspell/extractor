use iced::widget::image::Handle;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::components::map_render::EntitySpriteHandle;
use crate::editors::save_file_viewer::map_preview::state::{EntityKind, PreviewEntity};
use crate::editors::save_file_viewer::message::PreviewSpritesLoaded;
use dispel_core::map::sprite_loader::{load_last_frame_of_sequence, load_sprite_frames};
use dispel_core::sprite;
use dispel_core::{Extra, Extractor, MonsterIni, NpcIni};

/// Async-load entity sprites for the map preview.
///
/// Reads `Monster.ini` / `Npc.ini` / `Extra.ini` to map entity DB IDs → sprite
/// filenames, then decodes frame[0] of each unique `.spr` file.  Returns a
/// `Vec` parallel to `entity_markers` (None for entities without a resolvable
/// sprite).
pub async fn load_preview_sprites(
    game_path: PathBuf,
    entity_markers: Vec<PreviewEntity>,
) -> Result<PreviewSpritesLoaded, String> {
    // 1. Load Monster.ini → HashMap<id, sprite_filename>
    let monster_id_to_sprite: HashMap<i32, String> =
        MonsterIni::read_file(&game_path.join("Monster.ini"))
            .map_err(|e| format!("Failed to load Monster.ini: {}", e))?
            .into_iter()
            .filter_map(|m| m.sprite_filename.map(|s| (m.id, s)))
            .collect();

    // 2. Load Npc.ini → HashMap<id, sprite_filename>
    let npc_id_to_sprite: HashMap<i32, String> = NpcIni::read_file(&game_path.join("Npc.ini"))
        .map_err(|e| format!("Failed to load Npc.ini: {}", e))?
        .into_iter()
        .filter_map(|n| n.sprite_filename.map(|s| (n.id, s)))
        .collect();

    // 3. Load Extra.ini → HashMap<id, sprite_filename>
    let extra_id_to_sprite: HashMap<i32, String> = load_extra_ini_sprites(&game_path)
        .map_err(|e| format!("Failed to load Extra.ini: {}", e))?;

    // 4. Resolve sprites for each entity (parallel to entity_markers)
    // Cache key includes `is_dead` and `look_direction` because the same sprite
    // path can be shared by entities in different states (alive vs dead) or
    // facing different directions — without this the first loaded variant would
    // be reused for all others, showing the wrong frame or flip.
    let mut sprite_cache: HashMap<(PathBuf, bool, u8), Option<EntitySpriteHandle>> = HashMap::new();
    let sprites: Vec<Option<EntitySpriteHandle>> = entity_markers
        .iter()
        .map(|entity| {
            let db_id = entity.db_id?;
            let (sub_dir, id_to_sprite) = match entity.kind {
                EntityKind::Monster => ("MonsterInGame", &monster_id_to_sprite),
                EntityKind::Npc => ("NpcInGame", &npc_id_to_sprite),
                EntityKind::Extra => ("ExtraInGame", &extra_id_to_sprite),
                EntityKind::DrawItem => return None,
            };
            // The save file stores the Monster.db ID (0-based archetype index),
            // but Monster.ini / .ref files are keyed by the visual ID which is
            // offset by one (e.g. db 24 → ini 25). Translate before lookup.
            let lookup_id = if matches!(entity.kind, EntityKind::Monster) {
                db_id + 1
            } else {
                db_id
            };
            let sprite_name = id_to_sprite.get(&lookup_id)?;
            let path = resolve_sprite_path(&game_path, sub_dir, sprite_name)?;
            // Dead monsters render the LAST frame of the LAST sequence (the
            // death animation's final "corpse" pose).  Alive entities use the
            // NPC looking-direction formula (mirrors map_editor/update/map.rs)
            // to select a sprite sequence + flip.
            sprite_cache
                .entry((path.clone(), entity.is_dead, entity.look_direction))
                .or_insert_with(|| {
                    let frame = if entity.is_dead {
                        let seq_count = sprite::read_sprite_file(&path)
                            .ok()
                            .map(|sf| sf.sequences.len())
                            .unwrap_or(0);
                        if seq_count == 0 {
                            return None;
                        }
                        load_last_frame_of_sequence(&path, seq_count - 1)?
                    } else {
                        // Compute (sequence, flip) from looking direction,
                        // mirroring the map editor's formula in map.rs:473-479.
                        let dir = entity.look_direction;
                        let (seq, flip) = if dir > 4 {
                            ((8 - dir) as usize, true)
                        } else {
                            (dir as usize, false)
                        };
                        let frames = load_sprite_frames(&path)?;
                        let frame = frames.get(seq).or_else(|| frames.first())?;
                        let w = frame.image.width();
                        let h = frame.image.height();
                        return Some(EntitySpriteHandle {
                            handle: Handle::from_rgba(w, h, frame.image.as_raw().to_vec()),
                            width: w,
                            height: h,
                            origin_x: frame.origin_x,
                            origin_y: frame.origin_y,
                            flip,
                        });
                    };
                    let w = frame.image.width();
                    let h = frame.image.height();
                    Some(EntitySpriteHandle {
                        handle: Handle::from_rgba(w, h, frame.image.as_raw().to_vec()),
                        width: w,
                        height: h,
                        origin_x: frame.origin_x,
                        origin_y: frame.origin_y,
                        flip: false,
                    })
                })
                .clone()
        })
        .collect();

    Ok(PreviewSpritesLoaded { sprites })
}

/// Load Extra.ini → `HashMap<id, sprite_filename>`.
///
/// Tries `Extra::read_file()` (EUC-KR encoding per struct definition) first.
/// If the declared encoding rejects the file (Polish game version uses
/// WINDOWS-1250 for non-ASCII description fields), falls back to a raw-ASCII
/// CSV parse — the first two columns (id and sprite_filename) are always
/// pure ASCII and encoding-independent.
fn load_extra_ini_sprites(game_path: &Path) -> Result<HashMap<i32, String>, String> {
    let path = game_path.join("Extra.ini");
    // Try canonical Extractor read (EUC-KR encoding) first.
    if let Ok(extras) = Extra::read_file(&path) {
        return Ok(extras
            .into_iter()
            .filter_map(|e| e.sprite_filename.map(|s| (e.id, s)))
            .collect());
    }
    // Fallback: raw-bytes CSV parse (encoding-agnostic).
    let data = std::fs::read(&path).map_err(|e| format!("Cannot read Extra.ini: {}", e))?;
    let text = String::from_utf8_lossy(&data);
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let mut cols = line.splitn(4, ',');
        let id: i32 = match cols.next().and_then(|s| s.trim().parse().ok()) {
            Some(id) => id,
            None => continue,
        };
        let sprite = cols
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "null");
        if let Some(s) = sprite {
            map.insert(id, s);
        }
    }
    Ok(map)
}

/// Case-insensitive sprite file path resolution.
///
/// Tries original → uppercase → lowercase under `game_path/{sub_dir}/{filename}`.
fn resolve_sprite_path(game_path: &Path, sub_dir: &str, filename: &str) -> Option<PathBuf> {
    let base = game_path.join(sub_dir);
    for name in [
        filename.to_string(),
        filename.to_ascii_uppercase(),
        filename.to_ascii_lowercase(),
    ] {
        let p = base.join(&name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}
