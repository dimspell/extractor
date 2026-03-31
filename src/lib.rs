//! Dispel Extractor Core Library
//!
//! This library provides parsers and data structures for Dispel game files.
//! It's used by both the CLI extractor and the GUI editor.

pub mod database;
pub mod map;
pub mod references;
pub mod snf;
pub mod sprite;
pub mod commands;

// Re-export key types for easy access
pub use references::{
    edit_item_db::EditItem, enums::*, event_item_db::EventItem, extra_ref::ExtraRef,
    heal_item_db::HealItem, misc_item_db::MiscItem, extractor::Extractor, weapons_db::WeaponItem,
};
