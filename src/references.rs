use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::{EUC_KR, WINDOWS_1250};
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::io::{prelude::*, Cursor};
use std::io::{BufRead, BufReader, Result, Seek, SeekFrom};
use std::num::{IntErrorKind};
use std::{fs::File, path::Path};
use serde::{Deserialize, Serialize};

struct OnMapSpriteInfo {
    x: i32,
    y: i32,
    db_id: i32,
    sprite_id: i32,
    sprite_seq: i32,
    flip: bool,
}

pub fn read_ini() -> Result<()> {
    let f = File::open(&Path::new("sample-data/Extra.ini"))?;
    let mut reader = BufReader::new(f);

    loop {
        let mut line = String::new();
        let num = reader.read_line(&mut line)?;
        if num == 0 {
            break;
        }

        // println!("{line}");
        line.clear();
    }

    Ok(())
}

fn parse_null(s: &str) -> Option<String> {
    if s == "null" {
        None
    } else {
        Some(s.to_string())
    }
}

fn parse_int(s: &str) -> Option<i32> {
    match s.parse::<i32>() {
        Ok(value) => Some(value),
        Err(err) => match err.kind() {
            IntErrorKind::Empty => None,
            _ => {
                println!("{err:?} {s}");
                None
            }
        },
    }
}

// Message.scr
// first line of text
// second line of text or null
// third line of text or null

// Quest.scr
// id
// dairy type 0=main quest 1=side quest 2=traders journal
// title
// description

pub fn read_party_ini_db() {
    // ? something about party members
    todo!(); // PrtIni.db
}

// pub fn () {
//     // NPCs used only in events
//     //
//     // id
//     // sprite id
//     //     ?
//     //     ?
//     //     ?
//     //     ?
//     // x coordinate,
//     // y coordinate,
//     // 30 times ?

//     todo!(); // Eventnpc.ref
// }

// pub struct EventNpcRef {
//     id: i32,
//     event_id: i32,
//     name: String,
//     // _,
//     // _,

// }

// pub fn read_event_npc_ref(source_path: &Path) -> Result<Vec<MapIni>> {
//     let f = File::open(source_path)?;
//     let mut reader = BufReader::new(
//         DecodeReaderBytesBuilder::new()
//             .encoding(Some(WINDOWS_1250))
//             .build(f),
//     );
//     let mut map_inis: Vec<MapIni> = Vec::new();
//     for line in reader.lines() {
//         match line {
//             Ok(line) => {
//                 if line.starts_with(";") {
//                     continue;
//                 }
//                 println!("{line}");
//             }
//             _ => {
//                 println!("{:?}", line);
//             }
//         }
//     }
//     Ok(map_inis)
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Dialog {
    // Dialogs on map (translated from Korean)
    //
    // id,
    // previous event,
    // next dialog to check,
    // dialog type 0=normal 1=choose dialog
    // dialog topic(who is talking?) 0=main character 1=NPC
    // dialog id (ID in PGP file),
    // option 0 (dialog id),
    // option 1,
    // option 2,
    // event id to execute
    pub id: i32,
    pub previous_event_id: Option<i32>,
    pub next_dialog_to_check: Option<i32>,
    pub dialog_type_id: Option<i32>,
    pub dialog_owner: Option<i32>,
    pub dialog_id: Option<i32>,
    pub event_id: Option<i32>,
}

