//! This module handles input from the user, and directs the model/view appropriately
use ratatui::{
	crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
	widgets::{Block, BorderType, Borders},
};

use crate::{
	controller::{
		keymap::{KeyMap, KeyMapBuilder, Pred},
		popup::Popup,
	},
	model::Model,
	view::View,
};

mod keymap;
pub mod popup;

#[derive(Debug)]
pub struct Controller {
	pub state: ControllerState,
	keymaps: Vec<KeyMap>,
}

#[derive(Debug, Default)]
pub struct ControllerState {
	pub last_nums: Vec<u32>,
	pub last_chars: Vec<char>,
	pub popup: Option<Popup>,
	pub exit: bool,
}

impl ControllerState {
	pub fn handle_char(&mut self, c: char) {
		if let Some(d) = c.to_digit(10)
			&& d <= 9
		{
			self.last_nums.push(d);
			self.last_chars.clear();
		} else {
			self.last_chars.push(c);
			self.last_nums.clear();
		}
	}

	pub fn get_count_amount(&self) -> usize {
		self.last_nums
			.iter()
			.fold(0, |acc: u32, d| acc.saturating_mul(10).saturating_add(*d)) as usize
	}
}

impl Controller {
	pub fn handle_events(&mut self, event: &Event, model: &mut Model, view: &mut View) {
		match event {
			Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
				self.handle_key_event(key_event, model, view);
			}
			_ => {}
		}
	}

	fn handle_key_event(&mut self, key_event: &KeyEvent, model: &mut Model, view: &mut View) {
		if let Some(popup) = self.state.popup.take() {
			self.state.popup = popup.handle_key_event(key_event, model);
			return;
		}
		if let Some(km) = self
			.keymaps
			.iter_mut()
			.find(|km| km.matches(key_event, &self.state))
		{
			(km.action.as_mut())(view, model, &mut self.state);
		} else if let KeyCode::Char(c) = key_event.code {
			self.state.handle_char(c);
		}
	}

	pub fn new() -> Self {
		let shift_pressed: Pred = Pred::new(|ke, _cs| ke.modifiers.contains(KeyModifiers::SHIFT));
		let ctrl_pressed: Pred = Pred::new(|ke, _cs| ke.modifiers.contains(KeyModifiers::CONTROL));
		let _alt_pressed: Pred = Pred::new(|ke, _cs| ke.modifiers.contains(KeyModifiers::ALT));
		let last_nums_empty: Pred = Pred::new(|_ke, cs| cs.last_nums.is_empty());
		let last_chars_empty: Pred = Pred::new(|_ke, cs| cs.last_chars.is_empty());

		// NOTE: Be sure to define predicated keymaps before unpredicated ones, like in a match
		// function. If they are defined out of order, the unpredicated one will always run before
		// the predicated one gets a chance to be evaluated
		Self {
			state: ControllerState::default(),
			keymaps: vec![
				// With predicate
				// next/prev sheets
				KeyMapBuilder::new([KeyCode::Char('H'), KeyCode::Left])
					.when(&shift_pressed)
					.do_action(|view, model, _cs| view.previous_sheet(model)),
				KeyMapBuilder::new([KeyCode::Char('L'), KeyCode::Right])
					.when(&shift_pressed)
					.do_action(|view, model, _cs| view.next_sheet(model)),
				// up/down by count
				KeyMapBuilder::new([KeyCode::Char('j'), KeyCode::Down])
					.when(&last_nums_empty.not())
					.do_action(|view, model, cs| {
						view.down_by(cs.get_count_amount(), model);
						cs.last_nums.clear();
					}),
				KeyMapBuilder::new([KeyCode::Char('k'), KeyCode::Up])
					.when(&last_nums_empty.not())
					.do_action(|view, model, cs| {
						view.up_by(cs.get_count_amount(), model);
						cs.last_nums.clear();
					}),
				KeyMapBuilder::new([KeyCode::Enter])
					.when(&last_nums_empty.not())
					.do_action(|view, model, cs| {
						view.jump_to_row(cs.get_count_amount(), model);
						cs.last_nums.clear();
					}),
				// Make new sheet
				KeyMapBuilder::new([KeyCode::Char('t')])
					.when(&ctrl_pressed)
					.do_action(|_view, model, _cs| {
						model.create_sheet();
					}),
				KeyMapBuilder::new([KeyCode::Char('r')])
					.when(&ctrl_pressed)
					.do_action(|view, model, cs| {
						let sheet_index = view.selected_sheet;
						cs.popup = Some(
							Popup::new(move |text, model| {
								let sheet = model.get_sheet_mut(sheet_index).unwrap_or_else(|| {
									panic!("Couldnt get sheet with index {sheet_index}")
								});
								sheet.name = text;
							})
							.with_initial(&view.get_selected_sheet(model).name)
							.with_block(
								Block::new()
									.borders(Borders::ALL)
									.border_type(BorderType::Rounded)
									.title("Rename sheet"),
							),
						);
					}),
				// scroll up/down
				KeyMapBuilder::new([KeyCode::Char('u')])
					.when(&ctrl_pressed)
					.do_action(|view, model, _cs| view.half_up(model)),
				KeyMapBuilder::new([KeyCode::Char('d')])
					.when(&ctrl_pressed)
					.do_action(|view, model, _cs| view.half_down(model)),
				// jump to top
				KeyMapBuilder::new([KeyCode::Char('g')])
					.when(&Pred::new(|_ke, cs| cs.last_chars.last() == Some(&'g')))
					.do_action(|view, model, cs| {
						cs.last_chars.clear();
						view.first_row(model);
					}),
				KeyMapBuilder::new([KeyCode::Esc])
					.when(&last_nums_empty.not().or(&last_chars_empty.not()))
					.do_action(|_view, _model, cs| {
						cs.last_nums.clear();
						cs.last_chars.clear();
					}),
				// Without Predicate
				KeyMapBuilder::new([KeyCode::Char('q')])
					.do_action(|_view, _model, cs| cs.exit = true),
				KeyMapBuilder::new([KeyCode::Char('h'), KeyCode::Left])
					.do_action(|view, model, _cs| view.previous_column(model)),
				KeyMapBuilder::new([KeyCode::Char('j'), KeyCode::Left])
					.do_action(|view, model, _cs| view.next_row(model)),
				KeyMapBuilder::new([KeyCode::Char('k'), KeyCode::Left])
					.do_action(|view, model, _cs| view.previous_row(model)),
				KeyMapBuilder::new([KeyCode::Char('l'), KeyCode::Left])
					.do_action(|view, model, _cs| view.next_column(model)),
				KeyMapBuilder::new([KeyCode::Char('g')])
					.do_action(|_view, _model, cs| cs.last_chars.push('g')),
				KeyMapBuilder::new([KeyCode::Char('G')])
					.do_action(|view, model, _cs| view.last_row(model)),
			],
		}
	}
}
