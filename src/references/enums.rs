// Enumerations for Dispel game data
//
// This module defines type-safe enums for various game entities,
// replacing magic numbers with named variants for better code quality
// and maintainability.

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Event types for game scripting and quest progression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum EventType {
    /// Default/Unknown event type
    Unknown = 0,
    /// Conditional execution (executed N times unconditionally)
    Conditional = 2,
    /// Continue execution when previous event is unsatisfied
    ContinueOnUnsatisfied = 5,
    /// Execute 1 time when previous event is satisfied
    ExecuteOnSatisfied = 6,
}

impl EventType {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(EventType::Unknown),
            2 => Some(EventType::Conditional),
            5 => Some(EventType::ContinueOnUnsatisfied),
            6 => Some(EventType::ExecuteOnSatisfied),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for EventType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        EventType::from_i32(value).ok_or("Invalid event type value")
    }
}

impl From<EventType> for i32 {
    fn from(event_type: EventType) -> Self {
        event_type.value()
    }
}

/// Monster AI behavior types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum MonsterAiType {
    /// Passive monster with no AI
    Passive = 0,
    /// Aggressive monster that attacks on sight
    Aggressive = 1,
    /// Defensive monster that only attacks when provoked
    Defensive = 2,
    /// Ranged attacker
    Ranged = 3,
    /// Boss monster with special behavior
    Boss = 4,
    /// Special AI behavior
    Special = 5,
    /// Custom scripted AI
    Custom = 6,
}

impl MonsterAiType {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(MonsterAiType::Passive),
            1 => Some(MonsterAiType::Aggressive),
            2 => Some(MonsterAiType::Defensive),
            3 => Some(MonsterAiType::Ranged),
            4 => Some(MonsterAiType::Boss),
            5 => Some(MonsterAiType::Special),
            6 => Some(MonsterAiType::Custom),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for MonsterAiType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        MonsterAiType::from_i32(value).ok_or("Invalid monster AI type value")
    }
}

impl From<MonsterAiType> for i32 {
    fn from(ai_type: MonsterAiType) -> Self {
        ai_type.value()
    }
}

/// Binary property flag (Present/Absent)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum PropertyFlag {
    /// Property is false/absent
    Absent = 0,
    /// Property is true/present
    Present = 1,
}

impl PropertyFlag {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(PropertyFlag::Absent),
            1 => Some(PropertyFlag::Present),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }

    /// Convert to boolean
    pub fn as_bool(&self) -> bool {
        *self == PropertyFlag::Present
    }
}

impl TryFrom<i32> for PropertyFlag {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        PropertyFlag::from_i32(value).ok_or("Invalid property flag value")
    }
}

impl From<PropertyFlag> for i32 {
    fn from(flag: PropertyFlag) -> Self {
        flag.value()
    }
}

impl From<PropertyFlag> for bool {
    fn from(flag: PropertyFlag) -> Self {
        flag.as_bool()
    }
}

impl From<bool> for PropertyFlag {
    fn from(value: bool) -> Self {
        if value {
            PropertyFlag::Present
        } else {
            PropertyFlag::Absent
        }
    }
}

/// Map lighting types for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum MapLighting {
    /// Dark map (interior, dungeon)
    Dark = 0,
    /// Light map (exterior, daytime)
    Light = 1,
}

impl MapLighting {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(MapLighting::Dark),
            1 => Some(MapLighting::Light),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for MapLighting {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        MapLighting::from_i32(value).ok_or("Invalid map lighting value")
    }
}

impl From<MapLighting> for i32 {
    fn from(lighting: MapLighting) -> Self {
        lighting.value()
    }
}

/// Store types for inventory management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum StoreType {
    /// General goods store
    General = 0,
    /// Weapons and armor shop
    Weapons = 1,
    /// Armor and protective gear shop
    Armor = 2,
    /// Potions and consumables shop
    Potions = 3,
    /// Magic items and spell scrolls shop
    Magic = 4,
    /// Specialty or quest-related shop
    Specialty = 5,
}

