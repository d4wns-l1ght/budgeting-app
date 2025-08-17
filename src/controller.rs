use anyhow::Result;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

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
		match event {
			Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
				self.handle_key_event(key_event, model, view);
			}
			_ => {}
		}
		Ok(())
	}

	fn handle_key_event(&self, key_event: KeyEvent, model: &mut Model, view: &mut View) {
		match key_event.code {
			KeyCode::Char('q') => model.exit = true,
			_ => {}
		}
	}
}
