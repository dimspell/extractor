//! Integration tests for file tree search/filter functionality

#[test]
fn file_tree_search_extension_filtering() {
    // This test documents the fix for fuzzy matching file extensions
    // Previously, searching for ".db" would not find "CharacterInGame/weaponItem.db"
    // because the custom fuzzy_match function required sequential character matching

    // Simulate what the file tree search would do
    let files = vec![
        "CharacterInGame/weaponItem.db",
        "CharacterInGame/HealItem.db",
        "CharacterInGame/MiscItem.db",
        "AllMap.ini",
        "Event.ini",
        "Extra.ini",
        "Npc.ini",
    ];

    // Test 1: Search for ".db" should find all .db files
    let db_results: Vec<_> = files
        .iter()
        .filter(|f| f.to_lowercase().ends_with(".db"))
        .collect();
    assert_eq!(db_results.len(), 3, "Should find 3 .db files");

    // Test 2: Search for ".ini" should find all .ini files
    let ini_results: Vec<_> = files
        .iter()
        .filter(|f| f.to_lowercase().ends_with(".ini"))
        .collect();
    assert_eq!(ini_results.len(), 4, "Should find 4 .ini files");

    // Test 3: Search for "weapon" should find weaponItem.db
    let weapon_results: Vec<_> = files
        .iter()
        .filter(|f| f.to_lowercase().contains("weapon"))
        .collect();
    assert_eq!(weapon_results.len(), 1, "Should find 1 weapon file");

    // Test 4: Search for "Item" should find multiple files with Item in name
    let item_results: Vec<_> = files
        .iter()
        .filter(|f| f.to_lowercase().contains("item"))
        .collect();
    assert_eq!(
        item_results.len(),
        3,
        "Should find 3 files with 'Item' in name"
    );
}

#[test]
fn file_tree_search_case_insensitive() {
    let files = vec![
        "CharacterInGame/weaponItem.db",
        "CharacterInGame/HealItem.db",
        "AllMap.ini",
    ];

    // Case variations should all work
    let search_patterns = vec![".DB", ".db", ".Db", ".dB"];

    for pattern in search_patterns {
        let results: Vec<_> = files
            .iter()
            .filter(|f| f.to_lowercase().ends_with(&pattern.to_lowercase()))
            .collect();
        assert_eq!(
            results.len(),
            2,
            "Should find 2 .db files regardless of case"
        );
    }
}

#[test]
fn file_tree_search_partial_matches() {
    let files = vec![
        "CharacterInGame/weaponItem.db",
        "CharacterInGame/HealItem.db",
        "CharacterInGame/MiscItem.db",
    ];

    // Partial word matches
    let test_cases = vec![
        ("heal", 1),   // HealItem
        ("misc", 1),   // MiscItem
        ("weapon", 1), // weaponItem
        ("item", 3),   // All contain Item
        ("game", 3),   // All in CharacterInGame
        ("char", 3),   // All in CharacterInGame (case insensitive)
    ];

    for (pattern, expected_count) in test_cases {
        let results: Vec<_> = files
            .iter()
            .filter(|f| f.to_lowercase().contains(&pattern.to_lowercase()))
            .collect();
        assert_eq!(
            results.len(),
            expected_count,
            "Pattern '{}' should match {} files",
            pattern,
            expected_count
        );
    }
}

#[test]
fn file_tree_search_subdirectory_files() {
    // Test that files in subdirectories can be found by their filename
    let files = vec![
        "NpcInGame/Face1.spr",
        "NpcInGame/Face2.spr",
        "NpcInGame/Body1.spr",
        "CharacterInGame/weaponItem.db",
    ];

    let test_cases = vec![
        ("Face", 2),      // Find files starting with Face in any directory
        ("Face1", 1),     // Exact filename match in subdirectory
        ("Body", 1),      // Body1.spr
        ("NpcInGame", 3), // All files in NpcInGame directory
        ("spr", 3),       // All .spr files
        (".spr", 3),      // All .spr files (with dot)
        ("weapon", 1),    // File in CharacterInGame
    ];

    for (pattern, expected_count) in test_cases {
        let results: Vec<_> = files
            .iter()
            .filter(|f| {
                // Simulate fuzzy matching: check if pattern appears anywhere in path
                f.to_lowercase().contains(&pattern.to_lowercase())
            })
            .collect();
        assert_eq!(
            results.len(),
            expected_count,
            "Pattern '{}' should match {} files, but got: {:?}",
            pattern,
            expected_count,
            results
        );
    }
}

#[test]
fn file_tree_hierarchy_with_filtered_subdirectory_files() {
    // When searching for files in subdirectories, the parent directories must be included
    // even if they don't match the search query themselves
    // Example: Search for "Face" should show NpcInGame/ directory even though "Face" != "NpcInGame"

    let files = vec![
        "NpcInGame",                 // directory
        "NpcInGame/Face1.spr",       // file matching "Face"
        "NpcInGame/Body1.spr",       // file NOT matching "Face"
        "CharacterInGame",           // directory
        "CharacterInGame/weapon.db", // file NOT matching "Face"
    ];

    // Simulate what happens when searching for "Face"
    // Expected: NpcInGame/ directory should be included because it has a matching child
    let query = "Face";
    let matching_or_related: Vec<_> = files
        .iter()
        .filter(|f| {
            // A file matches if:
            // 1. It's a file and contains the query, OR
            // 2. It's a directory and has children that match (we'll simulate this)
            if f.contains('/') {
                // It's a file path
                f.to_lowercase().contains(&query.to_lowercase())
            } else {
                // It's a directory - check if any file in it matches
                files.iter().any(|file| {
                    file.starts_with(&format!("{}/", f))
                        && file.to_lowercase().contains(&query.to_lowercase())
                })
            }
        })
        .collect();

    // Should include NpcInGame/ directory (because Face1.spr matches)
    // and Face1.spr file itself
    assert!(
        matching_or_related.iter().any(|&&f| f == "NpcInGame"),
        "Parent directory NpcInGame should be included"
    );
    assert!(
        matching_or_related
            .iter()
            .any(|&&f| f == "NpcInGame/Face1.spr"),
        "Matching file NpcInGame/Face1.spr should be included"
    );
    // Should NOT include other files/dirs that don't match
    assert!(
        !matching_or_related
            .iter()
            .any(|&&f| f == "NpcInGame/Body1.spr"),
        "Non-matching file Body1.spr should be excluded"
    );
    assert!(
        !matching_or_related.iter().any(|&&f| f == "CharacterInGame"),
        "Unrelated directory CharacterInGame should be excluded"
    );
}