impl StoreType {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(StoreType::General),
            1 => Some(StoreType::Weapons),
            2 => Some(StoreType::Armor),
            3 => Some(StoreType::Potions),
            4 => Some(StoreType::Magic),
            5 => Some(StoreType::Specialty),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for StoreType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        StoreType::from_i32(value).ok_or("Invalid store type value")
    }
}

impl From<StoreType> for i32 {
    fn from(store_type: StoreType) -> Self {
        store_type.value()
    }
}

/// Extra object types for interactive objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ExtraType {
    /// No extra object
    None = 0,
    /// Treasure chest
    Chest = 1,
    /// Sign or notice board
    Sign = 2,
    /// Heraldic shield
    Shield = 3,
    /// Decorative object
    Decoration = 4,
    /// Interactive object
    Interactive = 5,
    /// Storage container
    Container = 6,
}

/// Dialog types for conversation systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum DialogType {
    /// Normal dialog line
    Normal = 0,
    /// Choice dialog (branching conversation)
    Choice = 1,
}

impl DialogType {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(DialogType::Normal),
            1 => Some(DialogType::Choice),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for DialogType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        DialogType::from_i32(value).ok_or("Invalid dialog type value")
    }
}

impl From<DialogType> for i32 {
    fn from(dialog_type: DialogType) -> Self {
        dialog_type.value()
    }
}

/// Dialog owner/speaker types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum DialogOwner {
    /// Main character speaking
    Player = 0,
    /// NPC speaking
    Npc = 1,
}

impl DialogOwner {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(DialogOwner::Player),
            1 => Some(DialogOwner::Npc),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for DialogOwner {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        DialogOwner::from_i32(value).ok_or("Invalid dialog owner value")
    }
}

impl From<DialogOwner> for i32 {
    fn from(dialog_owner: DialogOwner) -> Self {
        dialog_owner.value()
    }
}

/// Edit item modification flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EditItemModification {
    /// Item does not modify other items
    DoesNotModify = 0,
    /// Item can modify other items
    CanModify = 1,
}

impl EditItemModification {
    /// Convert from u8 with validation
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(EditItemModification::DoesNotModify),
            1 => Some(EditItemModification::CanModify),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for EditItemModification {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        EditItemModification::from_u8(value).ok_or("Invalid edit item modification value")
    }
}

impl From<EditItemModification> for u8 {
    fn from(modification: EditItemModification) -> Self {
        modification.value()
    }
}

impl From<EditItemModification> for bool {
    fn from(modification: EditItemModification) -> Self {
        modification == EditItemModification::CanModify
    }
}

impl From<bool> for EditItemModification {
    fn from(value: bool) -> Self {
        if value {
            EditItemModification::CanModify
        } else {
            EditItemModification::DoesNotModify
        }
    }
}

/// Edit item additional effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum EditItemEffect {
    /// No additional effect
    None = 0,
    /// Fire effect
    Fire = 1,
    /// Mana drain effect
    ManaDrain = 2,
}

impl EditItemEffect {
    /// Convert from i16 with validation
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            0 => Some(EditItemEffect::None),
            1 => Some(EditItemEffect::Fire),
            2 => Some(EditItemEffect::ManaDrain),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i16 {
        *self as i16
    }
}

impl TryFrom<i16> for EditItemEffect {
    type Error = &'static str;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        EditItemEffect::from_i16(value).ok_or("Invalid edit item effect value")
    }
}

impl From<EditItemEffect> for i16 {
    fn from(effect: EditItemEffect) -> Self {
        effect.value()
    }
}

/// Extra object types for interactive map objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ExtraObjectType {
    /// Chest object
    Chest = 0,
    /// Door object
    Door = 2,
    /// Sign object
    Sign = 4,
    /// Altar object
    Altar = 5,
    /// Interactive object
    Interactive = 6,
    /// Magic object
    Magic = 7,
    /// Unknown/Other object type
    Unknown = 255,
}