pub fn read_dialogs(source_path: &Path) -> Result<Vec<Dialog>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut dlgs: Vec<Dialog> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }
                let parts: Vec<&str> = line.split(",").collect();

                let id: i32 = parts[0].parse::<i32>().unwrap();
                let previous_event_id = parse_int(parts[1]);
                let next_dialog_to_check = parse_int(parts[2]);
                let dialog_type_id = parse_int(parts[3]);
                let dialog_owner = parse_int(parts[4]);
                let dialog_id = parse_int(parts[5]);
                let event_id = parse_int(parts[6]);

                dlgs.push(Dialog {
                    id,
                    previous_event_id,
                    next_dialog_to_check,
                    dialog_type_id,
                    dialog_owner,
                    dialog_id,
                    event_id,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(dlgs)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyPgp {
    pub id: i32,
    pub dialog_text: Option<String>,
    pub unknown_id1: Option<i32>,
    pub unknown_id2: Option<i32>,
}

pub fn read_party_pgps(source_path: &Path) -> Result<Vec<PartyPgp>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(f),
    );
    let mut pgps: Vec<PartyPgp> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }
                let parts: Vec<&str> = line.split("|").collect();

                let id: i32 = parts[0].parse::<i32>().unwrap();
                let dialog_text = parse_null(parts[1]);
                let unknown_id1 = parse_int(parts[2]);
                let unknown_id2 = parse_int(parts[3]);
                pgps.push(PartyPgp {
                    id,
                    dialog_text,
                    unknown_id1,
                    unknown_id2,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(pgps)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    pub id: i32,
    pub map_filename: String,
    pub map_name: String,
    pub pgp_filename: Option<String>,
    pub dlg_filename: Option<String>,
    // light - 0=light, 1=darkness
    pub is_light: bool,
}

pub fn read_all_map_ini(source_path: &Path) -> Result<Vec<Map>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(f),
    );

    let mut maps: Vec<Map> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                let id: i32 = parts[0].parse::<i32>().unwrap();
                let map_filename = parts[1].to_string();
                let map_name = parts[2].to_string();
                let pgp_filename = parse_null(parts[3]);
                let dlg_filename = parse_null(parts[4]);
                let is_light: bool = parts[5] == "1";

                maps.push(Map {
                    id,
                    map_filename,
                    map_name,
                    pgp_filename,
                    dlg_filename,
                    is_light,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(maps)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapIni {
    pub id: i32,
    // id
    pub event_id_on_camera_move: i32,
    // event that occurs when camera moves
    pub start_pos_x: i32,
    // start position X
    pub start_pos_y: i32,
    // start position Y
    pub map_id: i32,
    // map id
    pub monsters_filename: Option<String>,
    // monsters filename
    pub npc_filename: Option<String>,
    // NPC filename
    pub extra_filename: Option<String>,
    // extra filename
    pub cd_music_track_number: i32,        // CD music track number
}

pub fn read_map_ini(source_path: &Path) -> Result<Vec<MapIni>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut map_inis: Vec<MapIni> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                let id: i32 = parts[0].parse::<i32>().unwrap();
                let event_id_on_camera_move = parts[1].parse::<i32>().unwrap();
                let start_pos_x = parts[2].parse::<i32>().unwrap();
                let start_pos_y = parts[3].parse::<i32>().unwrap();
                let map_id = parts[4].parse::<i32>().unwrap();
                let monsters_filename = parse_null(parts[5]);
                let npc_filename = parse_null(parts[6]);
                let extra_filename = parse_null(parts[7]);
                let cd_music_track_number = parts[8].parse::<i32>().unwrap();

                map_inis.push(MapIni {
                    id,
                    event_id_on_camera_move,
                    start_pos_x,
                    start_pos_y,
                    map_id,
                    monsters_filename,
                    npc_filename,
                    extra_filename,
                    cd_music_track_number,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(map_inis)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extra {
    pub id: i32,
    pub sprite_filename: Option<String>,
    pub unknown: i32,
    pub description: Option<String>,
}

pub fn read_extra_ini(source_path: &Path) -> Result<Vec<Extra>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut extras: Vec<Extra> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                let id: i32 = parts[0].parse::<i32>().unwrap();
                let sprite_filename = parse_null(parts[1]);
                let unknown = parts[2].parse::<i32>().unwrap();
                let description = parse_null(parts[3]);

                extras.push(Extra {
                    id,
                    sprite_filename,
                    unknown,
                    description,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(extras)
}

enum EventType {
    // type (Translation from Korean)
    // 0 unconditionally executes once (ignores previous event)
    // 1 unconditionally executes N times (ignores previous event)
    // 2 unconditionally executed (ignores previous event)
    // 3 executed once when previous event failed
    // 4 before event. Execute N times when condition is true
    // 5 execute event when previous event is successful
    // 6 execute once when previous event is successful
    // 7 execute N times, when previous event is successful
    // 8 continues event, when previous event is successful
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_id: i32,
    pub previous_event_id: i32,
    pub event_type_id: i32,
    pub event_filename: Option<String>,
    pub counter: i32, // N counter
}

pub fn read_event_ini(source_path: &Path) -> Result<Vec<Event>> {
    let f = File::open(source_path)?;
    // let mut reader = BufReader::new(f);
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );

    let mut events: Vec<Event> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                let event_id = parts[0].parse::<i32>().unwrap();
                let previous_event_id: i32 = parts[1].parse::<i32>().unwrap();
                let event_type_id = parts[2].parse::<i32>().unwrap();
                let event_filename = parse_null(parts[3]);
                let counter = parts[4].parse::<i32>().unwrap();

                events.push(Event {
                    event_id,
                    previous_event_id,
                    event_type_id,
                    event_filename,
                    counter,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(events)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonsterIni {
    pub id: i32,
    pub name: Option<String>,
    pub sprite_filename: Option<String>,
    pub attack: i32,
    // animation sequence number
    pub hit: i32,
    // animation sequence number
    pub death: i32,
    // animation sequence number
    pub walking: i32,
    // animation sequence number
    pub casting_magic: i32, // animation sequence number
}

pub fn read_monster_ini(source_path: &Path) -> Result<Vec<MonsterIni>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(f),
    );
    let mut monsters: Vec<MonsterIni> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let name = parse_null(parts[1]);
                let sprite_filename = parse_null(parts[2]);
                let attack = parts[3].parse::<i32>().unwrap();
                let hit = parts[4].parse::<i32>().unwrap();
                let death = parts[5].parse::<i32>().unwrap();
                let walking = parts[6].parse::<i32>().unwrap();
                let casting_magic = parts[7].parse::<i32>().unwrap();

                monsters.push(MonsterIni {
                    id,
                    name,
                    sprite_filename,
                    attack,
                    hit,
                    death,
                    walking,
                    casting_magic,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(monsters)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NpcIni {
    pub id: i32,
    pub sprite_filename: Option<String>,
    pub description: String,
}

pub fn read_npc_ini(source_path: &Path) -> Result<Vec<NpcIni>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut npc_inis: Vec<NpcIni> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let sprite_filename = parse_null(parts[1]);
                let description = parts[2].to_string();

                npc_inis.push(NpcIni {
                    id,
                    sprite_filename,
                    description,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(npc_inis)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WaveIni {
    pub id: i32,
    pub snf_filename: Option<String>,
    pub unknown_flag: Option<String>,
}

pub fn read_wave_ini(source_path: &Path) -> Result<Vec<WaveIni>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut waves_inis: Vec<WaveIni> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let snf_filename = parse_null(parts[1]);
                let unknown_flag = parse_null(parts[2]);

                waves_inis.push(WaveIni {
                    id,
                    snf_filename,
                    unknown_flag,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(waves_inis)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyRef {
    pub id: i32,
    pub full_name: Option<String>,
    pub job_name: Option<String>,
    pub root_map_id: i32,
    pub npc_id: i32,
    pub dlg_when_not_in_party: i32,
    pub dlg_when_in_party: i32,
    pub ghost_face_id: i32,
}

pub fn read_part_refs(source_path: &Path) -> Result<Vec<PartyRef>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(f),
    );
    let mut party_refs: Vec<PartyRef> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let full_name = parse_null(parts[1]);
                let job_name = parse_null(parts[2]);
                let root_map_id = parts[3].parse::<i32>().unwrap();
                let npc_id = parts[4].parse::<i32>().unwrap();
                let dlg_when_not_in_party = parts[5].parse::<i32>().unwrap();
                let dlg_when_in_party = parts[6].parse::<i32>().unwrap();
                let ghost_face_id = parts[7].parse::<i32>().unwrap();

                party_refs.push(PartyRef {
                    id,
                    full_name,
                    job_name,
                    root_map_id,
                    npc_id,
                    dlg_when_not_in_party,
                    dlg_when_in_party,
                    ghost_face_id,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(party_refs)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrawItem {
    pub map_id: i32,
    pub x_coord: i32,
    pub y_coord: i32,
    pub item_id: i32, // item id (int32 but [item id, group id, 0 , 0]])
}

pub fn read_draw_items(source_path: &Path) -> Result<Vec<DrawItem>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut draw_items: Vec<DrawItem> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line
                    .trim_start_matches("(")
                    .trim_end_matches(")")
                    .split(",")
                    .collect();

                let map_id = parts[0].parse::<i32>().unwrap();
                let x_coord = parts[1].parse::<i32>().unwrap();
                let y_coord = parts[2].parse::<i32>().unwrap();
                let item_id = parts[3].parse::<i32>().unwrap();

                draw_items.push(DrawItem {
                    map_id,
                    x_coord,
                    y_coord,
                    item_id,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(draw_items)
}

#[derive(Debug)]
pub struct EditItem {
    pub index: i32,
    pub name: String,
    pub description: String,
    pub base_price: i16,
    pub pz: i16,
    pub pm: i16,
    pub sil: i16,
    pub zw: i16,
    pub mm: i16,
    pub tf: i16,
    pub unk: i16,
    pub trf: i16,
    pub atk: i16,
    pub obr: i16,
    pub item_destroying_power: i16,
    pub modifies_item: u8,
    pub additional_effect: i16,
}

pub fn read_edit_item_db(source_path: &Path) -> Result<Vec<EditItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 67 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut items: Vec<EditItem> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 30];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; 202];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let description = dst.0.trim_end_matches("\0").trim();

        let base_price = reader.read_i16::<LittleEndian>()?;

        let mut buffer = [0u8; 3 * 2];
        reader.read_exact(&mut buffer)?;

        let pz = reader.read_i16::<LittleEndian>()?; // PZ
        let pm = reader.read_i16::<LittleEndian>()?; // PM
        let sil = reader.read_i16::<LittleEndian>()?; // SIŁ
        let zw = reader.read_i16::<LittleEndian>()?; // ZW
        let mm = reader.read_i16::<LittleEndian>()?; // MM
        let tf = reader.read_i16::<LittleEndian>()?; // TF
        let unk = reader.read_i16::<LittleEndian>()?; // UNK
        let trf = reader.read_i16::<LittleEndian>()?; // TRF
        let atk = reader.read_i16::<LittleEndian>()?; // ATK
        let obr = reader.read_i16::<LittleEndian>()?; // OBR

        reader.read_i16::<LittleEndian>()?;

        let item_destroying_power = reader.read_i16::<LittleEndian>()?; // durability probably
        reader.read_u8()?;

        let modifies_item = reader.read_u8()?;
        let additional_effect = reader.read_i16::<LittleEndian>()?; // poison or burn or none

        items.push(EditItem {
            index: i,
            name: name.to_string(),
            description: description.to_string(),
            base_price,
            pz,
            pm,
            sil,
            zw,
            mm,
            tf,
            unk,
            trf,
            atk,
            obr,
            item_destroying_power,
            modifies_item,
            additional_effect,
        })
    }
    Ok(items)
}

#[derive(Debug)]
pub struct EventItem {
    pub id: i32,
    pub name: String,
    pub description: String,
}

pub fn read_event_item_db(source_path: &Path) -> Result<Vec<EventItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 60 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut items: Vec<EventItem> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 30];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; 202];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let description = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;

        items.push(EventItem {
            id: i,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    Ok(items)
}

#[derive(Debug)]
pub struct ExtraRef {
    pub id: i32,
    pub number_in_file: u8,
    pub ext_id: u8,
    pub name: String,
    pub object_type: u8,
    pub x_pos: i32,
    pub y_pos: i32,
    pub rotation: u8,
    pub closed: i32,
    pub required_item_id: u8,
    pub required_item_type_id: u8,
    pub required_item_id2: u8,
    pub required_item_type_id2: u8,
    pub gold_amount: i32,
    pub item_id: u8,
    pub item_type_id: u8,
    pub item_count: i32,
    pub event_id: i32,
    pub message_id: i32,
    pub visibility: u8,
}

pub fn read_extra_ref(source_path: &Path) -> Result<Vec<ExtraRef>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 46 * 4;
    // const FILLER: u8 = 0xcd;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut refs: Vec<ExtraRef> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let number_in_file = reader.read_u8()?;

        reader.read_u8()?;
        let ext_id = reader.read_u8()?; // Id from Extra.ini

        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let object_type = reader.read_u8()?; // 7-magic, 6-interactive object, 5-altar, 4-sign, 2-door, 0-chest

        let x_pos = reader.read_i32::<LittleEndian>()?;
        let y_pos = reader.read_i32::<LittleEndian>()?;
        let rotation = reader.read_u8()?;

        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        reader.read_i32::<LittleEndian>()?;

        let closed = reader.read_i32::<LittleEndian>()?; // chest 0-open, 1-closed

        let required_item_id = reader.read_u8()?; // lower bound
        let required_item_type_id = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let required_item_id2 = reader.read_u8()?; // upper bound
        let required_item_type_id2 = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let mut buffer = [0u8; 16];
        reader.read_exact(&mut buffer)?;

        let gold_amount = reader.read_i32::<LittleEndian>()?;

        let item_id = reader.read_u8()?;
        let item_type_id = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let item_count = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 40];
        reader.read_exact(&mut buffer)?;

        let event_id = reader.read_i32::<LittleEndian>()?; // id from event.ini
        let message_id = reader.read_i32::<LittleEndian>()?; // id from message.scr for signs

        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;

        let visibility = reader.read_u8()?;

        let mut buffer = [0u8; 3];
        reader.read_exact(&mut buffer)?;
        println!("{:?}", buffer);

        refs.push(ExtraRef {
            id: i,
            number_in_file,
            ext_id,
            name: name.to_string(),
            object_type,
            x_pos,
            y_pos,
            rotation,
            closed,
            required_item_id,
            required_item_type_id,
            required_item_id2,
            required_item_type_id2,
            gold_amount,
            item_id,
            item_type_id,
            item_count,
            event_id,
            message_id,
            visibility,
        })
    }

    Ok(refs)
}

#[derive(Debug)]
pub struct HealItem {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_price: i16,
    pub pz: i16,
    pub pm: i16,
    pub full_pz: u8,
    pub full_pm: u8,
    pub poison_heal: u8,
    pub petrif_heal: u8,
    pub polimorph_heal: u8,
}

pub fn read_heal_item_db(source_path: &Path) -> Result<Vec<HealItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 63 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut items: Vec<HealItem> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 30];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; 202];
        reader.read_exact(&mut buffer)?;
        let dst = EUC_KR.decode(&buffer);
        let description = dst.0.trim_end_matches("\0").trim();

        let base_price = reader.read_i16::<LittleEndian>()?;

        reader.read_i16::<LittleEndian>()?;
        reader.read_i16::<LittleEndian>()?;
        reader.read_i16::<LittleEndian>()?;

        let pz = reader.read_i16::<LittleEndian>()?;
        let pm = reader.read_i16::<LittleEndian>()?;
        let full_pz = reader.read_u8()?;
        let full_pm = reader.read_u8()?;
        let poison_heal = reader.read_u8()?;
        let petrif_heal = reader.read_u8()?;
        let polimorph_heal = reader.read_u8()?;

        reader.read_u8()?;
        reader.read_i16::<LittleEndian>()?;

        items.push(HealItem {
            id: i,
            name: name.to_string(),
            description: description.to_string(),
            base_price,
            pz,
            pm,
            full_pz,
            full_pm,
            poison_heal,
            petrif_heal,
            polimorph_heal,
        })
    }

    Ok(items)
}

#[derive(Debug)]
pub struct MiscItem {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_price: i32,
}

pub fn read_misc_item_db(source_path: &Path) -> Result<Vec<MiscItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 64 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut items: Vec<MiscItem> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 30];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; 202];
        reader.read_exact(&mut buffer)?;
        let dst = EUC_KR.decode(&buffer);
        let description = dst.0.trim_end_matches("\0").trim();

        let base_price = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 20];
        reader.read_exact(&mut buffer)?;
        let cursor = Cursor::new(&buffer);

        let dst = EUC_KR.decode(&buffer);
        let dst = dst.0.trim_end_matches("\0");
        let dst = dst.trim_start_matches("\0");
        // let name = dst.0.trim_end_matches("\0").trim();

        println!("{name} {description} {:?}, {dst:?}", cursor);

        items.push(MiscItem {
            id: i,
            base_price,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    Ok(items)
}

#[derive(Debug)]
pub struct MonsterRef {
    pub index: i32,
    pub file_id: i32,
    pub mon_id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub loot1_item_id: u8,
    pub loot1_item_type: u8,
    pub loot2_item_id: u8,
    pub loot2_item_type: u8,
    pub loot3_item_id: u8,
    pub loot3_item_type: u8,
}

pub fn read_monster_ref(source_path: &Path) -> Result<Vec<MonsterRef>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 14 * 4;
    // const FILLER: u8 = 0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut refs: Vec<MonsterRef> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let file_id = reader.read_i32::<LittleEndian>()?;
        let mon_id = reader.read_i32::<LittleEndian>()?;
        let pos_x = reader.read_i32::<LittleEndian>()?;
        let pos_y = reader.read_i32::<LittleEndian>()?;

        reader.read_i32::<LittleEndian>()?;
        reader.read_i32::<LittleEndian>()?;
        reader.read_i32::<LittleEndian>()?;
        reader.read_i32::<LittleEndian>()?;
        reader.read_i32::<LittleEndian>()?;

        let loot1_item_id = reader.read_u8()?;
        let loot1_item_type = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let loot2_item_id = reader.read_u8()?;
        let loot2_item_type = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let loot3_item_id = reader.read_u8()?;
        let loot3_item_type = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        reader.read_i32::<LittleEndian>()?; // 1 or 0
        reader.read_i32::<LittleEndian>()?;

        refs.push(MonsterRef {
            index: i,
            file_id,
            mon_id,
            pos_x,
            pos_y,
            loot1_item_id,
            loot1_item_type,
            loot2_item_id,
            loot2_item_type,
            loot3_item_id,
            loot3_item_type,
        })
    }

    Ok(refs)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub health_points_max: i32,
    pub health_points_min: i32,
    pub magic_points_max: i32,
    pub magic_points_min: i32,
    pub walk_speed: i32,
    pub to_hit_max: i32,
    pub to_hit_min: i32,
    pub to_dodge_max: i32,
    pub to_dodge_min: i32,
    pub offense_max: i32,
    pub offense_min: i32,
    pub defense_max: i32,
    pub defense_min: i32,
    pub magic_attack_max: i32,
    pub magic_attack_min: i32,
    pub is_undead: i32,
    pub has_blood: i32,
    pub ai_type: i32,
    pub exp_gain_max: i32,
    pub exp_gain_min: i32,
    pub gold_drop_max: i32,
    pub gold_drop_min: i32,
    pub detection_sight_size: i32,
    pub distance_range_size: i32,
    pub known_spell_slot1: i32,
    pub known_spell_slot2: i32,
    pub known_spell_slot3: i32,
    pub is_oversize: i32,
    pub magic_level: i32,
    pub special_attack: i32,
    pub special_attack_chance: i32,
    pub special_attack_duration: i32,
    pub boldness: i32,
    pub attack_speed: i32,
}

