//! bd_data — Content loading and validation for the BD Kernel.
//!
//! Handles RON deserialization, registries, and cross-file validation.
//! No gameplay logic lives here.

pub mod id;
pub mod loader;
pub mod registry;
pub mod validation;

pub use id::ContentId;
pub use registry::{Registry, RegistryError};

/// Placeholder — data loading deferred to Phase 8.
pub struct BdDataPlugin;

impl bevy_app::Plugin for BdDataPlugin {
    fn build(&self, _app: &mut bevy_app::App) {
        tracing::info!("BdDataPlugin initialized (placeholder)");
    }
}