impl ExtraObjectType {
    /// Convert from u8 with validation
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ExtraObjectType::Chest),
            2 => Some(ExtraObjectType::Door),
            4 => Some(ExtraObjectType::Sign),
            5 => Some(ExtraObjectType::Altar),
            6 => Some(ExtraObjectType::Interactive),
            7 => Some(ExtraObjectType::Magic),
            _ => Some(ExtraObjectType::Unknown),
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for ExtraObjectType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        ExtraObjectType::from_u8(value).ok_or("Invalid extra object type value")
    }
}

impl From<ExtraObjectType> for u8 {
    fn from(object_type: ExtraObjectType) -> Self {
        object_type.value()
    }
}

/// Visibility types for map objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum VisibilityType {
    Visible0 = 0,
    Visible10 = 10,
    /// Unknown visibility type
    Unknown = 255,
}

impl VisibilityType {
    /// Convert from u8 with validation
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(VisibilityType::Visible0),
            10 => Some(VisibilityType::Visible10),
            _ => Some(VisibilityType::Unknown),
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for VisibilityType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        VisibilityType::from_u8(value).ok_or("Invalid visibility type value")
    }
}

impl From<VisibilityType> for u8 {
    fn from(visibility: VisibilityType) -> Self {
        visibility.value()
    }
}

/// Item type identifiers for inventory and requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ItemTypeId {
    /// Weapon item type
    Weapon = 0,
    /// Healing item type
    Healing = 1,
    /// Edit item type
    Edit = 2,
    /// Miscellaneous item type
    Misc = 3,
    /// Event item type
    Event = 4,
    /// Other/Unknown item type (catch-all for undefined values)
    Other = 255,
}

impl ItemTypeId {
    /// Convert from u8 with validation
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ItemTypeId::Weapon),
            1 => Some(ItemTypeId::Healing),
            2 => Some(ItemTypeId::Edit),
            3 => Some(ItemTypeId::Misc),
            4 => Some(ItemTypeId::Event),
            _ => Some(ItemTypeId::Other),
        }
    }

    /// Convert from a string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Weapon" => Some(ItemTypeId::Weapon),
            "Healing" => Some(ItemTypeId::Healing),
            "Edit" => Some(ItemTypeId::Edit),
            "Misc" => Some(ItemTypeId::Misc),
            "Event" => Some(ItemTypeId::Event),
            "Other" => Some(ItemTypeId::Other),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

impl std::fmt::Display for ItemTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemTypeId::Weapon => write!(f, "Weapon"),
            ItemTypeId::Healing => write!(f, "Healing"),
            ItemTypeId::Edit => write!(f, "Edit"),
            ItemTypeId::Misc => write!(f, "Misc"),
            ItemTypeId::Event => write!(f, "Event"),
            ItemTypeId::Other => write!(f, "Other"),
        }
    }
}

impl TryFrom<u8> for ItemTypeId {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        ItemTypeId::from_u8(value).ok_or("Invalid item type ID value")
    }
}

impl From<ItemTypeId> for u8 {
    fn from(item_type: ItemTypeId) -> Self {
        item_type.value()
    }
}

/// Healing item flags for restoration effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HealItemFlag {
    /// No effect
    None = 0,
    /// Full restoration effect
    FullRestoration = 1,
}

impl std::fmt::Display for HealItemFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealItemFlag::None => write!(f, "None"),
            HealItemFlag::FullRestoration => write!(f, "Full Restoration"),
        }
    }
}

impl HealItemFlag {
    /// Convert from u8 with validation
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(HealItemFlag::None),
            1 => Some(HealItemFlag::FullRestoration),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for HealItemFlag {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        HealItemFlag::from_u8(value).ok_or("Invalid heal item flag value")
    }
}

impl From<HealItemFlag> for u8 {
    fn from(flag: HealItemFlag) -> Self {
        flag.value()
    }
}