pub fn read_monster_db(source_path: &Path) -> Result<Vec<Monster>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 0;
    const PROPERTY_ITEM_SIZE: i32 = 40 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut monsters: Vec<Monster> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 24];
        reader.read_exact(&mut buffer)?;
        let dst = EUC_KR.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let health_points_max = reader.read_i32::<LittleEndian>()?;
        let health_points_min = reader.read_i32::<LittleEndian>()?;
        let magic_points_max = reader.read_i32::<LittleEndian>()?;
        let magic_points_min = reader.read_i32::<LittleEndian>()?;

        let walk_speed = reader.read_i32::<LittleEndian>()?;

        let to_hit_max = reader.read_i32::<LittleEndian>()?;
        let to_hit_min = reader.read_i32::<LittleEndian>()?;

        let to_dodge_max = reader.read_i32::<LittleEndian>()?; // always = 10
        let to_dodge_min = reader.read_i32::<LittleEndian>()?; // always = 10

        let offense_max = reader.read_i32::<LittleEndian>()?;
        let offense_min = reader.read_i32::<LittleEndian>()?;

        let defense_max = reader.read_i32::<LittleEndian>()?;
        let defense_min = reader.read_i32::<LittleEndian>()?;

        let magic_attack_max = reader.read_i32::<LittleEndian>()?; // max
        let magic_attack_min = reader.read_i32::<LittleEndian>()?; // min

        let is_undead = reader.read_i32::<LittleEndian>()?; // "0 or 1"
        let has_blood = reader.read_i32::<LittleEndian>()?; // "0 or 1, golem is not alive and not undead"
        let ai_type = reader.read_i32::<LittleEndian>()?; // "goblin and chicken = 1,archers = 2, worm bot no zombie =3, deer and dog = 5"

        let exp_gain_max = reader.read_i32::<LittleEndian>()?;
        let exp_gain_min = reader.read_i32::<LittleEndian>()?;

        let gold_drop_max = reader.read_i32::<LittleEndian>()?;
        let gold_drop_min = reader.read_i32::<LittleEndian>()?;

        let detection_sight_size = reader.read_i32::<LittleEndian>()?; // "9 or 10 - only goblin king have 10"
        let distance_range_size = reader.read_i32::<LittleEndian>()?; // "1 or 6 if archer

        let known_spell_slot1 = reader.read_i32::<LittleEndian>()?;
        let known_spell_slot2 = reader.read_i32::<LittleEndian>()?;
        let known_spell_slot3 = reader.read_i32::<LittleEndian>()?;

        let is_oversize = reader.read_i32::<LittleEndian>()?; // redDragon, balrog, beholder, = 1

        let magic_level = reader.read_i32::<LittleEndian>()?; // always = 1

        let special_attack = reader.read_i32::<LittleEndian>()?; // 0 = none, 1 = bat/zombie/biteworm, 2 = basilisk
        let special_attack_chance = reader.read_i32::<LittleEndian>()?;
        let special_attack_duration = reader.read_i32::<LittleEndian>()?;

        let boldness = reader.read_i32::<LittleEndian>()?; // always = 10
        let attack_speed = reader.read_i32::<LittleEndian>()?;

        monsters.push(Monster {
            id: i,
            name: name.to_string(),
            health_points_max,
            health_points_min,
            magic_points_max,
            magic_points_min,
            walk_speed,
            to_hit_max,
            to_hit_min,
            to_dodge_max,
            to_dodge_min,
            offense_max,
            offense_min,
            defense_max,
            defense_min,
            magic_attack_max,
            magic_attack_min,
            is_undead,
            has_blood,
            ai_type,
            exp_gain_max,
            exp_gain_min,
            gold_drop_max,
            gold_drop_min,
            detection_sight_size,
            distance_range_size,
            known_spell_slot1,
            known_spell_slot2,
            known_spell_slot3,
            is_oversize,
            magic_level,
            special_attack,
            special_attack_chance,
            special_attack_duration,
            boldness,
            attack_speed,
        })
    }

    Ok(monsters)
}

