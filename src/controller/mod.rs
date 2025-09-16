//! This module handles input from the user, and directs the model/view appropriately

use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
	controller::{
		commands::CommandTrie,
		popup::{Popup, PopupBehaviour},
	},
	model::{Model, Transaction},
	view::View,
};

mod commands;
pub mod popup;

#[derive(Default)]
pub struct Controller {
	pub state: ControllerState,
	commands: CommandTrie,
}

#[derive(Default)]
pub struct ControllerState {
	pub last_nums: Vec<u32>,
	pub last_chars: Vec<char>,
	pub popup: Option<Popup>,
	pub exit: bool,
	register: Option<Transaction>,
}

impl ControllerState {
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
		match key_event.code {
			KeyCode::Char(c) => {
				if key_event.modifiers.contains(KeyModifiers::CONTROL) {
					self.handle_modified_char(c, key_event.modifiers);
				} else {
					if let Some(d) = c.to_digit(10)
						&& d < 10
					{
						self.state.last_nums.push(d);
						return;
					}
					self.state.last_chars.push(c);
				}
			}
			KeyCode::Backspace | KeyCode::Esc => self.reset_command(),
			_ => {
				self.handle_special_key(key_event);
			}
		}
		self.try_action(model, view);
	}

	fn try_action(&mut self, model: &mut Model, view: &mut View) {
		if let Some(command) = self
			.commands
			.traverse(self.state.last_chars.iter().copied())
			&& !command.has_children()
			&& command.has_action()
		{
			{
				(command
					.action()
					.expect("We have checked that the command has an action"))(
					view, model, &mut self.state
				);
				self.reset_command();
			}
		} else {
			self.state.last_nums.clear();
		}
	}

	fn handle_modified_char(&mut self, char: char, modifiers: KeyModifiers) {
		self.state.last_chars.push('<');
		if modifiers.contains(KeyModifiers::CONTROL) {
			self.state.last_chars.push('C');
			self.state.last_chars.push('-');
		}
		// I don't think this is necessary to check for? e.g. <S-H> can also just be, H
		// And <C-S-_> isn't possible without messing around with some other stuff
		// if modifiers.contains(KeyModifiers::SHIFT) {
		// 	self.state.last_chars.push('S');
		// 	self.state.last_chars.push('-');
		// }

		self.state.last_chars.push(char);
		self.state.last_chars.push('>');
	}

	fn handle_special_key(&mut self, key_event: &KeyEvent) {
		match (key_event.modifiers, key_event.code) {
			(KeyModifiers::CONTROL, KeyCode::Up) => {
				self.handle_modified_char('k', KeyModifiers::CONTROL);
			}
			(KeyModifiers::CONTROL, KeyCode::Down) => {
				self.handle_modified_char('j', KeyModifiers::CONTROL);
			}
			(KeyModifiers::CONTROL, KeyCode::Left) => {
				self.handle_modified_char('h', KeyModifiers::CONTROL);
			}
			(KeyModifiers::CONTROL, KeyCode::Right) => {
				self.handle_modified_char('l', KeyModifiers::CONTROL);
			}
			(KeyModifiers::CONTROL, KeyCode::Delete) => {
				self.state.last_chars.push('<');
				self.state.last_chars.push('C');
				self.state.last_chars.push('-');
				self.state.last_chars.push('D');
				self.state.last_chars.push('e');
				self.state.last_chars.push('l');
				self.state.last_chars.push('>');
			}
			(KeyModifiers::SHIFT, KeyCode::Up) => {
				self.state.last_chars.push('K');
			}
			(KeyModifiers::SHIFT, KeyCode::Down) => {
				self.state.last_chars.push('J');
			}
			(KeyModifiers::SHIFT, KeyCode::Left) => {
				self.state.last_chars.push('H');
			}
			(KeyModifiers::SHIFT, KeyCode::Right) => {
				self.state.last_chars.push('L');
			}
			(_, KeyCode::Up) => self.state.last_chars.push('k'),

			(_, KeyCode::Down) => self.state.last_chars.push('j'),

			(_, KeyCode::Left) => self.state.last_chars.push('h'),

			(_, KeyCode::Right) => self.state.last_chars.push('l'),
			(_, KeyCode::PageUp) => self.handle_modified_char('u', KeyModifiers::CONTROL),
			(_, KeyCode::PageDown) => self.handle_modified_char('d', KeyModifiers::CONTROL),
			(_, KeyCode::Home) => {
				self.state.last_chars.push('g');
				self.state.last_chars.push('g');
			}
			(_, KeyCode::End) => self.state.last_chars.push('G'),

			_ => {}
		}
	}

	fn reset_command(&mut self) {
		self.state.last_chars.clear();
		self.state.last_nums.clear();
	}

	pub fn new() -> Self {
		let trie = CommandTrie::default()
			.add("q", |_view, _model, cs| cs.exit = true)
			.add("<C-c>", |_view, _model, cs| cs.exit = true)
			.add("j", |view, model, cs| {
				if cs.last_nums.is_empty() {
					view.next_row(model);
					return;
				}
				view.down_by(cs.get_count_amount(), model);
			})
			.add("k", |view, model, cs| {
				if cs.last_nums.is_empty() {
					view.previous_row(model);
					return;
				}
				view.up_by(cs.get_count_amount(), model);
			})
			.add("h", |view, model, _cs| view.previous_column(model))
			.add("l", |view, model, _cs| view.next_column(model))
			.add("i", popup::defaults::insert_action)
			.add("gg", |view, model, _cs| view.first_row(model))
			.add("G", |view, model, _cs| view.last_row(model))
			.add("H", |view, model, _cs| view.previous_sheet(model))
			.add("L", |view, model, _cs| view.next_sheet(model))
			.add("J", |view, model, _cs| {
				let sheet_index = view.selected_sheet;
				let sheet = view.get_selected_sheet(model);
				if let Some(row) = view.get_selected_row(sheet) {
					model.move_transaction_down(sheet_index, row);
					view.next_row(model);
				}
			})
			.add("K", |view, model, _cs| {
				let sheet_index = view.selected_sheet;
				let sheet = view.get_selected_sheet(model);
				if let Some(row) = view.get_selected_row(sheet) {
					model.move_transaction_up(sheet_index, row);
					view.previous_row(model);
				}
			})
			.add("y", |view, model, cs| {
				let sheet_index = view.selected_sheet;
				let sheet = view.get_selected_sheet(model);
				if let Some(row) = view.get_selected_row(sheet) {
					cs.register = Some(model.copy_row(sheet_index, row));
				}
			})
			.add("d", |view, model, cs| {
				let sheet_index = view.selected_sheet;
				let sheet = view.get_selected_sheet(model);
				if let Some(row) = view.get_selected_row(sheet) {
					cs.register = Some(model.delete_row(sheet_index, row));
				}
			})
			.add("p", |view, model, cs| {
				let sheet_index = view.selected_sheet;
				let sheet = view.get_selected_sheet(model);
				if let Some(row) = view.get_selected_row(sheet)
					&& let Some(transaction) = cs.register.clone()
				{
					model.insert_row(sheet_index, row + 1, transaction);
					view.next_row(model);
				}
			})
			.add("P", |view, model, cs| {
				let sheet_index = view.selected_sheet;
				let sheet = view.get_selected_sheet(model);
				if let Some(row) = view.get_selected_row(sheet)
					&& let Some(transaction) = cs.register.clone()
				{
					model.insert_row(sheet_index, row, transaction);
				}
			})
			.add("o", popup::defaults::new_row_below)
			.add("O", popup::defaults::new_row_above)
			.add("<C-d>", |view, model, _cs| view.half_down(model))
			.add("<C-u>", |view, model, _cs| view.half_up(model))
			.add("<C-t>", |_view, model, _cs| model.create_sheet())
			.add("<C-r>", popup::defaults::rename_sheet)
			.add("<C-Del>", popup::defaults::delete_sheet)
			.add("?", popup::defaults::help);
		Self {
			commands: trie,
			..Default::default()
		}
	}
}
