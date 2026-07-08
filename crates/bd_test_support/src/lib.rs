//! bd_test_support — Shared test utilities for the BD Kernel.
//!
//! Provides deterministic RNG, minimal app builders, and snapshot helpers.

use bevy_app::App;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Create a deterministic RNG from a fixed seed for reproducible tests.
pub fn seeded_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

/// Build a minimal Bevy app with just the core plugin for unit testing.
pub fn minimal_app() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdCorePlugin);
    app
}