fn read_multi_monster_db() {
    todo!();
}

#[derive(Debug)]
pub struct NPC {
    pub index: i32,
    pub id: i32,
    pub npc_id: i32,
    pub name: String,
    pub party_script_id: i32,
    pub show_on_event: i32,
    pub goto1_filled: i32,
    pub goto2_filled: i32,
    pub goto3_filled: i32,
    pub goto4_filled: i32,
    pub goto1_x: i32,
    pub goto2_x: i32,
    pub goto3_x: i32,
    pub goto4_x: i32,
    pub goto1_y: i32,
    pub goto2_y: i32,
    pub goto3_y: i32,
    pub goto4_y: i32,
    pub looking_direction: i32,
    pub dialog_id: i32,
}

pub fn read_npc_ref(source_path: &Path) -> Result<Vec<NPC>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 0x2a0;
    // const FILLER: u8 = 205; // 0xCD
    const STRING_MAX_LENGTH: usize = 260;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut npcs: Vec<NPC> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let id = reader.read_i32::<LittleEndian>()?;
        let npc_id = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; STRING_MAX_LENGTH];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; STRING_MAX_LENGTH];
        reader.read_exact(&mut buffer)?;
        // let dst = WINDOWS_1250.decode(&buffer);
        // let test = dst.0.trim_end_matches("\0").trim();

        let party_script_id = reader.read_i32::<LittleEndian>()?;
        let show_on_event = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("4: {:?}", cursor);

        let goto1_filled = reader.read_i32::<LittleEndian>()?;
        let goto2_filled = reader.read_i32::<LittleEndian>()?;
        let goto3_filled = reader.read_i32::<LittleEndian>()?;
        let goto4_filled = reader.read_i32::<LittleEndian>()?; // "when goto4 not filled its 1, idk why

        let goto1_x = reader.read_i32::<LittleEndian>()?;
        let goto2_x = reader.read_i32::<LittleEndian>()?;
        let goto3_x = reader.read_i32::<LittleEndian>()?;
        let goto4_x = reader.read_i32::<LittleEndian>()?;

        let goto1_y = reader.read_i32::<LittleEndian>()?;
        let goto2_y = reader.read_i32::<LittleEndian>()?;
        let goto3_y = reader.read_i32::<LittleEndian>()?;
        let goto4_y = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 16];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("16: {:?}", cursor);

        let looking_direction = reader.read_i32::<LittleEndian>()?; // 0 = up, clockwise

        let mut buffer = [0u8; 16 + 16 + 16 + 8];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("56: {:?}", cursor);

        let dialog_id = reader.read_i32::<LittleEndian>()?; // also text for shop

        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("Last: {:?}", cursor);

        npcs.push(NPC {
            index: i,
            id,
            npc_id,
            name: name.to_string(),
            party_script_id,
            show_on_event,
            goto1_filled,
            goto2_filled,
            goto3_filled,
            goto4_filled,
            goto1_x,
            goto2_x,
            goto3_x,
            goto4_x,
            goto1_y,
            goto2_y,
            goto3_y,
            goto4_y,
            looking_direction,
            dialog_id,
        })
    }

    Ok(npcs)
}