impl From<HealItemFlag> for bool {
    fn from(flag: HealItemFlag) -> Self {
        flag == HealItemFlag::FullRestoration
    }
}

impl From<bool> for HealItemFlag {
    fn from(value: bool) -> Self {
        if value {
            HealItemFlag::FullRestoration
        } else {
            HealItemFlag::None
        }
    }
}

/// Magic school types for spell classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MagicSchool {
    /// Unknown or unclassified magic school
    Unknown = 0,
    /// School 1 (specific type unknown)
    School1 = 1,
    /// School 2 (specific type unknown)
    School2 = 2,
    /// School 3 (specific type unknown)
    School3 = 3,
    /// School 4 (specific type unknown)
    School4 = 4,
    /// School 5 (specific type unknown)
    School5 = 5,
    /// School 6 (specific type unknown)
    School6 = 6,
}

impl MagicSchool {
    /// Convert from u32 with validation
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(MagicSchool::Unknown),
            1 => Some(MagicSchool::School1),
            2 => Some(MagicSchool::School2),
            3 => Some(MagicSchool::School3),
            4 => Some(MagicSchool::School4),
            5 => Some(MagicSchool::School5),
            6 => Some(MagicSchool::School6),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

impl TryFrom<u32> for MagicSchool {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        MagicSchool::from_u32(value).ok_or("Invalid magic school value")
    }
}

impl From<MagicSchool> for u32 {
    fn from(school: MagicSchool) -> Self {
        school.value()
    }
}

/// Spell target types for magic effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum SpellTargetType {
    /// Single target spell
    Single = 1,
    /// Self-targeted spell
    SelfTarget = 2,
    /// Area of effect spell
    AreaOfEffect = 3,
    /// Multi-target spell
    MultiTarget = 4,
}

impl SpellTargetType {
    /// Convert from u32 with validation
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(SpellTargetType::Single),
            2 => Some(SpellTargetType::SelfTarget),
            3 => Some(SpellTargetType::AreaOfEffect),
            4 => Some(SpellTargetType::MultiTarget),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

impl TryFrom<u32> for SpellTargetType {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        SpellTargetType::from_u32(value).ok_or("Invalid spell target type value")
    }
}

impl From<SpellTargetType> for u32 {
    fn from(target_type: SpellTargetType) -> Self {
        target_type.value()
    }
}

/// Magic spell boolean flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MagicSpellFlag {
    /// Flag is disabled/false
    Disabled = 0,
    /// Flag is enabled/true
    Enabled = 1,
}

impl MagicSpellFlag {
    /// Convert from u32 with validation
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(MagicSpellFlag::Disabled),
            1 => Some(MagicSpellFlag::Enabled),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

impl TryFrom<u32> for MagicSpellFlag {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        MagicSpellFlag::from_u32(value).ok_or("Invalid magic spell flag value")
    }
}

impl From<MagicSpellFlag> for u32 {
    fn from(flag: MagicSpellFlag) -> Self {
        flag.value()
    }
}

impl From<MagicSpellFlag> for bool {
    fn from(flag: MagicSpellFlag) -> Self {
        flag == MagicSpellFlag::Enabled
    }
}

impl From<bool> for MagicSpellFlag {
    fn from(value: bool) -> Self {
        if value {
            MagicSpellFlag::Enabled
        } else {
            MagicSpellFlag::Disabled
        }
    }
}

/// Magic spell constant values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MagicSpellConstant {
    /// Invalid/unknown constant
    Invalid = 0,
    /// Standard constant value
    Standard = 1,
}

impl MagicSpellConstant {
    /// Convert from u32 with validation
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(MagicSpellConstant::Standard),
            _ => Some(MagicSpellConstant::Invalid),
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

impl TryFrom<u32> for MagicSpellConstant {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        MagicSpellConstant::from_u32(value).ok_or("Invalid magic spell constant value")
    }
}

impl From<MagicSpellConstant> for u32 {
    fn from(constant: MagicSpellConstant) -> Self {
        constant.value()
    }
}

