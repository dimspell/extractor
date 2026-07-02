//! TMX (Tiled Map Editor) export module.
//!
//! Exports Dispel `.map` data to the Tiled TMX XML format using orthogonal
//! tile layers and CSV-encoded tile data. Tiles are exported as raw 32×32
//! square pixels (no isometric diamond mask).
//!
//! # Usage
//!
//! ```ignore
//! use std::path::Path;
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! let map_path = Path::new("map/field01.map");
//! let gtl_path = Path::new("map/field01.gtl");
//! let btl_path = Path::new("map/field01.btl");
//! let out_dir = Path::new("tmx_output");
//!
//! let file = File::open(map_path).unwrap();
//! let mut reader = BufReader::new(file);
//! let map_data = crate::map::read_map_data(&mut reader).unwrap();
//!
//! crate::map::tmx::export_tmx(&map_data, gtl_path, btl_path, out_dir).unwrap();
//! ```

use super::tileset::Tile;
use super::MapData;
use crate::sprite::Color;
use std::io::{BufWriter, Write};
use std::path::Path;

use image::RgbaImage;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Plots a single tile as raw 32×32 square pixels (no isometric diamond mask).
///
/// Pixels are in row-major order (`colors[y * 32 + x]`). Pure-black pixels
/// (`r = g = b = 0`) are treated as transparent and skipped.  This matches
/// the convention established by [`super::tileset::plot_tile_rgba`] but
/// writes every pixel directly without the isometric diamond mask.
fn plot_tile_square_rgba(
    imgbuf: &mut RgbaImage,
    colors: [Color; 1024],
    dest_x: i32,
    dest_y: i32,
) {
    let img_w = imgbuf.width() as i32;
    let img_h = imgbuf.height() as i32;

    for row in 0..32 {
        for col in 0..32 {
            let idx = (row * 32 + col) as usize;
            let pixel = colors[idx];

            // Pure black is assumed to be transparent in the game's tile
            // format — skip it just like plot_tile / plot_tile_rgba do.
            if pixel.r == 0 && pixel.g == 0 && pixel.b == 0 {
                continue;
            }

            let fx = dest_x + col;
            let fy = dest_y + row;

            if fx >= 0 && fx < img_w && fy >= 0 && fy < img_h {
                imgbuf.put_pixel(
                    fx as u32,
                    fy as u32,
                    image::Rgba([pixel.r, pixel.g, pixel.b, 255]),
                );
            }
        }
    }
}