#[derive(Debug)]
pub struct Store {
    pub index: i32,
    pub store_name: String,
    pub inn_night_cost: i32,
    pub some_unknown_number: i16,
    pub products: Vec<StoreProduct>,
    pub invitation: String,
    pub haggle_success: String,
    pub haggle_fail: String,
}

pub type StoreProduct = (i16, i16, i16); // order, product_type, product_id

pub fn read_store_db(source_path: &Path) -> Result<Vec<Store>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 237 * 4;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    let mut store: Vec<Store> = vec![];
    for i in 0..elements as usize {
        // name
        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let inn_night_cost = reader.read_i32::<LittleEndian>()?;
        let mut some_unknown_number = 0;
        let mut products: Vec<StoreProduct> = vec![];

        if inn_night_cost > 0 {
            reader.seek(SeekFrom::Current(144))?;
        } else {
            some_unknown_number = reader.read_i16::<LittleEndian>()?; // price modifier?

            let mut buffer = [0u8; 142];
            reader.read_exact(&mut buffer)?;
            let mut cursor = Cursor::new(&buffer);

            for i in 0..buffer.len() / 2 {
                let item_type = cursor.read_i16::<LittleEndian>().unwrap();
                // 1 = Bron
                // 2,3 = wyposazenie (3 = edibles/ 2 =modfiers?)
                // 4 = magiczny
                if item_type == 0 {
                    break;
                }

                let item_id = cursor.read_i16::<LittleEndian>().unwrap();
                products.push((i as i16, item_type, item_id));
            }
        }

        // text
        let mut buffer = [0u8; 512];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let invitation = dst.0.trim_end_matches("\0").trim();

        // haggle_success
        let mut buffer = [0u8; 128];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let haggle_success = dst.0.trim_end_matches("\0").trim();

        // haggle_fail
        let mut buffer = [0u8; 128];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let haggle_fail = dst.0.trim_end_matches("\0").trim();

        let item = Store {
            index: i as i32,
            store_name: name.to_string(),
            inn_night_cost,
            some_unknown_number,
            products,
            invitation: invitation.to_string(),
            haggle_success: haggle_success.to_string(),
            haggle_fail: haggle_fail.to_string(),
        };

        store.push(item);
    }

    Ok(store)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeaponItem {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_price: i16,
    pub health_points: i16,
    pub magic_points: i16,
    pub strength: i16,
    pub agility: i16,
    pub wisdom: i16,
    pub tf: i16,
    pub unk: i16,
    pub trf: i16,
    pub attack: i16,
    pub defense: i16,
    pub mag: i16,
    pub durability: i16,
    pub req_strength: i16,
    pub req_zw: i16,
    pub req_wisdom: i16,
}

