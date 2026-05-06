//! Concrete [`RecordPatcher`](super::patcher::RecordPatcher) implementations,
//! one per catalog file format.
//!
//! Phase 2 ships a single working example ([`MiscItemPatcher`]); the same
//! pattern extends to every other `Extractor`-derived record type.

mod misc_item;

pub use misc_item::MiscItemPatcher;
