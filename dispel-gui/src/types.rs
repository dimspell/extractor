use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Map,
    Ref,
    Database,
    Sprite,
    Sound,
    DbViewer,
    ChestEditor,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Map,
        Tab::Ref,
        Tab::Database,
        Tab::Sprite,
        Tab::Sound,
        Tab::DbViewer,
        Tab::ChestEditor,
    ];
    pub fn label(&self) -> &str {
        match self {
            Tab::Map => "Map",
            Tab::Ref => "Ref",
            Tab::Database => "Database",
            Tab::Sprite => "Sprite",
            Tab::Sound => "Sound",
            Tab::DbViewer => "DbViewer",
            Tab::ChestEditor => "ChestEditor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapOp {
    Tiles,
    Atlas,
    Render,
    FromDb,
    ToDb,
    Sprites,
}

impl MapOp {
    pub const ALL: &'static [MapOp] = &[
        MapOp::Tiles,
        MapOp::Atlas,
        MapOp::Render,
        MapOp::FromDb,
        MapOp::ToDb,
        MapOp::Sprites,
    ];
    pub fn label(&self) -> &str {
        match self {
            MapOp::Tiles => "Extract Tiles",
            MapOp::Atlas => "Generate Atlas",
            MapOp::Render => "Render Map",
            MapOp::FromDb => "Render from DB",
            MapOp::ToDb => "Import to DB",
            MapOp::Sprites => "Extract Sprites",
        }
    }
}

impl fmt::Display for MapOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefOp {
    AllMaps,
    Map,
    Extra,
    Event,
    Monster,
    Npc,
    Wave,
    PartyRef,
    DrawItem,
    PartyPgp,
    PartyDialog,
    Dialog,
    Weapons,
    Monsters,
    MultiMagic,
    Store,
    NpcRef,
    MonsterRef,
    MiscItem,
    HealItems,
    ExtraRef,
    EventItems,
    EditItems,
    PartyLevel,
    PartyIni,
    EventNpcRef,
    Magic,
    Quest,
    Message,
    ChData,
}

impl RefOp {
    pub const ALL: &'static [RefOp] = &[
        RefOp::AllMaps,
        RefOp::Map,
        RefOp::Extra,
        RefOp::Event,
        RefOp::Monster,
        RefOp::Npc,
        RefOp::Wave,
        RefOp::PartyRef,
        RefOp::DrawItem,
        RefOp::PartyPgp,
        RefOp::PartyDialog,
        RefOp::Dialog,
        RefOp::Weapons,
        RefOp::Monsters,
        RefOp::MultiMagic,
        RefOp::Store,
        RefOp::NpcRef,
        RefOp::MonsterRef,
        RefOp::MiscItem,
        RefOp::HealItems,
        RefOp::ExtraRef,
        RefOp::EventItems,
        RefOp::EditItems,
        RefOp::PartyLevel,
        RefOp::PartyIni,
        RefOp::EventNpcRef,
        RefOp::Magic,
        RefOp::Quest,
        RefOp::Message,
        RefOp::ChData,
    ];
    pub fn cli_name(&self) -> &str {
        match self {
            RefOp::AllMaps => "all-maps",
            RefOp::Map => "map",
            RefOp::Extra => "extra",
            RefOp::Event => "event",
            RefOp::Monster => "monster",
            RefOp::Npc => "npc",
            RefOp::Wave => "wave",
            RefOp::PartyRef => "party-ref",
            RefOp::DrawItem => "draw-item",
            RefOp::PartyPgp => "party-pgp",
            RefOp::PartyDialog => "party-dialog",
            RefOp::Dialog => "dialog",
            RefOp::Weapons => "weapons",
            RefOp::Monsters => "monsters",
            RefOp::MultiMagic => "multi-magic",
            RefOp::Store => "store",
            RefOp::NpcRef => "npc-ref",
            RefOp::MonsterRef => "monster-ref",
            RefOp::MiscItem => "misc-item",
            RefOp::HealItems => "heal-items",
            RefOp::ExtraRef => "extra-ref",
            RefOp::EventItems => "event-items",
            RefOp::EditItems => "edit-items",
            RefOp::PartyLevel => "party-level",
            RefOp::PartyIni => "party-ini",
            RefOp::EventNpcRef => "event-npc-ref",
            RefOp::Magic => "magic",
            RefOp::Quest => "quest",
            RefOp::Message => "message",
            RefOp::ChData => "ch-data",
        }
    }
}

impl fmt::Display for RefOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cli_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbOp {
    Import,
    DialogTexts,
    Maps,
    Databases,
    Refs,
    Rest,
}

impl DbOp {
    pub const ALL: &'static [DbOp] = &[
        DbOp::Import,
        DbOp::DialogTexts,
        DbOp::Maps,
        DbOp::Databases,
        DbOp::Refs,
        DbOp::Rest,
    ];
    pub fn label(&self) -> &str {
        match self {
            DbOp::Import => "Import All",
            DbOp::DialogTexts => "Dialog Texts",
            DbOp::Maps => "Maps",
            DbOp::Databases => "Databases",
            DbOp::Refs => "Refs",
            DbOp::Rest => "Rest",
        }
    }
    pub fn cli_name(&self) -> &str {
        match self {
            DbOp::Import => "import",
            DbOp::DialogTexts => "dialog-texts",
            DbOp::Maps => "maps",
            DbOp::Databases => "databases",
            DbOp::Refs => "refs",
            DbOp::Rest => "rest",
        }
    }
}

impl fmt::Display for DbOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteMode {
    Sprite,
    Animation,
}

impl SpriteMode {
    pub const ALL: &'static [SpriteMode] = &[SpriteMode::Sprite, SpriteMode::Animation];
}

impl fmt::Display for SpriteMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpriteMode::Sprite => write!(f, "Sprite"),
            SpriteMode::Animation => write!(f, "Animation"),
        }
    }
}