/// NPC looking direction (compass orientation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum NpcLookingDirection {
    /// Facing up (north)
    Up = 0,
    /// Facing up-right (northeast)
    UpRight = 1,
    /// Facing right (east)
    Right = 2,
    /// Facing down-right (southeast)
    DownRight = 3,
    /// Facing down (south)
    Down = 4,
    /// Facing down-left (southwest)
    DownLeft = 5,
    /// Facing left (west)
    Left = 6,
    /// Facing up-left (northwest)
    UpLeft = 7,
}

impl NpcLookingDirection {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(NpcLookingDirection::Up),
            1 => Some(NpcLookingDirection::UpRight),
            2 => Some(NpcLookingDirection::Right),
            3 => Some(NpcLookingDirection::DownRight),
            4 => Some(NpcLookingDirection::Down),
            5 => Some(NpcLookingDirection::DownLeft),
            6 => Some(NpcLookingDirection::Left),
            7 => Some(NpcLookingDirection::UpLeft),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for NpcLookingDirection {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        NpcLookingDirection::from_i32(value).ok_or("Invalid NPC looking direction value")
    }
}

impl From<NpcLookingDirection> for i32 {
    fn from(direction: NpcLookingDirection) -> Self {
        direction.value()
    }
}

/// Party member root map identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum PartyRootMapId {
    /// Unknown/invalid map
    Unknown = 0,
    /// Map 1
    Map1 = 1,
    /// Map 2
    Map2 = 2,
}

impl PartyRootMapId {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(PartyRootMapId::Unknown),
            1 => Some(PartyRootMapId::Map1),
            2 => Some(PartyRootMapId::Map2),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for PartyRootMapId {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        PartyRootMapId::from_i32(value).ok_or("Invalid party root map ID value")
    }
}

impl From<PartyRootMapId> for i32 {
    fn from(map_id: PartyRootMapId) -> Self {
        map_id.value()
    }
}

/// Ghost face identifiers for party members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum GhostFaceId {
    /// No ghost face/unknown
    None = 0,
}

impl GhostFaceId {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(GhostFaceId::None),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for GhostFaceId {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        GhostFaceId::from_i32(value).ok_or("Invalid ghost face ID value")
    }
}

impl From<GhostFaceId> for i32 {
    fn from(face_id: GhostFaceId) -> Self {
        face_id.value()
    }
}

/// Product types for store inventory items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ProductType {
    /// Weapon or armor
    Weapon = 1,
    /// Healing item (potions, etc.)
    Healing = 2,
    /// Edit item (modifiable item)
    EditItem = 3,
    /// Miscellaneous item
    MiscItem = 4,
}

impl ProductType {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(ProductType::Weapon),
            2 => Some(ProductType::Healing),
            3 => Some(ProductType::EditItem),
            4 => Some(ProductType::MiscItem),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for ProductType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        ProductType::from_i32(value).ok_or("Invalid product type value")
    }
}

impl From<ProductType> for i32 {
    fn from(product_type: ProductType) -> Self {
        product_type.value()
    }
}

impl ExtraType {
    /// Convert from i32 with validation
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ExtraType::None),
            1 => Some(ExtraType::Chest),
            2 => Some(ExtraType::Sign),
            3 => Some(ExtraType::Shield),
            4 => Some(ExtraType::Decoration),
            5 => Some(ExtraType::Interactive),
            6 => Some(ExtraType::Container),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for ExtraType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        ExtraType::from_i32(value).ok_or("Invalid extra type value")
    }
}

