# Wave.ini Documentation

## File Format: Audio/Sound Effect References

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted audio content, sound files, or proprietary assets. All references to sound effects are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Text file that maps sound IDs to SNF audio files with playback behavior flags for the game's audio system.

### File Structure

**Location**: `Wave.ini`
**Encoding**: EUC-KR (Korean character encoding)
**Format**: CSV (Comma-Separated Values) with comments
**Total Entries**: 290 sound effects (IDs 0-290)

### Format Specification

```
; Comment line
id,snf_filename,unknown_flag
0,HitM.snf,5
1,HitF.snf,5
2,DieM.snf,5
...
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| id | i32 | Unique sound/audio identifier (0-290) |
| snf_filename | string | SNF audio filename (e.g., "HitM.snf") |
| unknown_flag | string | Playback behavior flag (typically "5") |

### Sound Organization

The file organizes sound effects into functional categories using comment headers. Typical categories include:

**Gameplay Sounds:**
- Character action sounds
- Combat effects
- Movement and interaction sounds

**Environmental Sounds:**
- Ambient noise and background effects
- Nature and weather sounds
- Location-specific audio

**Interface Sounds:**
- User interface feedback
- System notifications
- Game state changes

**Special Effects:**
- Magical and spell effects
- Unique ability sounds
- Event-triggered audio

**Party Sounds:**
- Character voice variations for party members
- Multiple variations per character type
- Consistent naming pattern: Party[X][1-5].snf
- 8 character types × 5 voice variations each

Each category contains multiple sound variations following consistent naming patterns for easy identification and management.

### Field Value Analysis

**unknown_flag:**
- Most entries use "5" (285 out of 290 entries)
- Some entries use "1": 249, 256-261, 286-290
- Likely represents playback priority or channel assignment
- "5" = standard sound effect channel
- "1" = higher priority or special channel

**snf_filename:**
- All entries have valid SNF filenames (no "null" values)
- Follows consistent naming patterns within categories
- Files are referenced from game's audio system

### Example Format

```ini
; Comment line describing sound category
id,sound_file.snf,flag
1,sound001.snf,5
2,sound002.snf,5
3,sound003.snf,5
```

The file follows a consistent CSV format where each line represents a sound effect mapping with:
- **id**: Unique numerical identifier
- **sound_file.snf**: Audio filename in proprietary format
- **flag**: Playback behavior indicator

### Technical Details

**Encoding**: EUC-KR (Extended Unix Code Korea)
- Supports Korean characters in comments
- Requires proper encoding handling for reading/writing

**File Processing**:
- Comments (lines starting with ";") are ignored
- Empty lines are skipped
- CSV format with comma delimiter
- All fields are required (no null values in this file)

**Database Integration**:
- Processed by `WaveIni` struct in the codebase
- Stored in database with id, snf_filename, and unknown_flag fields
- Referenced by dialogue_text.rs for dialogue sound effects

### Usage in Game

1. **Audio System Initialization**: Game loads sound mappings from Wave.ini
2. **Sound Playback**: Uses IDs to play appropriate SNF files
3. **Dialogue Integration**: wave_ini_entry_id in dialogue_text.rs links to these sound IDs
4. **Dynamic Audio**: unknown_flag controls playback behavior (looping, priority, etc.)
5. **Resource Management**: Organizes 290+ sound effects for efficient loading

### Sound File Format

**SNF Files**: Proprietary audio format used by the game
- Likely contains compressed audio data
- Referenced by filename in Wave.ini
- Stored in game's audio resource directory

### Characteristics

- **Entry Count**: Multiple sound effect mappings
- **Organization**: Logical categorization by function
- **Flag System**: Playback behavior indicators
- **ID System**: Sequential numerical identifiers
- **Naming**: Consistent patterns within categories

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any SNF audio files or sound content
- Uses **generic descriptions** of sound categories
- Focuses on **technical organization**, not creative content
- Maintains **nominal fair use** for trademark references

### Notes

- File uses Windows-style line endings (\r\n)
- All sound IDs are used (no gaps in sequence)
- Comments are in Korean (EUC-KR encoding)
- unknown_flag values suggest priority/channel system
- File is complete and well-organized by functional categories
- **No copyrighted audio content** is reproduced or distributed
