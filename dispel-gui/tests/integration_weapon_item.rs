//! Integration tests for WeaponItem using core library
//!
//! Run it with:
//!     cargo test -p dispel-gui --test integration_weapon_item -- --nocapture

use std::path::PathBuf;

fn weapon_fixture_path() -> PathBuf {
    PathBuf::from("../fixtures/Dispel/CharacterInGame/WeaponItem.db")
}

#[test]
fn test_weapon_fixture_exists() {
    let path = weapon_fixture_path();
    if !path.exists() {
        eprintln!("SKIP: fixture not found: {}", path.display());
        return;
    }
    println!("Weapon fixture exists: {:?}", path);
}

#[test]
fn test_weapon_item_fixture_loadable() {
    use dispel_core::references::weapons_db::WeaponItem;
    use dispel_core::Extractor;

    let fixture_path = weapon_fixture_path();
    if !fixture_path.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture_path.display());
        return;
    }

    let result = WeaponItem::read_file(&fixture_path);

    match result {
        Ok(items) => {
            println!(
                "Successfully loaded {} weapon items from fixture",
                items.len()
            );
            assert!(!items.is_empty(), "Weapon fixture should have items");

            if let Some(first) = items.first() {
                println!(
                    "First weapon: id={}, name='{}', attack={}",
                    first.id, first.name, first.attack
                );
            }
        }
        Err(e) => {
            panic!("Failed to load weapon fixture: {}", e);
        }
    }
}

#[test]
fn test_weapon_item_save_roundtrip() {
    use dispel_core::references::weapons_db::WeaponItem;
    use dispel_core::Extractor;

    let fixture_path = weapon_fixture_path();
    if !fixture_path.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture_path.display());
        return;
    }

    let items = WeaponItem::read_file(&fixture_path).expect("Failed to load weapon items");

    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    WeaponItem::save_file(&items, temp_file.path()).expect("Failed to save weapon items");

    let saved_items = WeaponItem::read_file(temp_file.path()).expect("Failed to reload");

    assert_eq!(items.len(), saved_items.len(), "Item count should match");

    for (original, saved) in items.iter().zip(saved_items.iter()) {
        assert_eq!(original.name, saved.name, "Weapon name should be preserved");
        assert_eq!(original.attack, saved.attack, "Attack should be preserved");
        assert_eq!(
            original.defense, saved.defense,
            "Defense should be preserved"
        );
        assert_eq!(
            original.base_price, saved.base_price,
            "Price should be preserved"
        );
    }

    println!("Weapon item save round-trip: PASS (data preserved)");
}

#[test]
fn test_weapon_item_edit_workflow() {
    use dispel_core::references::weapons_db::WeaponItem;
    use dispel_core::Extractor;

    let fixture_path = weapon_fixture_path();
    if !fixture_path.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture_path.display());
        return;
    }

    let mut items = WeaponItem::read_file(&fixture_path).expect("Failed to load");
    assert!(!items.is_empty(), "Should have items");

    let original_attack = items[0].attack;
    let original_name = items[0].name.clone();
    items[0].attack = original_attack + 1;

    println!(
        "Edited weapon[0]: attack {} -> {}",
        original_attack, items[0].attack
    );

    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    WeaponItem::save_file(&items, temp_file.path()).expect("Failed to save");

    let reloaded = WeaponItem::read_file(temp_file.path()).expect("Failed to reload");

    assert_eq!(
        reloaded[0].attack,
        original_attack + 1,
        "Reloaded weapon should have edited attack value"
    );

    items[0].attack = original_attack;
    WeaponItem::save_file(&items, temp_file.path()).expect("Failed to restore");

    let restored = WeaponItem::read_file(temp_file.path()).expect("Failed to reload restored");
    assert_eq!(
        restored[0].attack, original_attack,
        "Attack should be restored"
    );
    assert_eq!(restored[0].name, original_name, "Name should be preserved");

    println!("Weapon edit workflow test: PASS");
}
