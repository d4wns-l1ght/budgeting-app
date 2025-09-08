//! This module reads from the model and displays the relevant information to the user
use std::{collections::HashMap, fmt::Display};

use ratatui::{
	Frame,
	layout::{Constraint, Layout},
	style::{Color, Style},
	symbols,
	text::Text,
	widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::{
	controller::ControllerState,
	model::{Model, Sheet, SheetId, Transaction},
	view::{
		rendering::{PopupWidget, SheetWidget},
		states::SheetState,
	},
};

mod rendering;
mod states;

/// The height of the rows of a sheet when displayed as a table
const ITEM_HEIGHT: u16 = 1;
/// The currency symbol used in front of the amounts
const CURRENCY_SYMBOL: char = '$';

impl Display for ControllerState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let chars: String = self.last_chars.iter().collect();
		let nums: String = self
			.last_nums
			.iter()
			.map(std::string::ToString::to_string)
			.collect();
		write!(f, "{chars}{nums}")
	}
}

/// A helper function to format currency according to accounting formatting
/// E.g. -10.0 becomes "$(10.00)" and 10.0 becomes "$10.00"
fn format_currency(a: f64) -> String {
	if a >= 0.0 {
		format!("{CURRENCY_SYMBOL}{a:05.2}")
	} else {
		format!("{}({:05.2})", CURRENCY_SYMBOL, -a)
	}
}

pub fn get_string_of_transaction_member(transaction: &Transaction, index: usize) -> String {
	match index {
		0 => transaction.date.to_string(),
		1 => transaction.label.clone(),
		2 => transaction.amount.to_string(),
		_ => String::new(),
	}
}

/// Represents the view of the user
#[derive(Default)]
pub struct View {
	/// All the (loaded) states of the (currently or previously viewed) sheets
	sheet_states: HashMap<SheetId, SheetState>,
	/// The currently selected sheet. See [`Model::get_sheet`] for indexing logic
	pub selected_sheet: usize,
}

impl View {
	/// Returns a new view. Currently just returns [`View::default`]
	pub fn new() -> Self {
		Self::default()
	}

