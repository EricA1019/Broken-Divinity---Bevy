pub mod recovery;

pub mod core;
pub mod game;
pub mod ui;

#[path = "core/escape.rs"]
pub mod escape;

#[path = "ui/primary_cta.rs"]
pub mod primary_cta;

#[path = "ui/copy_catalog.rs"]
pub mod copy_catalog;

#[path = "core/save_recap.rs"]
pub mod save_recap;

#[path = "core/qa_profile.rs"]
pub mod qa_profile;

pub mod runtime_app;
pub mod runtime_flow;

pub mod alpha_battery;
pub mod alpha_signoff;

pub use crate::core::gamelog;
pub use crate::core::resources as core_resources;
pub use crate::core::save as core_save;
pub use crate::core::state as core_state;
pub use crate::core::turn as core_turn;
pub use crate::game::overworld::weather as overworld_weather;
pub use crate::ui::help_panel;
pub use crate::ui::menu;
pub use crate::ui::modal_priority;
pub use crate::ui::objective_prompt;
pub use crate::ui::overworld_panel;
pub use crate::ui::readability;

#[cfg(test)]
mod tests;