/// Writes a tileset atlas PNG image with 32×32 square tiles arranged in a
/// fixed-width grid.
///
/// # Layout
///
/// Uses 48 tiles per row (matching the convention of
/// [`super::tileset::plot_tileset_map`]), producing an image whose width is
/// `48 × 32 = 1536` pixels and whose height is a multiple of 32.
fn write_tileset_atlas(tiles: &[Tile], path: &Path) -> std::io::Result<()> {
    let tiles_per_row: u32 = 48;

    if tiles.is_empty() {
        // Write a minimal 1×1 pixel PNG so we never create a zero-size image
        // (which would panic in `RgbaImage::new`).
        let bitmap = RgbaImage::new(1, 1);
        bitmap
            .save(path)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        return Ok(());
    }

    let rows = (tiles.len() as f64 / tiles_per_row as f64).ceil() as u32;
    let width = tiles_per_row * 32;
    let height = rows * 32;

    let mut bitmap = RgbaImage::new(width, height);

    for (idx, tile) in tiles.iter().enumerate() {
        let tx = (idx as u32 % tiles_per_row) * 32;
        let ty = (idx as u32 / tiles_per_row) * 32;
        plot_tile_square_rgba(&mut bitmap, tile.colors, tx as i32, ty as i32);
    }

    bitmap
        .save(path)
        .map_err(|e| std::io::Error::other(e.to_string()))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Export a Dispel `.map` to the Tiled TMX format.
///
/// # Arguments
///
/// * `map_data` — parsed `.map` data (see [`super::read_map_data`])
/// * `gtl_path` — path to the `.GTL` ground tileset file
/// * `btl_path` — path to the `.BTL` roof / building tileset file
/// * `output_dir` — directory where the `.tmx` file and tileset `.png` files
///   are written (created if it does not exist)
///
/// # Generated Files
///
/// | File | Description |
/// |------|-------------|
/// | `{stem}.tmx` | Main TMX map (orthogonal, CSV-encoded tile layers) |
/// | `{stem}_ground.png` | GTL tileset atlas (32×32 square tiles, 48/row) |
/// | `{stem}_roof.png` | BTL tileset atlas (32×32 square tiles, 48/row) |
///
/// The `stem` is derived from the `gtl_path` file stem (e.g. `cat1`).
///
/// # Layers and Object Groups
///
/// | ID | Name | Type | Contents |
/// |----|------|------|----------|
/// | 1 | Ground | tile layer | GTL tile GIDs |
/// | 2 | Roofs | tile layer | BTL tile GIDs |
/// | 3 | Collisions | object group | 32×32 rectangles at blocked tiles |
/// | 4 | Events | object group | point objects with `event_id` property |
/// | 5 | TiledObjects | object group | 32×32 rectangles at building bases |
///
/// # GID Mapping
///
/// * GTL firstgid = 1
/// * BTL firstgid = `gtl_tile_count + 1`
/// * Empty / no tile → GID 0
///
/// Tile index 0 in the game format means "no tile" (empty).
pub fn export_tmx(
    map_data: &MapData,
    gtl_path: &Path,
    btl_path: &Path,
    output_dir: &Path,
) -> std::io::Result<()> {
    let map_name = gtl_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("map");

    // ---- Read tilesets ---------------------------------------------------
    let gtl_tileset = super::tileset::extract(gtl_path)?;
    let btl_tileset = super::tileset::extract(btl_path)?;

    std::fs::create_dir_all(output_dir)?;

    // ---- Write tileset PNGs ----------------------------------------------
    let ground_png = output_dir.join(format!("{}_ground.png", map_name));
    let roof_png = output_dir.join(format!("{}_roof.png", map_name));
    write_tileset_atlas(&gtl_tileset, &ground_png)?;
    write_tileset_atlas(&btl_tileset, &roof_png)?;

    // ---- Build TMX XML ---------------------------------------------------

    let tiles_per_row: u32 = 48;
    let gtl_count = gtl_tileset.len();
    let btl_count = btl_tileset.len();

    // Atlas image dimensions (defensive min size to avoid zero-dimension PNG).
    let gtl_rows = (gtl_count as f64 / tiles_per_row as f64).ceil() as u32;
    let gtl_img_w = (tiles_per_row * 32).max(1);
    let gtl_img_h = (gtl_rows * 32).max(1);

    let btl_rows = (btl_count as f64 / tiles_per_row as f64).ceil() as u32;
    let btl_img_w = (tiles_per_row * 32).max(1);
    let btl_img_h = (btl_rows * 32).max(1);

    let map_w = map_data.model.tiled_map_width;
    let map_h = map_data.model.tiled_map_height;
    let gtl_firstgid: u32 = 1;
    let btl_firstgid: u32 = 1 + gtl_count as u32;

    let tmx_path = output_dir.join(format!("{}.tmx", map_name));
    let file = std::fs::File::create(&tmx_path)?;
    let mut w = BufWriter::new(file);

    // -- Map + tileset headers ---------------------------------------------
    write!(
        w,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" tiledversion="1.11" orientation="orthogonal" \
 renderorder="right-down" width="{}" height="{}" tilewidth="32" \
 tileheight="32" infinite="0">
  <tileset firstgid="{}" name="ground" tilewidth="32" tileheight="32" \
 tilecount="{}" columns="{}">
    <image source="{}_ground.png" width="{}" height="{}"/>
  </tileset>
  <tileset firstgid="{}" name="roof" tilewidth="32" tileheight="32" \
 tilecount="{}" columns="{}">
    <image source="{}_roof.png" width="{}" height="{}"/>
  </tileset>
"#,
        map_w, map_h,
        gtl_firstgid, gtl_count, tiles_per_row,
        map_name, gtl_img_w, gtl_img_h,
        btl_firstgid, btl_count, tiles_per_row,
        map_name, btl_img_w, btl_img_h,
    )?;

    // -- Layer 1: Ground (GTL) --------------------------------------------
    write!(
        w,
        r#"  <layer id="1" name="Ground" width="{}" height="{}">
    <data encoding="csv">
"#,
        map_w, map_h
    )?;

    for y in 0..map_h {
        for x in 0..map_w {
            // tile_id == 0 in the game format means "no tile" → GID 0.
            let gid = match map_data.gtl_tiles.get(&(x, y)) {
                Some(&id) if id > 0 => gtl_firstgid as i32 + id,
                _ => 0,
            };
            write!(w, "{}", gid)?;
            if x < map_w - 1 {
                write!(w, ",")?;
            }
        }
        writeln!(w)?;
    }
    write!(w, "    </data>\n  </layer>\n")?;

    // -- Layer 2: Roofs (BTL) ---------------------------------------------
    write!(
        w,
        r#"  <layer id="2" name="Roofs" width="{}" height="{}">
    <data encoding="csv">
"#,
        map_w, map_h
    )?;

    for y in 0..map_h {
        for x in 0..map_w {
            // BTL entries with tile_id > 0 only (reader skips 0).
            let gid = match map_data.btl_tiles.get(&(x, y)) {
                Some(&id) => btl_firstgid as i32 + id,
                None => 0,
            };
            write!(w, "{}", gid)?;
            if x < map_w - 1 {
                write!(w, ",")?;
            }
        }
        writeln!(w)?;
    }
    write!(w, "    </data>\n  </layer>\n")?;

    // -- Object group 3: Collisions ---------------------------------------
    write!(w, r#"  <objectgroup id="3" name="Collisions">
"#)?;
    {
        let mut obj_id = 1u32;
        for y in 0..map_h {
            for x in 0..map_w {
                if map_data.collisions.get(&(x, y)).copied().unwrap_or(false) {
                    let px = x * 32;
                    let py = y * 32;
                    write!(
                        w,
                        r#"    <object id="{}" x="{}" y="{}" width="32" height="32"/>
"#,
                        obj_id, px, py
                    )?;
                    obj_id += 1;
                }
            }
        }
    }
    write!(w, "  </objectgroup>\n")?;

    // -- Object group 4: Events -------------------------------------------
    write!(w, r#"  <objectgroup id="4" name="Events">
"#)?;
    {
        let mut obj_id = 1u32;
        for y in 0..map_h {
            for x in 0..map_w {
                if let Some(event) = map_data.events.get(&(x, y)) {
                    if event.event_id != 0 {
                        let px = x * 32;
                        let py = y * 32;
                        write!(
                            w,
                            r#"    <object id="{}" x="{}" y="{}" width="32" height="32">
      <properties>
        <property name="event_id" value="{}"/>
      </properties>
    </object>
"#,
                            obj_id, px, py, event.event_id
                        )?;
                        obj_id += 1;
                    }
                }
            }
        }
    }
    write!(w, "  </objectgroup>\n")?;

    // -- Object group 5: TiledObjects (building placements) ----------------
    write!(w, r#"  <objectgroup id="5" name="TiledObjects">
"#)?;
    {
        let mut obj_id = 1u32;
        for obj in &map_data.tiled_infos {
            let px = obj.x * 32;
            let py = obj.y * 32;
            write!(
                w,
                r#"    <object id="{}" x="{}" y="{}" width="32" height="32"/>
"#,
                obj_id, px, py
            )?;
            obj_id += 1;
        }
    }
    write!(w, "  </objectgroup>\n")?;

    // -- Close map root element -------------------------------------------
    write!(w, "</map>\n")?;

    w.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Basic unit test for `plot_tile_square_rgba`: create a 32×32 image,
    /// plot a tile with a single red pixel at (0,0), and verify the pixel.
    #[test]
    fn test_plot_tile_square_rgba_single_pixel() {
        let mut colors = [Color { r: 0, g: 0, b: 0 }; 1024];
        // Set pixel at row 0, col 0 to red.
        colors[0] = Color { r: 255, g: 0, b: 0 };

        let mut img = RgbaImage::new(32, 32);
        plot_tile_square_rgba(&mut img, colors, 0, 0);

        let pixel = img.get_pixel(0, 0);
        assert_eq!(pixel[0], 255, "red channel");
        assert_eq!(pixel[1], 0, "green channel");
        assert_eq!(pixel[2], 0, "blue channel");
        assert_eq!(pixel[3], 255, "alpha channel");

        // Pixel (31, 31) should still be transparent (black was skipped).
        let p2 = img.get_pixel(31, 31);
        assert_eq!(p2[3], 0, "unwritten pixel should stay transparent");
    }

    /// `plot_tile_square_rgba` should skip pure-black pixels.
    #[test]
    fn test_plot_tile_square_rgba_skips_black() {
        let colors = [Color { r: 0, g: 0, b: 0 }; 1024]; // all black
        let mut img = RgbaImage::new(32, 32);
        plot_tile_square_rgba(&mut img, colors, 0, 0);

        // All pixels should remain at the default (0,0,0,0) since every
        // input pixel was pure black → skipped.
        for y in 0..32 {
            for x in 0..32 {
                let p = img.get_pixel(x, y);
                assert_eq!(p[3], 0, "pixel ({},{}) should be transparent", x, y);
            }
        }
    }

    /// `plot_tile_square_rgba` writes all 1024 colour values to the correct
    /// row/col positions.
    #[test]
    fn test_plot_tile_square_rgba_all_positions() {
        // Create a tile where every pixel has a unique colour so we can
        // verify that position (r, c) gets colour (r, c, 0).
        let mut colors = [Color { r: 0, g: 0, b: 0 }; 1024];
        for row in 0..32 {
            for col in 0..32 {
                let idx = row * 32 + col;
                colors[idx] = Color {
                    r: row as u8,
                    g: col as u8,
                    b: 128,
                };
            }
        }

        let mut img = RgbaImage::new(32, 32);
        plot_tile_square_rgba(&mut img, colors, 0, 0);

        for row in 0..32 {
            for col in 0..32 {
                let p = img.get_pixel(col, row);
                assert_eq!(p[0], row as u8, "R at ({},{})", col, row);
                assert_eq!(p[1], col as u8, "G at ({},{})", col, row);
                assert_eq!(p[2], 128, "B at ({},{})", col, row);
                assert_eq!(p[3], 255, "A at ({},{})", col, row);
            }
        }
    }

    /// `plot_tile_square_rgba` handles non-zero destination offsets.
    #[test]
    fn test_plot_tile_square_rgba_offset() {
        let mut colors = [Color { r: 0, g: 0, b: 0 }; 1024];
        colors[0] = Color { r: 255, g: 0, b: 0 };

        let mut img = RgbaImage::new(64, 64);
        plot_tile_square_rgba(&mut img, colors, 16, 16);

        // The red pixel should appear at (16, 16), not at (0, 0).
        let p_at_offset = img.get_pixel(16, 16);
        assert_eq!(p_at_offset[0], 255);

        let p_at_origin = img.get_pixel(0, 0);
        assert_eq!(p_at_origin[3], 0, "origin should remain transparent");
    }

    /// `write_tileset_atlas` produces a PNG with the correct dimensions for a
    /// known number of tiles.
    #[test]
    fn test_write_tileset_atlas_dimensions() {
        let tiles: Vec<Tile> = (0..100)
            .map(|_| Tile {
                colors: [Color { r: 0, g: 0, b: 0 }; 1024],
            })
            .collect();

        let out_dir = std::env::temp_dir().join("tmx_test_atlas");
        fs::create_dir_all(&out_dir).unwrap();
        let path = out_dir.join("test_atlas.png");

        write_tileset_atlas(&tiles, &path).unwrap();

        // 100 tiles at 48 per row → ceil(100/48) = 3 rows → 3 * 32 = 96 px
        let img = image::open(&path).unwrap();
        assert_eq!(img.width(), 48 * 32, "atlas width");
        assert_eq!(img.height(), 3 * 32, "atlas height");

        fs::remove_dir_all(&out_dir).ok();
    }

    /// `write_tileset_atlas` handles an empty tile list gracefully.
    #[test]
    fn test_write_tileset_atlas_empty() {
        let out_dir = std::env::temp_dir().join("tmx_test_empty_atlas");
        fs::create_dir_all(&out_dir).unwrap();
        let path = out_dir.join("empty.png");

        write_tileset_atlas(&[], &path).unwrap();

        let img = image::open(&path).unwrap();
        assert_eq!(img.width(), 1, "empty atlas should be 1×1");
        assert_eq!(img.height(), 1, "empty atlas should be 1×1");

        fs::remove_dir_all(&out_dir).ok();
    }

    /// End-to-end test: export a real map fixture and validate the TMX output.
    #[test]
    fn test_export_tmx_creates_files() {
        let fixture_map = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.map"
        ));
        let fixture_gtl = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.gtl"
        ));
        let fixture_btl = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.btl"
        ));

        if !fixture_map.exists() {
            eprintln!("Skipping TMX test: fixtures not found");
            return;
        }

        // Parse map
        use super::super::read_map_data;
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(fixture_map).unwrap();
        let mut reader = BufReader::new(file);
        let map_data = read_map_data(&mut reader).unwrap();

        // Export to temp dir
        let out_dir = std::env::temp_dir().join("tmx_test_cat1");
        fs::create_dir_all(&out_dir).unwrap();
        export_tmx(&map_data, fixture_gtl, fixture_btl, &out_dir).unwrap();

        // Verify files exist
        assert!(
            out_dir.join("cat1.tmx").exists(),
            "TMX file not created"
        );
        assert!(
            out_dir.join("cat1_ground.png").exists(),
            "Ground tileset not created"
        );
        assert!(
            out_dir.join("cat1_roof.png").exists(),
            "Roof tileset not created"
        );

        // Read back TMX and validate basic structure
        let tmx_content = fs::read_to_string(out_dir.join("cat1.tmx")).unwrap();
        assert!(tmx_content.contains("<?xml"), "Missing XML declaration");
        assert!(tmx_content.contains("<map"), "Missing map element");
        assert!(
            tmx_content.contains("orientation=\"orthogonal\""),
            "Missing orientation"
        );
        assert!(tmx_content.contains("<layer"), "Missing layer");
        assert!(
            tmx_content.contains("name=\"Ground\""),
            "Missing Ground layer"
        );
        assert!(
            tmx_content.contains("name=\"Roofs\""),
            "Missing Roofs layer"
        );
        assert!(
            tmx_content.contains("name=\"Collisions\""),
            "Missing Collisions object group"
        );
        assert!(
            tmx_content.contains("name=\"Events\""),
            "Missing Events object group"
        );
        assert!(
            tmx_content.contains("name=\"TiledObjects\""),
            "Missing TiledObjects object group"
        );

        // Verify tile counts match
        let w = map_data.model.tiled_map_width;
        let h = map_data.model.tiled_map_height;
        assert!(
            tmx_content.contains(&format!("width=\"{}\"", w)),
            "Wrong width"
        );
        assert!(
            tmx_content.contains(&format!("height=\"{}\"", h)),
            "Wrong height"
        );

        // Cleanup
        fs::remove_dir_all(&out_dir).ok();
    }

    /// Validate CSV tile data content by counting expected number of commas
    /// per row and total rows in a layer.
    #[test]
    fn test_export_tmx_csv_tile_count() {
        let fixture_map = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.map"
        ));
        let fixture_gtl = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.gtl"
        ));
        let fixture_btl = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.btl"
        ));

        if !fixture_map.exists() {
            eprintln!("Skipping CSV count test: fixtures not found");
            return;
        }

        use super::super::read_map_data;
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(fixture_map).unwrap();
        let mut reader = BufReader::new(file);
        let map_data = read_map_data(&mut reader).unwrap();

        let out_dir = std::env::temp_dir().join("tmx_test_csv_count");
        fs::create_dir_all(&out_dir).unwrap();
        export_tmx(&map_data, fixture_gtl, fixture_btl, &out_dir).unwrap();

        let content = fs::read_to_string(out_dir.join("cat1.tmx")).unwrap();
        let w = map_data.model.tiled_map_width as usize;
        let h = map_data.model.tiled_map_height as usize;

        // Extract the Ground layer CSV block
        let ground_start = content
            .find(r#"name="Ground""#)
            .expect("Ground layer not found");
        let csv_start = content[ground_start..]
            .find("<data")
            .expect("data tag not found")
            + ground_start;
        let csv_open = content[csv_start..]
            .find("encoding=\"csv\"")
            .expect("csv encoding not found")
            + csv_start;
        let data_start = content[csv_open..]
            .find(">\n")
            .expect("data open not found")
            + csv_open
            + 2;
        let data_end = content[data_start..]
            .find("    </data>")
            .expect("data close not found")
            + data_start;

        let csv_block = &content[data_start..data_end];

        // Count lines (each is a row).  Ignore blank lines.
        let rows: Vec<&str> = csv_block
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        assert_eq!(rows.len(), h, "Ground CSV row count");

        // Each row should have w entries separated by commas (w-1 commas).
        for (i, row) in rows.iter().enumerate() {
            let parts: Vec<&str> = row.split(',').filter(|s| !s.is_empty()).collect();
            assert_eq!(
                parts.len(),
                w,
                "Ground CSV row {} has {} entries, expected {}",
                i,
                parts.len(),
                w
            );
        }

        fs::remove_dir_all(&out_dir).ok();
    }

    /// Verify that the tileset image dimensions in the TMX match the actual
    /// atlas image files.
    #[test]
    fn test_export_tmx_tileset_image_dimensions() {
        let fixture_map = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.map"
        ));
        let fixture_gtl = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.gtl"
        ));
        let fixture_btl = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.btl"
        ));

        if !fixture_map.exists() {
            eprintln!("Skipping dimension test: fixtures not found");
            return;
        }

        use super::super::read_map_data;
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(fixture_map).unwrap();
        let mut reader = BufReader::new(file);
        let map_data = read_map_data(&mut reader).unwrap();

        let out_dir = std::env::temp_dir().join("tmx_test_dims");
        fs::create_dir_all(&out_dir).unwrap();
        export_tmx(&map_data, fixture_gtl, fixture_btl, &out_dir).unwrap();

        // Read actual image dimensions
        let ground_img = image::open(out_dir.join("cat1_ground.png")).unwrap();
        let roof_img = image::open(out_dir.join("cat1_roof.png")).unwrap();

        // Verify against TMX declarations
        let tmx = fs::read_to_string(out_dir.join("cat1.tmx")).unwrap();

        // Extract declared ground image size
        let ground_decl_w = extract_tmx_attr(&tmx, "source=\"cat1_ground.png\"", "width")
            .parse::<u32>()
            .unwrap();
        let ground_decl_h = extract_tmx_attr(&tmx, "source=\"cat1_ground.png\"", "height")
            .parse::<u32>()
            .unwrap();

        assert_eq!(
            ground_img.width(), ground_decl_w,
            "Ground PNG width matches TMX declaration"
        );
        assert_eq!(
            ground_img.height(), ground_decl_h,
            "Ground PNG height matches TMX declaration"
        );

        // Extract declared roof image size
        let roof_decl_w = extract_tmx_attr(&tmx, "source=\"cat1_roof.png\"", "width")
            .parse::<u32>()
            .unwrap();
        let roof_decl_h = extract_tmx_attr(&tmx, "source=\"cat1_roof.png\"", "height")
            .parse::<u32>()
            .unwrap();

        assert_eq!(
            roof_img.width(), roof_decl_w,
            "Roof PNG width matches TMX declaration"
        );
        assert_eq!(
            roof_img.height(), roof_decl_h,
            "Roof PNG height matches TMX declaration"
        );

        fs::remove_dir_all(&out_dir).ok();
    }

    /// Helper: extract an XML attribute value.  Searches for a string that
    /// is *after* `after_token` in the content and returns the value of the
    /// first attribute named `attr_name` found after it.
    fn extract_tmx_attr<'a>(content: &'a str, after_token: &str, attr_name: &str) -> &'a str {
        let pos = content.find(after_token).expect("token not found");
        let rest = &content[pos..];
        // Find ` attr_name="`
        let search = format!(" {}=\"", attr_name);
        let val_start = rest.find(&search).expect("attr not found") + search.len();
        let val_end = rest[val_start..]
            .find('"')
            .expect("attr value not closed");
        &rest[val_start..val_start + val_end]
    }
}