impl From<ExtraType> for i32 {
    fn from(extra_type: ExtraType) -> Self {
        extra_type.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_conversion() {
        assert_eq!(EventType::from_i32(0), Some(EventType::Unknown));
        assert_eq!(EventType::from_i32(2), Some(EventType::Conditional));
        assert_eq!(
            EventType::from_i32(5),
            Some(EventType::ContinueOnUnsatisfied)
        );
        assert_eq!(EventType::from_i32(6), Some(EventType::ExecuteOnSatisfied));
        assert_eq!(EventType::from_i32(99), None);

        assert_eq!(i32::from(EventType::Conditional), 2);
        assert_eq!(EventType::Conditional.value(), 2);
    }

    #[test]
    fn test_monster_ai_type_conversion() {
        assert_eq!(MonsterAiType::from_i32(1), Some(MonsterAiType::Aggressive));
        assert_eq!(MonsterAiType::from_i32(4), Some(MonsterAiType::Boss));
        assert_eq!(MonsterAiType::from_i32(7), None);

        assert_eq!(i32::from(MonsterAiType::Ranged), 3);
    }

    #[test]
    fn test_property_flag_conversion() {
        assert_eq!(PropertyFlag::from_i32(0), Some(PropertyFlag::Absent));
        assert_eq!(PropertyFlag::from_i32(1), Some(PropertyFlag::Present));
        assert_eq!(PropertyFlag::from_i32(2), None);

        assert_eq!(bool::from(PropertyFlag::Present), true);
        assert_eq!(bool::from(PropertyFlag::Absent), false);
        assert_eq!(PropertyFlag::from(true), PropertyFlag::Present);
        assert_eq!(PropertyFlag::from(false), PropertyFlag::Absent);
    }

    #[test]
    fn test_try_from_impl() {
        assert!(EventType::try_from(2).is_ok());
        assert!(EventType::try_from(99).is_err());

        assert!(MonsterAiType::try_from(3).is_ok());
        assert!(MonsterAiType::try_from(-1).is_err());
    }

    #[test]
    fn test_product_type_conversion() {
        assert_eq!(ProductType::from_i32(1), Some(ProductType::Weapon));
        assert_eq!(ProductType::from_i32(2), Some(ProductType::Healing));
        assert_eq!(ProductType::from_i32(3), Some(ProductType::EditItem));
        assert_eq!(ProductType::from_i32(4), Some(ProductType::MiscItem));
        assert_eq!(ProductType::from_i32(99), None);

        assert_eq!(i32::from(ProductType::Healing), 2);
        assert_eq!(ProductType::Healing.value(), 2);
    }

    #[test]
    fn test_dialog_type_conversion() {
        assert_eq!(DialogType::from_i32(0), Some(DialogType::Normal));
        assert_eq!(DialogType::from_i32(1), Some(DialogType::Choice));
        assert_eq!(DialogType::from_i32(99), None);

        assert_eq!(i32::from(DialogType::Choice), 1);
        assert_eq!(DialogType::Choice.value(), 1);
    }

    #[test]
    fn test_dialog_owner_conversion() {
        assert_eq!(DialogOwner::from_i32(0), Some(DialogOwner::Player));
        assert_eq!(DialogOwner::from_i32(1), Some(DialogOwner::Npc));
        assert_eq!(DialogOwner::from_i32(99), None);

        assert_eq!(i32::from(DialogOwner::Npc), 1);
        assert_eq!(DialogOwner::Npc.value(), 1);
    }

    #[test]
    fn test_edit_item_modification_conversion() {
        assert_eq!(
            EditItemModification::from_u8(0),
            Some(EditItemModification::DoesNotModify)
        );
        assert_eq!(
            EditItemModification::from_u8(1),
            Some(EditItemModification::CanModify)
        );
        assert_eq!(EditItemModification::from_u8(99), None);

        assert_eq!(u8::from(EditItemModification::CanModify), 1);
        assert_eq!(EditItemModification::CanModify.value(), 1);
        assert_eq!(bool::from(EditItemModification::CanModify), true);
        assert_eq!(
            EditItemModification::from(true),
            EditItemModification::CanModify
        );
    }

    #[test]
    fn test_edit_item_effect_conversion() {
        assert_eq!(EditItemEffect::from_i16(0), Some(EditItemEffect::None));
        assert_eq!(EditItemEffect::from_i16(1), Some(EditItemEffect::Fire));
        assert_eq!(EditItemEffect::from_i16(2), Some(EditItemEffect::ManaDrain));
        assert_eq!(EditItemEffect::from_i16(99), None);

        assert_eq!(i16::from(EditItemEffect::ManaDrain), 2);
        assert_eq!(EditItemEffect::ManaDrain.value(), 2);
    }

    #[test]
    fn test_extra_object_type_conversion() {
        assert_eq!(ExtraObjectType::from_u8(0), Some(ExtraObjectType::Chest));
        assert_eq!(ExtraObjectType::from_u8(2), Some(ExtraObjectType::Door));
        assert_eq!(ExtraObjectType::from_u8(4), Some(ExtraObjectType::Sign));
        assert_eq!(ExtraObjectType::from_u8(5), Some(ExtraObjectType::Altar));
        assert_eq!(
            ExtraObjectType::from_u8(6),
            Some(ExtraObjectType::Interactive)
        );
        assert_eq!(ExtraObjectType::from_u8(7), Some(ExtraObjectType::Magic));
        assert_eq!(ExtraObjectType::from_u8(99), Some(ExtraObjectType::Unknown));

        assert_eq!(u8::from(ExtraObjectType::Magic), 7);
        assert_eq!(ExtraObjectType::Magic.value(), 7);
    }

    #[test]
    fn test_visibility_type_conversion() {
        assert_eq!(VisibilityType::from_u8(0), Some(VisibilityType::Visible0));
        assert_eq!(VisibilityType::from_u8(10), Some(VisibilityType::Visible10));
        assert_eq!(VisibilityType::from_u8(99), Some(VisibilityType::Unknown));

        assert_eq!(u8::from(VisibilityType::Visible10), 10);
        assert_eq!(VisibilityType::Visible10.value(), 10);
    }

    #[test]
    fn test_item_type_id_conversion() {
        assert_eq!(ItemTypeId::from_u8(0), Some(ItemTypeId::Weapon));
        assert_eq!(ItemTypeId::from_u8(1), Some(ItemTypeId::Healing));
        assert_eq!(ItemTypeId::from_u8(2), Some(ItemTypeId::Edit));
        assert_eq!(ItemTypeId::from_u8(3), Some(ItemTypeId::Misc));
        assert_eq!(ItemTypeId::from_u8(4), Some(ItemTypeId::Event));
        assert_eq!(ItemTypeId::from_u8(99), Some(ItemTypeId::Other));

        assert_eq!(u8::from(ItemTypeId::Edit), 2);
        assert_eq!(ItemTypeId::Edit.value(), 2);
    }

    #[test]
    fn test_heal_item_flag_conversion() {
        assert_eq!(HealItemFlag::from_u8(0), Some(HealItemFlag::None));
        assert_eq!(
            HealItemFlag::from_u8(1),
            Some(HealItemFlag::FullRestoration)
        );
        assert_eq!(HealItemFlag::from_u8(99), None);

        assert_eq!(u8::from(HealItemFlag::FullRestoration), 1);
        assert_eq!(HealItemFlag::FullRestoration.value(), 1);
        assert_eq!(bool::from(HealItemFlag::FullRestoration), true);
        assert_eq!(HealItemFlag::from(true), HealItemFlag::FullRestoration);
    }

    #[test]
    fn test_magic_school_conversion() {
        assert_eq!(MagicSchool::from_u32(0), Some(MagicSchool::Unknown));
        assert_eq!(MagicSchool::from_u32(1), Some(MagicSchool::School1));
        assert_eq!(MagicSchool::from_u32(2), Some(MagicSchool::School2));
        assert_eq!(MagicSchool::from_u32(3), Some(MagicSchool::School3));
        assert_eq!(MagicSchool::from_u32(4), Some(MagicSchool::School4));
        assert_eq!(MagicSchool::from_u32(5), Some(MagicSchool::School5));
        assert_eq!(MagicSchool::from_u32(6), Some(MagicSchool::School6));
        assert_eq!(MagicSchool::from_u32(99), None);

        assert_eq!(u32::from(MagicSchool::School6), 6);
        assert_eq!(MagicSchool::School6.value(), 6);
    }

    #[test]
    fn test_spell_target_type_conversion() {
        assert_eq!(SpellTargetType::from_u32(1), Some(SpellTargetType::Single));
        assert_eq!(
            SpellTargetType::from_u32(2),
            Some(SpellTargetType::SelfTarget)
        );
        assert_eq!(
            SpellTargetType::from_u32(3),
            Some(SpellTargetType::AreaOfEffect)
        );
        assert_eq!(
            SpellTargetType::from_u32(4),
            Some(SpellTargetType::MultiTarget)
        );
        assert_eq!(SpellTargetType::from_u32(99), None);

        assert_eq!(u32::from(SpellTargetType::AreaOfEffect), 3);
        assert_eq!(SpellTargetType::AreaOfEffect.value(), 3);
    }

    #[test]
    fn test_magic_spell_flag_conversion() {
        assert_eq!(MagicSpellFlag::from_u32(0), Some(MagicSpellFlag::Disabled));
        assert_eq!(MagicSpellFlag::from_u32(1), Some(MagicSpellFlag::Enabled));
        assert_eq!(MagicSpellFlag::from_u32(99), None);

        assert_eq!(u32::from(MagicSpellFlag::Enabled), 1);
        assert_eq!(MagicSpellFlag::Enabled.value(), 1);
        assert_eq!(bool::from(MagicSpellFlag::Enabled), true);
        assert_eq!(MagicSpellFlag::from(true), MagicSpellFlag::Enabled);
    }

    #[test]
    fn test_magic_spell_constant_conversion() {
        assert_eq!(
            MagicSpellConstant::from_u32(1),
            Some(MagicSpellConstant::Standard)
        );
        assert_eq!(
            MagicSpellConstant::from_u32(0),
            Some(MagicSpellConstant::Invalid)
        );
        assert_eq!(
            MagicSpellConstant::from_u32(99),
            Some(MagicSpellConstant::Invalid)
        );

        assert_eq!(u32::from(MagicSpellConstant::Standard), 1);
        assert_eq!(MagicSpellConstant::Standard.value(), 1);
    }

    #[test]
    fn test_npc_looking_direction_conversion() {
        assert_eq!(
            NpcLookingDirection::from_i32(0),
            Some(NpcLookingDirection::Up)
        );
        assert_eq!(
            NpcLookingDirection::from_i32(1),
            Some(NpcLookingDirection::UpRight)
        );
        assert_eq!(
            NpcLookingDirection::from_i32(2),
            Some(NpcLookingDirection::Right)
        );
        assert_eq!(
            NpcLookingDirection::from_i32(3),
            Some(NpcLookingDirection::DownRight)
        );
        assert_eq!(
            NpcLookingDirection::from_i32(4),
            Some(NpcLookingDirection::Down)
        );
        assert_eq!(
            NpcLookingDirection::from_i32(5),
            Some(NpcLookingDirection::DownLeft)
        );
        assert_eq!(
            NpcLookingDirection::from_i32(6),
            Some(NpcLookingDirection::Left)
        );
        assert_eq!(
            NpcLookingDirection::from_i32(7),
            Some(NpcLookingDirection::UpLeft)
        );
        assert_eq!(NpcLookingDirection::from_i32(99), None);

        assert_eq!(i32::from(NpcLookingDirection::UpLeft), 7);
        assert_eq!(NpcLookingDirection::UpLeft.value(), 7);
    }

    #[test]
    fn test_ghost_face_id_conversion() {
        assert_eq!(GhostFaceId::from_i32(0), Some(GhostFaceId::None));
        assert_eq!(GhostFaceId::from_i32(99), None);

        assert_eq!(i32::from(GhostFaceId::None), 0);
        assert_eq!(GhostFaceId::None.value(), 0);
    }
}