	/// Gets the `selected_sheet` from the model, and unwraps it as `selected_sheet` should always be
	/// valid
	// NOTE: Maybe unwrap or get the main sheet? Not sure how this will interact with deleting
	// sheets
	pub fn get_selected_sheet<'a>(&self, model: &'a Model) -> &'a Sheet {
		model.get_sheet(self.selected_sheet).unwrap_or_else(|| {
			panic!(
				"Could not get selected sheet with index {} - internal error",
				self.selected_sheet
			)
		})
	}

	pub fn get_selected_cell(&mut self, sheet: &Sheet) -> Option<(usize, usize)> {
		self.get_state_of(sheet).table_state.selected_cell()
	}

	pub fn get_selected_row(&mut self, sheet: &Sheet) -> Option<usize> {
		self.get_state_of(sheet).table_state.selected()
	}

	/// Finds the stored state of a given sheet, or creates a new state to track as this is the
	/// first time the user has viewed this sheet
	fn get_state_of(&mut self, sheet: &Sheet) -> &mut SheetState {
		self.sheet_states
			.entry(sheet.name.clone())
			.or_insert_with(|| SheetState::new(sheet))
	}

	/// Renders the view for the user
	pub fn render(&mut self, frame: &mut Frame, model: &Model, controller_state: &ControllerState) {
		let [header, sheet_area, sheets_list, footer] = Layout::vertical([
			Constraint::Length(3),
			Constraint::Min(5),
			Constraint::Length(3),
			Constraint::Length(1),
		])
		.areas(frame.area());

		let title_block = Block::default()
			.borders(Borders::ALL)
			.style(Style::default());
		let title = Paragraph::new(Text::styled(
			model.filename.as_deref().unwrap_or("scratch"),
			Style::default().fg(Color::Green),
		))
		.block(title_block);

		frame.render_widget(title, header);

		let sheet = self.get_selected_sheet(model);

		let sheet_state = self.get_state_of(sheet);

		let sheet_widget = SheetWidget { sheet };

		frame.render_stateful_widget(sheet_widget, sheet_area, sheet_state);

		let tabs = Tabs::new(model.sheet_titles())
			.block(Block::bordered().title_top("Sheets"))
			.highlight_style(Style::default().fg(Color::Yellow))
			.select(self.selected_sheet)
			.divider(symbols::DOT)
			.padding(" | ", " | ");

		frame.render_widget(tabs, sheets_list);

		let controller_text = Text::from(format!("{controller_state}"));
		frame.render_widget(controller_text, footer);

		if let Some(popup) = controller_state.popup.as_ref() {
			let popup_widget = PopupWidget { popup };
			frame.render_widget(popup_widget, frame.area());
		}
	}

	/// Scroll to the given row
	pub fn jump_to_row(&mut self, row: usize, model: &Model) {
		self.get_state_of(self.get_selected_sheet(model))
			.scroll_to_row(row.saturating_sub(1));
	}

	/// Scroll to the next row
	pub fn next_row(&mut self, model: &Model) {
		self.down_by(1, model);
	}

	/// Scroll to the previous row
	pub fn previous_row(&mut self, model: &Model) {
		self.up_by(1, model);
	}

	/// Scroll to the first row
	pub fn first_row(&mut self, model: &Model) {
		self.get_state_of(self.get_selected_sheet(model))
			.scroll_to_row(0);
	}

	/// Scroll to the last row
	pub fn last_row(&mut self, model: &Model) {
		let sheet = self.get_selected_sheet(model);
		self.get_state_of(sheet)
			.scroll_to_row(sheet.transactions.len().saturating_sub(1));
	}

	/// Move the cursor to the next column
	pub fn next_column(&mut self, model: &Model) {
		self.get_state_of(self.get_selected_sheet(model))
			.table_state
			.select_next_column();
	}

	/// Move the cursor to the previous column
	pub fn previous_column(&mut self, model: &Model) {
		self.get_state_of(self.get_selected_sheet(model))
			.table_state
			.select_previous_column();
	}

	/// Scroll up by a count
	pub fn up_by(&mut self, count: usize, model: &Model) {
		let state = self.get_state_of(self.get_selected_sheet(model));
		let new = state
			.table_state
			.selected()
			.unwrap_or(0)
			.saturating_sub(count)
			.max(0);

		state.scroll_to_row(new);
	}

	/// Scroll down by a count
	pub fn down_by(&mut self, count: usize, model: &Model) {
		let sheet = self.get_selected_sheet(model);
		let state = self.get_state_of(sheet);
		let new = state
			.table_state
			.selected()
			.unwrap_or(0)
			.saturating_add(count)
			.min(sheet.transactions.len() - 1);

		state.scroll_to_row(new);
	}

	/// Scroll up by half the screen
	pub fn half_up(&mut self, model: &Model) {
		let count = self
			.get_state_of(self.get_selected_sheet(model))
			.visible_row_num
			.saturating_div(2);
		self.up_by(count.max(1) as usize, model);
	}

	/// Scroll down by half the screen
	pub fn half_down(&mut self, model: &Model) {
		let count = self
			.get_state_of(self.get_selected_sheet(model))
			.visible_row_num
			.saturating_div(2);
		self.down_by(count.max(1) as usize, model);
	}

	/// Switch to the next sheet
	pub fn next_sheet(&mut self, model: &Model) {
		let count = model.sheet_count();
		if count > 0 {
			self.selected_sheet = (self.selected_sheet + 1) % count;
		}
	}

	/// Switch to the previous sheet
	pub fn previous_sheet(&mut self, model: &Model) {
		let count = model.sheet_count();
		if count > 0 {
			self.selected_sheet = (self.selected_sheet + count - 1) % count;
		}
	}

	pub fn deselect_cell(&mut self, model: &Model) {
		self.get_state_of(self.get_selected_sheet(model))
			.deselect_cell();
	}
}
