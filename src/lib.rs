pub mod recovery;

#[path = "core/escape.rs"]
pub mod escape;

#[path = "core/gamelog.rs"]
pub mod gamelog;

#[path = "core/state.rs"]
pub mod core_state;

#[path = "core/resources.rs"]
pub mod core_resources;

#[path = "core/save_runtime.rs"]
pub mod core_save;

#[path = "core/turn_runtime.rs"]
pub mod core_turn;

#[path = "ui/modal_priority.rs"]
pub mod modal_priority;

#[path = "ui/objective_prompt.rs"]
pub mod objective_prompt;

#[path = "ui/help_panel.rs"]
pub mod help_panel;

#[path = "ui/overworld_panel.rs"]
pub mod overworld_panel;

#[path = "ui/readability.rs"]
pub mod readability;

#[path = "ui/menu.rs"]
pub mod menu;

pub mod runtime_flow;

pub mod runtime_app;

#[path = "ui/primary_cta.rs"]
pub mod primary_cta;

#[path = "ui/copy_catalog.rs"]
pub mod copy_catalog;

#[path = "core/save_recap.rs"]
pub mod save_recap;

#[path = "core/qa_profile.rs"]
pub mod qa_profile;

#[path = "game/overworld/weather.rs"]
pub mod overworld_weather;

pub mod alpha_battery;

pub mod alpha_signoff;

pub mod core {
	pub use crate::core_state as state;
	pub use crate::core_resources as resources;
	pub use crate::core_save as save;
	pub use crate::core_turn as turn;
	pub use crate::escape;
	pub use crate::gamelog;
	pub use crate::qa_profile;
	pub use crate::save_recap;
}

pub mod game {
	pub mod overworld {
		pub use crate::overworld_weather as weather;
	}
}

pub mod ui {
	pub use crate::copy_catalog;
	pub use crate::help_panel;
	pub use crate::menu;
	pub use crate::modal_priority;
	pub use crate::objective_prompt;
	pub use crate::overworld_panel;
	pub use crate::primary_cta;
	pub use crate::readability;
}
