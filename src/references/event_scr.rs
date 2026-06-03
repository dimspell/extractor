use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek, Write};
use std::path::Path;

use crate::references::extractor::Extractor;
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite;
use serde::{Deserialize, Serialize};

/// Event Script Files (Event*.scr)
///
/// # ASCII Structure
///
/// ```text
/// +--------------------------------------+
/// | Event Script Files                  |
/// +--------------------------------------+
/// | Encoding: EUC-KR                     |
/// | Format: INI-style with sections      |
/// | Record Size: Variable (text)        |
/// +--------------------------------------+
/// | ; Comment line                       |
/// | [VAR]                                |
/// | varname=value                        |
/// |                                     |
/// | [MAP]                                |
/// | map_id                               |
/// |                                     |
/// | [CHR]                                |
/// | char_id                              |
/// |                                     |
/// | [NPC]                                |
/// | npc_id                               |
/// |                                     |
/// | [SPR]                                |
/// | Pope(PopeBlessing.spr)              |
/// |                                     |
/// | [WAV]                                |
/// | sound_id                             |
/// |                                     |
/// | [ACT]                                |
/// | function_call(param1, param2)        |
/// | if(condition)                        |
/// | {                                    |
/// |   action1()                          |
/// | }                                    |
/// | else                                 |
/// | {                                    |
/// |   action2()                          |
/// | }                                    |
/// +--------------------------------------+
/// ```
///
/// # Section Definitions
///
/// - `[VAR]`: Variables used in the script
/// - `[MAP]`: Map ID to load/unload
/// - `[CHR]`: Character reference
/// - `[NPC]`: NPC reference
/// - `[SPR]`: Sprites to load, providing further alias in the script
/// - `[WAV]`: Sounds
/// - `[ACT]`: Actions/script logic
///
/// # Special Values
///
/// - Lines starting with `;` are comments
/// - Empty lines ignored
/// - Variables in format: `name=value`
/// - Actions are function calls with parameters
/// - Control flow: `if()`, `else`, `repeat`, etc.
///
/// # File Purpose
///
/// Defines game events, scripts, and interactive sequences.
/// Used for quests, cutscenes, NPC interactions, and game
/// state changes. Events are triggered by map interactions
/// or quest progression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventScript {
    /// Event script identifier (from filename).
    pub id: i32,
    /// Header comments from the file.
    pub header_comments: Vec<String>,
    /// Whether the original file's last line had a trailing CRLF.
    /// Set during `read_file` by inspecting raw bytes; defaults to `true`
    /// for files parsed from bytes (e.g. Cursor) where detection is unavailable.
    pub trailing_newline: bool,
    /// Raw ACT section lines as they appeared in the original file (before
    /// trimming).  When present, `to_writer` uses these verbatim instead of
    /// reconstructing from `actions`, guaranteeing lossless round-trip even
    /// for minor formatting differences (trailing whitespace, etc.).
    ///
    /// Set only during `read_file`.  The GUI clears this after modifying
    /// `actions`, so the writer falls back to the structured representation.
    #[serde(skip)]
    pub act_raw_lines: Option<Vec<String>>,
    /// Variables defined in the [VAR] section.
    pub variables: Vec<Variable>,
    /// Map content from the [MAP] section (can be "map(null)" or other values).
    pub map_content: Vec<String>,
    /// Character content from the [CHR] section.
    pub chr_content: Vec<String>,
    /// NPC content from the [NPC] section.
    pub npc_content: Vec<String>,
    /// Sprite definitions from the [SPR] section (e.g., "Pope(PopeBlessing.spr)").
    pub spr_content: Vec<SpriteDefinition>,
    /// Sound/WAV content from the [WAV] section.
    pub wav_content: Vec<String>,
    /// Actions/script logic from the [ACT] section.
    pub actions: Vec<ActionFunction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Variable {
    /// Variable name.
    pub name: String,
    /// Variable value.
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteDefinition {
    /// Sprite alias/identifier (e.g., "Pope").
    pub sprite_alias: String,
    /// Sprite filename (e.g., "PopeBlessing.spr").
    pub sprite_file: String,
}

impl SpriteDefinition {
    /// Parse a sprite definition line like "Pope(PopeBlessing.spr)"
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();
        // Check if the line has the format: alias(filename)
        if let Some(start) = trimmed.find('(') {
            if let Some(end) = trimmed.rfind(')') {
                let alias = trimmed[..start].trim();
                let file = trimmed[start + 1..end].trim();
                return SpriteDefinition {
                    sprite_alias: alias.to_string(),
                    sprite_file: file.to_string(),
                };
            }
        }
        // Handle the simple format without parentheses: alias
        SpriteDefinition {
            sprite_alias: trimmed.to_string(),
            sprite_file: String::new(),
        }
    }
}

