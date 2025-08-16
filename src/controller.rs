use anyhow::Result;
use ratatui::crossterm::event::Event;

use crate::{model::Model, view::View};

#[derive(Debug)]
pub enum CursorMode {
	Traverse,
	Normal,
	Insert,
}

impl Default for CursorMode {
	fn default() -> Self {
		Self::Traverse
	}
}

// TODO: mappable keybinds or if the controller needs multi-step commands, or mode switching
// (like vim bindings)
#[derive(Debug, Default)]
pub struct Controller {
	mode: CursorMode,
}

impl Controller {
	pub fn handle_events(&self, event: Event, model: &mut Model, view: &mut View) -> Result<()> {
		todo!()
	}
}