pub fn read_weapons_db(source_path: &Path) -> Result<Vec<WeaponItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 71 * 4;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    const NAME_STRING_MAX_LENGTH: usize = 30;
    const DESCRIPTION_STRING_MAX_LENGTH: usize = 202;

    let mut weapons: Vec<WeaponItem> = vec![];
    for i in 0..elements as usize {
        // println!("{i}");

        // name
        let mut buffer = [0u8; NAME_STRING_MAX_LENGTH];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();
        // println!("{:?}", name);

        // description
        let mut buffer = [0u8; DESCRIPTION_STRING_MAX_LENGTH];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let description = dst.0.trim_end_matches("\0").trim_end_matches("\00").trim();
        // println!("{:?}", description);

        // "Base price"
        let base_price = reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        // "PZ"
        let health_points = reader.read_i16::<LittleEndian>()?;
        // "PM"
        let magic_points = reader.read_i16::<LittleEndian>()?;
        // "SIŁ"
        let strength = reader.read_i16::<LittleEndian>()?;
        // "ZW"
        let zw = reader.read_i16::<LittleEndian>()?;
        // "MM"
        let wisdom = reader.read_i16::<LittleEndian>()?;
        // "TF"
        let tf = reader.read_i16::<LittleEndian>()?;
        // "UNK"
        let unk = reader.read_i16::<LittleEndian>()?;
        // "TRF"
        let trf = reader.read_i16::<LittleEndian>()?;
        // "ATK"
        let attack = reader.read_i16::<LittleEndian>()?;
        // "OBR"
        let defense = reader.read_i16::<LittleEndian>()?;
        // "MAG"
        let mag = reader.read_i16::<LittleEndian>()?;
        // "WYT"
        let durability = reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        // "REQ SIŁ"
        let req_strength = reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        // "REQ ZW"
        let req_zw = reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        // "REQ MM"
        let req_wisdom = reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;
        //
        reader.read_i16::<LittleEndian>()?;

        let item = WeaponItem {
            id: i as i32,
            attack,
            base_price,
            description: description.to_string(),
            mag,
            wisdom,
            name: name.to_string(),
            defense,
            magic_points,
            health_points,
            req_wisdom,
            req_strength,
            req_zw,
            strength,
            tf,
            trf,
            unk,
            durability,
            agility: zw,
        };
        weapons.push(item);
    }

    Ok(weapons)
}