impl Default for EventScript {
    fn default() -> Self {
        Self {
            id: 0,
            header_comments: Vec::new(),
            trailing_newline: true,
            act_raw_lines: None,
            variables: Vec::new(),
            map_content: Vec::new(),
            chr_content: Vec::new(),
            npc_content: Vec::new(),
            spr_content: Vec::new(),
            wav_content: Vec::new(),
            actions: Vec::new(),
        }
    }
}

impl std::fmt::Display for SpriteDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sprite_file.is_empty() {
            write!(f, "{}", self.sprite_alias)
        } else {
            write!(f, "{}({})", self.sprite_alias, self.sprite_file)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionFunction {
    /// Object prefix (e.g., "Pope~" for Pope~setmappos)
    pub prefix: Option<String>,
    /// Function name (e.g., "addquest", "setmappos")
    pub function_name: String,
    /// Function parameters as individual strings
    pub parameters: Vec<String>,
    /// Raw line content for non-function lines (control flow, braces, etc.)
    pub raw_content: Option<String>,
}

impl ActionFunction {
    /// Parse a line into an ActionFunction
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();

        // Handle control flow and braces (if, else, {, }, etc.)
        if trimmed == "{"
            || trimmed == "}"
            || trimmed.starts_with("if(")
            || trimmed.starts_with("else")
            || trimmed.starts_with("return(")
        {
            return ActionFunction {
                prefix: None,
                function_name: String::new(),
                parameters: Vec::new(),
                raw_content: Some(trimmed.to_string()),
            };
        }

        // Handle object prefix (e.g., "King~setmappos")
        let (prefix, rest) = if let Some(tilde_pos) = trimmed.find('~') {
            let prefix_part = trimmed[..tilde_pos].to_string();
            let rest_part = trimmed[tilde_pos + 1..].to_string();
            (Some(prefix_part), rest_part)
        } else {
            (None, trimmed.to_string())
        };

        // Parse function name and parameters
        if let Some(open_paren) = rest.find('(') {
            let function_name = rest[..open_paren].to_string();
            let params_str = &rest[open_paren + 1..rest.len() - 1]; // Remove parentheses

            // Split parameters by comma and trim whitespace
            let parameters: Vec<String> = params_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            ActionFunction {
                prefix,
                function_name,
                parameters,
                raw_content: None,
            }
        } else {
            // Simple function call without parameters
            ActionFunction {
                prefix,
                function_name: rest,
                parameters: Vec::new(),
                raw_content: None,
            }
        }
    }
}

impl std::fmt::Display for ActionFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(raw) = &self.raw_content {
            return write!(f, "{raw}");
        }

        let mut result = String::new();

        // Add prefix if present
        if let Some(prefix) = &self.prefix {
            result.push_str(prefix);
            result.push('~');
        }

        // Add function name
        result.push_str(&self.function_name);

        // Add parameters
        if !self.parameters.is_empty() {
            result.push('(');
            result.push_str(&self.parameters.join(","));
            result.push(')');
        }

        write!(f, "{result}")
    }
}

impl Extractor for EventScript {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(reader.by_ref());
        let buf_reader = BufReader::new(decoded);

        let mut scripts = Vec::new();
        let mut current_script = EventScript::default();
        let mut current_section = String::new();
        let mut in_header = true;

