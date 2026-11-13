//! The viewport light template (kb/11 §10.13).
//!
//! The game never reads light levels from the map: the world tick rewrites
//! bits 15–29 of every visible tile's access word **every frame** from a
//! static 500-byte table (`DAT_004a2c68` in `Dispel_145_wm.exe`). The table
//! is indexed by *visible slot* — a 32-row tile diamond anchored to the
//! camera tile:
//!
//! ```text
//! row runs: 1,3,…,27 (rows 0–13) · 27×5 (rows 14–18) · 25,23,…,1 (rows 19–31)
//! Σ = 500
//! ```
//!
//! The top half is anchored to the camera tile, the bottom half to a second
//! anchor at top + (−13, +13). Rows 0–12 hold `dx ∈ [−dy, dy]`, rows 13–18
//! hold `dx ∈ [dy−26, dy]`, rows 19–31 hold `dx ∈ [dy−26, dy−26+run−1]`.
//!
//! Because the `.map` file's own light bits are dead data, a static render
//! that uses them shows every dark map as pitch black. Feeding the template
//! in reproduces the lighting the game actually draws.

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::path::Path;

use crate::map::types::Coords;

/// Number of visible slots the template covers.
pub const SLOT_COUNT: usize = 500;

/// Row runs of the visible-slot diamond (rows 0–31).
const ROW_RUNS: [usize; 32] = [
    1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 27, 27, 27, 27, 27, 25, 23, 21, 19, 17, 15,
    13, 11, 9, 7, 5, 3, 1,
];

/// The 500-byte light template (one level per visible slot).
#[derive(Debug, Clone)]
pub struct LightTemplate {
    levels: Vec<u8>,
}

impl LightTemplate {
    /// Loads the template from a 500×1 grayscale PNG (the format
    /// `tools/convert_lighting.py` writes) or from a raw 500-byte file.
    pub fn load(path: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(path)?;
        if bytes.len() == SLOT_COUNT {
            return Ok(Self { levels: bytes });
        }
        let img = image::load_from_memory(&bytes)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .to_luma8();
        if img.width() as usize != SLOT_COUNT || img.height() != 1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "light template must be {SLOT_COUNT}×1 (got {}×{})",
                    img.width(),
                    img.height()
                ),
            ));
        }
        Ok(Self {
            levels: img.into_raw(),
        })
    }

    /// Builds a template from raw slot levels (tests).
    pub fn from_levels(levels: Vec<u8>) -> Self {
        Self { levels }
    }

    /// The level of one visible slot.
    pub fn level(&self, slot: usize) -> u8 {
        self.levels.get(slot).copied().unwrap_or(0)
    }

    /// The lattice offset `(dx, dy)` of a visible slot, relative to the
    /// camera tile, or `None` for an out-of-range slot.
    pub fn slot_offset(slot: usize) -> Option<(i32, i32)> {
        if slot >= SLOT_COUNT {
            return None;
        }
        let mut row = 0usize;
        let mut base = 0usize;
        while row < ROW_RUNS.len() && base + ROW_RUNS[row] <= slot {
            base += ROW_RUNS[row];
            row += 1;
        }
        if row >= ROW_RUNS.len() {
            return None;
        }
        let i = (slot - base) as i32;
        let r = row as i32;
        if r < 13 {
            Some((i - r, r))
        } else {
            Some((r - 26 + i, r))
        }
    }

    /// Writes the template's levels into the map's access words (bits 15–29),
    /// anchored at `camera`. Tiles outside the visible diamond keep their own
    /// (dead) bits.
    pub fn apply(&self, words: &mut HashMap<Coords, u32>, camera: Coords) {
        for slot in 0..SLOT_COUNT {
            let Some((dx, dy)) = Self::slot_offset(slot) else {
                continue;
            };
            let coords = (camera.0 + dx, camera.1 + dy);
            let word = words.entry(coords).or_insert(0);
            let level = u32::from(self.level(slot)) & 0x7FFF;
            *word = (*word & !(0x7FFF_u32 << 15)) | (level << 15);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_offsets_follow_the_visible_diamond() {
        // Slot 0 is the camera tile itself.
        assert_eq!(LightTemplate::slot_offset(0), Some((0, 0)));
        // Row 1 holds two tiles (run 3 → slots 1..3): dx −1, 0, +1.
        assert_eq!(LightTemplate::slot_offset(1), Some((-1, 1)));
        assert_eq!(LightTemplate::slot_offset(2), Some((0, 1)));
        assert_eq!(LightTemplate::slot_offset(3), Some((1, 1)));
        // The bottom half uses the second anchor: row 13 starts at dx = −13.
        assert_eq!(LightTemplate::slot_offset(169), Some((-13, 13)));
        // The last slot is the bottom tip of the diamond (row 31, one tile).
        assert_eq!(LightTemplate::slot_offset(SLOT_COUNT - 1), Some((5, 31)));
        assert_eq!(LightTemplate::slot_offset(SLOT_COUNT), None);
    }

    #[test]
    fn apply_writes_the_level_bits_only() {
        let mut words: HashMap<Coords, u32> = HashMap::new();
        words.insert((0, 0), 0xC000_1234); // bits 30-31 set, low bits set
        let template = LightTemplate::from_levels(vec![200; SLOT_COUNT]);
        template.apply(&mut words, (0, 0));
        let word = words[&(0, 0)];
        assert_eq!(word & 0x7FFF, 0x1234, "the low bits survive");
        assert_eq!(
            word & 0xC000_0000,
            0xC000_0000,
            "the light-source bits survive"
        );
        assert_eq!((word >> 15) & 0x7FFF, 200, "the level is written");
    }
}
