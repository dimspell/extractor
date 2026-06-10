use std::{fs, env};
use dispel_core::references::save_file::SaveFile;
use dispel_core::Extractor;

fn check(path: &str) {
    let original = fs::read(path).unwrap();
    println!("File: {}, size: {}", path, original.len());
    
    let save = SaveFile::parse(&original).unwrap();
    println!("  Surface monsters: {}", save.surface_monsters.len());
    println!("  NPCs: {}", save.npcs.len());
    println!("  Surface objects: {}", save.surface_objects.len());
    println!("  Dungeon map ID: {}", save.dungeon_map_id);
    println!("  Dungeon monsters: {}", save.dungeon_monsters.len());
    println!("  Dungeon objects: {}", save.dungeon_objects.len());
    println!("  Events: {}", save.events.len());
    println!("  Player: '{}' (class '{}' id={})",
        save.player_name.trim(), save.player_class_name.trim(), save.player_class_id);
    println!("  Stats: str={} dex={} wis={} con={} unk={} hp={}/{} mp={}/{} xp={} lvl={} gold={}",
        save.player_attributes.strength,
        save.player_attributes.dexterity,
        save.player_attributes.wisdom,
        save.player_attributes.constitution,
        save.player_attributes.unknown_stat,
        save.player_attributes.hp_current,
        save.player_attributes.hp_maximum,
        save.player_attributes.mp_current,
        save.player_attributes.mp_maximum,
        save.player_attributes.xp_current,
        save.player_attributes.level,
        save.player_attributes.gold);
    
    let mut output = Vec::new();
    SaveFile::to_writer(&[save], &mut output).unwrap();
    
    if original.len() == output.len() {
        let mismatches: Vec<usize> = (0..original.len()).filter(|&i| original[i] != output[i]).collect();
        if mismatches.is_empty() {
            println!("  ✓ ALL {} BYTES MATCH!", original.len());
        } else {
            println!("  ✗ {} mismatches (showing first 10):", mismatches.len());
            for &i in mismatches.iter().take(10) {
                println!("    byte {}: orig={:02x} out={:02x} (expected {})", i, original[i], output[i],
                    if output[i] == original[i] { "MATCH" } else { "DIFF" });
            }
        }
    } else {
        println!("  ✗ Size mismatch: original={} output={}", original.len(), output.len());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        for path in &args[1..] {
            check(path);
            println!();
        }
    } else {
        check("nuno-0.sav");
        check("0.sav");
        check("2.sav");
    }
}