pub fn read_party_level_db(source_path: &Path) -> Result<()> {
    // 5760
    // Divisors of number 5760: 1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 16, 18, 20, 24, 30, 32, 36, 40, 45, 48, 60, 64, 72, 80, 90, 96, 120, 128, 144, 160, 180, 192, 240, 288, 320, 360, 384, 480, 576, 640, 720, 960, 1152, 1440, 1920, 2880, 5760

    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    // let mut buffer: Vec<u8> = Vec::new();
    // reader.read_to_end(&mut buffer)?;
    // let dst = WINDOWS_1250.decode(&buffer);
    // println!("{:?}", buffer.len());
    // println!("{:?}", dst.0);
    // let pos = reader.seek(SeekFrom::Current(0))?;
    // println!("{file_len} {pos}");

    const COUNTER_SIZE: u8 = 0;
    const PROPERTY_ITEM_SIZE: i32 = 180 * 4;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    // let pos = reader.seek(SeekFrom::Current(0))?;

    println!("{elements}");

    for i in 0..16 {
        let mut buffer = [0u8; 360];
        reader.read_exact(&mut buffer)?;

        let cursor = Cursor::new(&buffer);
        println!("{i} {cursor:?}");

        let dst = EUC_KR.decode(&buffer);
        println!("{:?}", dst.0);
    }

    Ok(())
}

pub fn read_mutli_magic_db(source_path: &Path) -> Result<()> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 90;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    let pos = reader.seek(SeekFrom::Current(0))?;
    println!("{:?} {:?} {:?} {:?}", file_len, elements, pos, PROPERTY_ITEM_SIZE * elements);

    for i in 0..elements {
        let mut buffer = [0u8; 90];
        reader.read_exact(&mut buffer)?;
        println!("{:?}", buffer);
    }

    // println!("{:?} {:?} {:?}", file_len, elements, pos);
    Ok(())
}

pub fn read_mapper(
    reader: &mut BufReader<File>,
    file_len: u64,
    counter_size: u8,
    property_item_size: i32,
) -> Result<i32> {
    let mut expected_elements = 0;
    let space_for_elements =
        (((file_len - counter_size as u64) as f64) / property_item_size as f64).floor();
    let space_for_elements: i32 = space_for_elements as i32;

    if counter_size > 0 {
        expected_elements = reader.read_i32::<LittleEndian>()?;
    } else {
        expected_elements = space_for_elements;
    }
    if expected_elements != space_for_elements {
        println!(
            "expected_elements: {expected_elements} / space_for_elements: {space_for_elements}"
        );
    }

    Ok(space_for_elements)
}
