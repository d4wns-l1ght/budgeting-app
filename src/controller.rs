use anyhow::Result;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{model::Model, view::View};

// TODO: mappable keybinds or if the controller needs multi-step commands, or mode switching
// (like vim bindings)
#[derive(Debug, Default)]
pub struct Controller {}

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
		let shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);
		match key_event.code {
			KeyCode::Char('q') => model.exit = true,
			KeyCode::Char('H') | KeyCode::Left if shift_pressed => view.previous_sheet(model),
			KeyCode::Char('L') | KeyCode::Right if shift_pressed => view.next_sheet(model),
			KeyCode::Char('h') | KeyCode::Left => view.previous_column(model),
			KeyCode::Char('j') | KeyCode::Down => view.next_row(model),
			KeyCode::Char('k') | KeyCode::Up => view.previous_row(model),
			KeyCode::Char('l') | KeyCode::Right => view.next_column(model),
			_ => {}
		}
	}
}
