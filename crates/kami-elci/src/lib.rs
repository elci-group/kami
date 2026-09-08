//! ELCI sibling adapters. Kami never recreates functionality a sibling
//! ELCI tool already owns; it detects the sibling and either shells out
//! to it or reports `CAPABILITY_UNAVAILABLE` — it must never silently
//! change its own behaviour when a sibling is missing (§24, §29).

pub mod adapter;
pub mod registry;

pub use adapter::{AdapterHealth, AdapterKind, ElciAdapter, INVOKE_UNAVAILABLE};
pub use registry::{component_registry, elci_root, ComponentDescriptor};