        for line in buf_reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            // Handle header comments (before the first section)
            if in_header {
                if trimmed.starts_with(';') {
                    current_script.header_comments.push(trimmed.to_string());
                    continue;
                } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_header = false;
                    current_section = trimmed[1..trimmed.len() - 1].to_string();
                    continue;
                } else if !trimmed.is_empty() {
                    in_header = false;
                }
            }

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Skip comments in non-ACT sections (in ACT they're inline
            // commented-out function calls that must be preserved).
            if trimmed.starts_with(';') && current_section != "ACT" {
                continue;
            }

            // Check for section headers
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                continue;
            }

            // Parse content based on the current section
            match current_section.as_str() {
                "VAR" => {
                    if let Some(eq_pos) = trimmed.find('=') {
                        let name = trimmed[..eq_pos].trim().to_string();
                        let value = trimmed[eq_pos + 1..].trim().to_string();
                        current_script.variables.push(Variable { name, value });
                    }
                }
                "MAP" if !trimmed.is_empty() => {
                    current_script.map_content.push(trimmed.to_string());
                }
                "CHR" if !trimmed.is_empty() => {
                    current_script.chr_content.push(trimmed.to_string());
                }
                "NPC" if !trimmed.is_empty() => {
                    current_script.npc_content.push(trimmed.to_string());
                }
                "SPR" if !trimmed.is_empty() => {
                    current_script
                        .spr_content
                        .push(SpriteDefinition::parse(trimmed));
                }
                "WAV" if !trimmed.is_empty() => {
                    current_script.wav_content.push(trimmed.to_string());
                }
                "ACT" if !trimmed.is_empty() => {
                    current_script.actions.push(ActionFunction::parse(trimmed));
                }
                _ => {}
            }
        }

        scripts.push(current_script);
        Ok(scripts)
    }

    fn read_file(path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();

        // Read raw bytes first so we can detect trailing newlines
        // and extract raw ACT lines for lossless round-trip.
        let mut raw = Vec::with_capacity(len as usize);
        let mut reader = std::io::BufReader::new(file);
        reader.read_to_end(&mut raw)?;
        let has_trailing_newline = raw.last() == Some(&b'\n');

        // Extract raw ACT section lines (before decoding, so formatting
        // like trailing whitespace is preserved byte-perfectly).
        let raw_act_lines = extract_act_section(&raw);

        let mut cursor = Cursor::new(&raw);
        let mut scripts = Self::parse(&mut cursor, len)?;
        if let Some(script) = scripts.first_mut() {
            script.trailing_newline = has_trailing_newline;
            script.act_raw_lines = raw_act_lines.map(|lines| {
                // Decode each raw line from EUC-KR
                lines
                    .iter()
                    .map(|l| {
                        let (cow, _, _) = EUC_KR.decode(l);
                        cow.to_string()
                    })
                    .collect()
            });
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let lower = stem.to_lowercase();
                if lower.starts_with("event") {
                    script.id = stem[5..].parse::<i32>().unwrap_or(0);
                }
            }
        }
        Ok(scripts)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for script in records {
            // Write header comments
            for comment in &script.header_comments {
                let line = format!("{}\r\n", comment);
                let (cow, _, _) = EUC_KR.encode(&line);
                writer.write_all(&cow)?;
            }
            if !script.header_comments.is_empty() {
                writer.write_all(b"\r\n")?;
            }

            // ── Section writer helper ──────────────────────────────────
            // The original files follow this pattern:
            //   Empty section: [SECTION]\r\n\r\n (blank line inside)
            //   Filled section: [SECTION]\r\ncontent\r\n\r\n (blank line after content)
            //   Last section (filled): [SECTION]\r\ncontent (no trailing blank line)

            // Write the VAR section
            write_euc_kr_line(writer, "[VAR]")?;
            if !script.variables.is_empty() {
                for var in &script.variables {
                    let line = format!("{}={}", var.name, var.value);
                    write_euc_kr_line(writer, &line)?;
                }
                writer.write_all(b"\r\n")?; // trailing blank line
            } else {
                writer.write_all(b"\r\n")?; // blank line inside empty section
            }

            // Write MAP section
            write_euc_kr_line(writer, "[MAP]")?;
            if !script.map_content.is_empty() {
                for line in &script.map_content {
                    write_euc_kr_line(writer, line)?;
                }
                writer.write_all(b"\r\n")?;
            } else {
                writer.write_all(b"\r\n")?;
            }

            // Write CHR section
            write_euc_kr_line(writer, "[CHR]")?;
            if !script.chr_content.is_empty() {
                for line in &script.chr_content {
                    write_euc_kr_line(writer, line)?;
                }
                writer.write_all(b"\r\n")?;
            } else {
                writer.write_all(b"\r\n")?;
            }

            // Write NPC section
            write_euc_kr_line(writer, "[NPC]")?;
            if !script.npc_content.is_empty() {
                for line in &script.npc_content {
                    write_euc_kr_line(writer, line)?;
                }
                writer.write_all(b"\r\n")?;
            } else {
                writer.write_all(b"\r\n")?;
            }

            // Write the SPR section
            write_euc_kr_line(writer, "[SPR]")?;
            if !script.spr_content.is_empty() {
                for sprite in &script.spr_content {
                    write_euc_kr_line(writer, &sprite.to_string())?;
                }
                writer.write_all(b"\r\n")?;
            } else {
                writer.write_all(b"\r\n")?;
            }

            // Write WAV section
            write_euc_kr_line(writer, "[WAV]")?;
            if !script.wav_content.is_empty() {
                for line in &script.wav_content {
                    write_euc_kr_line(writer, line)?;
                }
                writer.write_all(b"\r\n")?;
            } else {
                writer.write_all(b"\r\n")?;
            }

            // Write the ACT section (last section — no trailing blank line)
            write_euc_kr_line(writer, "[ACT]")?;
            // Use raw lines when available (lossless round-trip), otherwise
            // reconstruct from the structured `actions` vec.
            if let Some(ref raw_lines) = script.act_raw_lines {
                let last_idx = raw_lines.len().saturating_sub(1);
                for (i, raw_line) in raw_lines.iter().enumerate() {
                    let (cow, _, _) = EUC_KR.encode(raw_line);
                    writer.write_all(&cow)?;
                    if i != last_idx || script.trailing_newline {
                        writer.write_all(b"\r\n")?;
                    }
                }
            } else {
                let last_idx = script.actions.len().saturating_sub(1);
                for (i, action) in script.actions.iter().enumerate() {
                    let encoded = format!("{}", action);
                    let (cow, _, _) = EUC_KR.encode(&encoded);
                    writer.write_all(&cow)?;
                    // Write CRLF after every action line; the last line's CRLF
                    // is only written if the original file had one.
                    if i != last_idx || script.trailing_newline {
                        writer.write_all(b"\r\n")?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Write a line of text to the writer, encoded as EUC-KR, followed by CRLF.
fn write_euc_kr_line<W: Write>(writer: &mut W, text: &str) -> std::io::Result<()> {
    let (cow, _, _) = EUC_KR.encode(text);
    writer.write_all(&cow)?;
    writer.write_all(b"\r\n")
}

/// Extract the raw bytes of each line in the `[ACT]` section from a
/// raw (undecoded) file.  These are stored verbatim for lossless
/// round-trip, preserving any trailing whitespace the original had.
fn extract_act_section(raw: &[u8]) -> Option<Vec<&[u8]>> {
    // Normalise line endings: split on \n, strip trailing \r
    let text = if raw.contains(&b'\r') {
        // CRLF: split on \n, each line ends with \r before the \n
        raw.split(|&b| b == b'\n')
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
            .collect::<Vec<_>>()
    } else {
        raw.split(|&b| b == b'\n').collect::<Vec<_>>()
    };

    // Find [ACT] section header (case-insensitive)
    let act_start = text.iter().position(|line| {
        let trimmed = line.to_vec();
        let trimmed = std::str::from_utf8(&trimmed)
            .ok()
            .map(|s| s.trim())
            .unwrap_or("");
        trimmed.eq_ignore_ascii_case("[act]")
    })?;

    // Collect lines after [ACT] until the next section or end of file.
    let mut act_lines: Vec<&[u8]> = Vec::new();
    for line in text.iter().skip(act_start + 1) {
        let trimmed = line.to_vec();
        let trimmed = std::str::from_utf8(&trimmed)
            .ok()
            .map(|s| s.trim())
            .unwrap_or("");
        // Stop at the next section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            break;
        }
        act_lines.push(line);
    }
    // Strip trailing empty lines that come from the file's final CRLF
    while act_lines.last().is_some_and(|l| l.is_empty()) {
        act_lines.pop();
    }

    if act_lines.is_empty() {
        None
    } else {
        Some(act_lines)
    }
}

pub fn read_event_scripts(path: &Path) -> std::io::Result<Vec<EventScript>> {
    EventScript::read_file(path)
}

pub fn save_event_scripts(
    conn: &mut rusqlite::Connection,
    scripts: &[EventScript],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt_script = tx.prepare(include_str!("../queries/insert_event_script.sql"))?;
        let mut stmt_variable = tx.prepare(include_str!("../queries/insert_event_variable.sql"))?;
        let mut stmt_sprite = tx.prepare(include_str!("../queries/insert_event_sprite.sql"))?;
        let mut stmt_action = tx.prepare(include_str!("../queries/insert_event_action.sql"))?;

        for script in scripts {
            // Save the main script record
            let header_comments = script.header_comments.join("\n");
            let map_content = script.map_content.join("\n");
            let chr_content = script.chr_content.join("\n");
            let npc_content = script.npc_content.join("\n");
            let wav_content = script.wav_content.join("\n");

            // Convert event ID from string to integer
            let event_id = script.id;

            stmt_script.execute(rusqlite::params![
                event_id,
                header_comments,
                map_content,
                chr_content,
                npc_content,
                wav_content
            ])?;

            // Save variables
            for variable in &script.variables {
                stmt_variable.execute(rusqlite::params![
                    event_id,
                    variable.name,
                    variable.value
                ])?;
            }

            // Save sprites
            for sprite in &script.spr_content {
                stmt_sprite.execute(rusqlite::params![
                    event_id,
                    sprite.sprite_alias,
                    sprite.sprite_file
                ])?;
            }

            // Save actions
            for (order, action) in script.actions.iter().enumerate() {
                let parameters = action.parameters.join(", ");
                let raw_content = action.raw_content.clone().unwrap_or_default();

                stmt_action.execute(rusqlite::params![
                    event_id,
                    order as i32,
                    action.prefix.clone().unwrap_or_default(),
                    action.function_name,
                    parameters,
                    raw_content
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_sections() {
        let data = b"; header comment\n[VAR]\nspawn=5\n[MAP]\nmap_cmd1\n[CHR]\n[NPC]\n[SPR]\n[WAV]\n[ACT]\ndo_action(1)\n";
        let mut c = Cursor::new(data.as_ref());
        let scripts = EventScript::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(scripts.len(), 1);
        let s = &scripts[0];
        assert_eq!(s.id, 0); // ID=0 when parsed without a path
        assert_eq!(s.header_comments, vec!["; header comment"]);
        assert_eq!(s.variables.len(), 1);
        assert_eq!(s.variables[0].name, "spawn");
        assert_eq!(s.variables[0].value, "5");
        assert_eq!(s.map_content, vec!["map_cmd1"]);
        assert_eq!(s.actions.len(), 1);
    }

    #[test]
    fn parse_empty_sections() {
        let data = b"[VAR]\n[MAP]\n[CHR]\n[NPC]\n[SPR]\n[WAV]\n[ACT]\n";
        let mut c = Cursor::new(data.as_ref());
        let scripts = EventScript::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(scripts.len(), 1);
        let s = &scripts[0];
        assert!(s.variables.is_empty());
        assert!(s.map_content.is_empty());
        assert!(s.actions.is_empty());
    }

    #[test]
    fn serialize_round_trip() {
        let data =
            b"[VAR]\nspawn=5\n[MAP]\nmap_cmd1\n[CHR]\n[NPC]\n[SPR]\n[WAV]\n[ACT]\ndo_action(1)\n";
        let mut c = Cursor::new(data.as_ref());
        let records = EventScript::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        EventScript::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = EventScript::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].variables.len(), records2[0].variables.len());
        assert_eq!(records[0].variables[0].name, records2[0].variables[0].name);
        assert_eq!(records[0].actions, records2[0].actions);
    }
}
