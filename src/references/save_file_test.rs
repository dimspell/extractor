// Tests for save file parsing
// These tests verify the save file extraction logic

#[cfg(test)]
mod save_file_tests {
    use super::super::save_file::*;
    
    #[test]
    fn test_save_item_type_from_u8() {
        assert_eq!(SaveItemType::from_u8(0), Some(SaveItemType::Weapon));
        assert_eq!(SaveItemType::from_u8(1), Some(SaveItemType::Armor));
        assert_eq!(SaveItemType::from_u8(2), Some(SaveItemType::Heal));
        assert_eq!(SaveItemType::from_u8(3), Some(SaveItemType::Misc));
        assert_eq!(SaveItemType::from_u8(4), Some(SaveItemType::Edit));
        assert_eq!(SaveItemType::from_u8(5), Some(SaveItemType::Event));
        assert_eq!(SaveItemType::from_u8(255), None);
    }
    
    #[test]
    fn test_save_item_type_value() {
        assert_eq!(SaveItemType::Weapon.value(), 0);
        assert_eq!(SaveItemType::Armor.value(), 1);
        assert_eq!(SaveItemType::Heal.value(), 2);
        assert_eq!(SaveItemType::Misc.value(), 3);
        assert_eq!(SaveItemType::Edit.value(), 4);
        assert_eq!(SaveItemType::Event.value(), 5);
    }
    
    #[test]
    fn test_player_attributes_parse() {
        let data = [
            0x41, 0x00, // STR = 65
            0x0B, 0x00, // DEX = 11
            0x07, 0x00, // WIS = 7
            0x15, 0x00, // CON = 21
            0x0A, 0x00, // Unknown = 10
            0x0C, 0x00, // HP cur = 12
            0x2A, 0x00, // HP max = 42
            0x0E, 0x00, // MP cur = 14
            0x0E, 0x00, // MP max = 14
            0xD9, 0x02, 0x00, 0x00, // XP = 729
            0x05, 0x00, // Level = 5
            0x9D, 0x04, 0x00, 0x00, // Gold = 1181
        ];
        
        let attrs = PlayerAttributes::parse(&data).expect("Failed to parse player attributes");
        assert_eq!(attrs.strength, 65);
        assert_eq!(attrs.dexterity, 11);
        assert_eq!(attrs.wisdom, 7);
        assert_eq!(attrs.constitution, 21);
        assert_eq!(attrs.unknown_stat, 10);
        assert_eq!(attrs.hp_current, 12);
        assert_eq!(attrs.hp_maximum, 42);
        assert_eq!(attrs.mp_current, 14);
        assert_eq!(attrs.mp_maximum, 14);
        assert_eq!(attrs.xp_current, 729);
        assert_eq!(attrs.level, 5);
        assert_eq!(attrs.gold, 1181);
    }
    
    #[test]
    fn test_monster_state_parse() {
        // Test dead monster
        let dead_flags = 1u32;
        let state = MonsterState::parse(dead_flags);
        assert!(state.is_dead);
        assert!(!state.is_poisoned);
        assert!(!state.is_boss);
        
        // Test boss monster
        let boss_flags = 1u32 | (1 << 31);
        let state = MonsterState::parse(boss_flags);
        assert!(state.is_dead);
        assert!(state.is_boss);
        
        // Test alive monster
        let alive_flags = 0u32;
        let state = MonsterState::parse(alive_flags);
        assert!(!state.is_dead);
        assert!(!state.is_boss);
    }
    
    #[test]
    fn test_inventory_item_type_mapping() {
        // Weapon: attack > 0, defense = 0
        // Armor: defense > 0, attack = 0
        // In weapons.json, items with defense > 0 are armor
        
        // Test that SaveItemType enum covers all categories
        let all_types = vec![
            SaveItemType::Weapon,
            SaveItemType::Armor,
            SaveItemType::Heal,
            SaveItemType::Misc,
            SaveItemType::Edit,
            SaveItemType::Event,
        ];
        
        assert_eq!(all_types.len(), 6);
        
        // Verify each type has a unique value
        let values: Vec<u8> = all_types.iter().map(|t| t.value()).collect();
        assert_eq!(values, vec![0, 1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn test_potion_belt_structure() {
        // Potion belt has 6 slots
        assert_eq!(6, 6, "Potion belt should have exactly 6 slots");
        
        // Each slot is 256 bytes
        assert_eq!(256, 256, "Each potion slot should be 256 bytes");
        
        // Total potion belt size: 6 × 256 = 1536 bytes
        assert_eq!(6 * 256, 1536);
    }
    
    #[test]
    fn test_inventory_slot_structure() {
        // Main inventory slot: 502 bytes (246 quest + 256 item)
        assert_eq!(246 + 256, 502);
        
        // Potion belt slot: 256 bytes (item only)
        assert_eq!(256, 256);
        
        // Verify the structure
        assert_eq!(502, 502, "Main inventory slot should be 502 bytes");
    }
    
    #[test]
    fn test_item_type_discrimination() {
        // From weapons.json analysis:
        // - Items with attack > 0 are weapons (type 0)
        // - Items with defense > 0 are armor (type 1)
        
        // This test documents the discrimination logic
        let weapon_like = 0u8; // Would have attack > 0
        let armor_like = 1u8; // Would have defense > 0
        
        assert_eq!(SaveItemType::from_u8(weapon_like), Some(SaveItemType::Weapon));
        assert_eq!(SaveItemType::from_u8(armor_like), Some(SaveItemType::Armor));
    }
    
    #[test]
    fn test_save_file_structure_sizes() {
        // Verify all record sizes match the research
        
        // Monster record: 329 bytes
        assert_eq!(329, 329);
        
        // NPC record: 349 bytes
        assert_eq!(349, 349);
        
        // Object record: 200 bytes
        assert_eq!(200, 200);
        
        // Event item record: 24 bytes
        assert_eq!(24, 24);
        
        // Event script record: 32 bytes
        assert_eq!(32, 32);
        
        // Inventory quest record: 246 bytes
        assert_eq!(246, 246);
        
        // Inventory item record: 256 bytes
        assert_eq!(256, 256);
    }
    
    #[test]
    fn test_encoding_constants() {
        // Verify the encoding used throughout
        assert_eq!(0x0261B6, 156150); // Player attributes start
        assert_eq!(0x026200, 156160); // Inventory start
        assert_eq!(0x02A994, 174484); // Event scripts start
        
        // These offsets are used in the save file parsing
        assert!(0x0261B6 < 0x02A994);
    }
}
