use anyhow::Result;
use ratatui::crossterm::event::{Event, KeyCode as K, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{model::Model, view::View};

// TODO: mappable keybinds or if the controller needs multi-step commands, or mode switching
// (like vim bindings)
#[derive(Debug, Default)]
pub struct Controller {
	pub state: ControllerState,
}

#[derive(Debug, Default)]
pub struct ControllerState {
	pub last_nums: Vec<u32>,
	pub last_chars: Vec<char>,
}

impl Controller {
	pub fn handle_events(
		&mut self,
		event: Event,
		model: &mut Model,
		view: &mut View,
	) -> Result<()> {
		match event {
			Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
				self.handle_key_event(key_event, model, view);
			}
			_ => {}
		}
		Ok(())
	}

	fn handle_key_event(&mut self, key_event: KeyEvent, model: &mut Model, view: &mut View) {
		let shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);
		let ctrl_pressed = key_event.modifiers.contains(KeyModifiers::CONTROL);
		let _alt_pressed = key_event.modifiers.contains(KeyModifiers::ALT);
		match key_event.code {
			// quit
			K::Char('q') => model.exit = true,
			// move between sheets
			K::Char('H') | K::Left if shift_pressed => view.previous_sheet(model),
			K::Char('L') | K::Right if shift_pressed => view.next_sheet(model),
			// move by a count
			K::Char('j') | K::Down if !self.state.last_nums.is_empty() => {
				let amount = self.state.last_nums.iter().fold(0, |acc, d| acc * 10 + d) as usize;
				view.down_by(amount, model);
				self.state.last_nums.clear();
			}
			K::Char('k') | K::Up if !self.state.last_nums.is_empty() => {
				let amount = self.state.last_nums.iter().fold(0, |acc, d| acc * 10 + d) as usize;
				view.up_by(amount, model);
				self.state.last_nums.clear();
			}
			// move normally
			K::Char('h') | K::Left => view.previous_column(model),
			K::Char('j') | K::Down => view.next_row(model),
			K::Char('k') | K::Up => view.previous_row(model),
			K::Char('l') | K::Right => view.next_column(model),
			// jump to start
			K::Char('g') => {
				if self.state.last_chars.last() == Some(&'g') {
					view.first_row(model);
					self.state.last_chars.clear();
				} else {
					self.state.last_chars.push('g');
				}
			}
			// jump to end
			K::Char('G') => view.last_row(model),
			// half up/down
			K::Char('u') if ctrl_pressed => view.half_up(model),
			K::Char('d') if ctrl_pressed => view.half_down(model),
			K::Esc => {
				self.state.last_nums.clear();
				self.state.last_chars.clear();
			}
			K::Char(c) => {
				if let Some(d) = c.to_digit(10)
					&& d <= 9
				{
					self.state.last_nums.push(d);
					self.state.last_chars.clear();
				} else {
					self.state.last_chars.push(c);
					self.state.last_nums.clear();
				}
			}
			_ => {}
		}
	}
}
