//! Application services, ported from `Trainer/Services/*.cs`.
//!
//! Every service works over the [`Storage`](crate::storage::Storage) trait
//! rather than a concrete store, so the whole layer runs in the fast native
//! tier against `MemStorage` — the same role Moq played in the C# tests.

pub mod active_activity;
pub mod active_time;
pub mod activity;
pub mod activity_type;
pub mod export_import;
pub mod goal;
pub mod known_location;
pub mod week_fill;
