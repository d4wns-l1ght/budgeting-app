//! This module reads from the model and displays the relevant information to the user
use std::{collections::HashMap, fmt::Display};

use ratatui::{
	Frame,
	layout::{self, Constraint, Layout},
	style::{Color, Style},
	symbols,
	text::Text,
	widgets::{Block, Borders, Paragraph, ScrollbarState, TableState, Tabs},
};

use crate::{
	controller::ControllerState,
	model::{Model, Sheet, SheetId},
	view::rendering::SheetWidget,
};

mod rendering;

/// The height of the rows of a sheet when displayed as a table
const ITEM_HEIGHT: u16 = 1;
/// The currency symbol used in front of the amounts
const CURRENCY_SYMBOL: char = '$';

impl Display for ControllerState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let chars: String = self.last_chars.iter().collect();
		let nums: String = self.last_nums.iter().map(|d| d.to_string()).collect();
		write!(f, "{}{}", chars, nums)
	}
}

/// A helper function to format currency according to accounting formatting
/// E.g. -10.0 becomes "$(10.00)" and 10.0 becomes "$10.00"
fn format_currency(a: &f64) -> String {
	if *a >= 0.0 {
		format!("{}{:05.2}", CURRENCY_SYMBOL, a)
	} else {
		format!("{}({:05.2})", CURRENCY_SYMBOL, -a)
	}
}

/// A struct to track the view states of sheets
struct SheetState {
	/// The state of the table used to display the sheet
	pub table_state: TableState,
	/// The state of the scrollbar displayed alongside the sheet
	pub scroll_state: ScrollbarState,
	/// The number of visible rows on the screen. This is used for scrolling up and down by half
	/// the visible rows
	pub visible_row_num: u16,
}

impl SheetState {
	/// Creates a new SheetState with a new table state with the last row selected, a new sheet
	/// state with the last row similarly selected, and the amount of visible rows set to 0 (it
	/// will be updated when the view is rendered for the first time)
	fn new(sheet: &Sheet) -> Self {
		Self {
			table_state: TableState::default()
				.with_selected(sheet.transactions.len().saturating_sub(1)),
			scroll_state: ScrollbarState::new(
				(sheet.transactions.len().saturating_sub(1)) * ITEM_HEIGHT as usize,
			)
			.position(sheet.transactions.len().saturating_sub(1) * ITEM_HEIGHT as usize),
			visible_row_num: 0,
		}
	}

	/// Scrolls to the given row of the table
	fn scroll_to_row(&mut self, row: usize) {
		self.table_state.select(Some(row));
		self.scroll_state = self.scroll_state.position(row * ITEM_HEIGHT as usize);
	}

	/// Scrolls to the first row of the table
	fn scroll_to_first(&mut self) {
		self.table_state.select_first();
		self.scroll_state.first();
	}

	/// Scrolls to the last row of the table
	fn scroll_to_last(&mut self) {
		self.table_state.select_last();
		self.scroll_state.last();
	}

	/// updates the number of visible row according to the given areas height - 2 (as the table is
	/// bordered which takes up 2 rows worth of height)
	fn update_visible_row_num(&mut self, area: &layout::Rect) {
		self.visible_row_num = area.height - 2;
	}
}

/// Represents the view of the user
#[derive(Default)]
pub struct View {
	/// All the (loaded) states of the (currently or previously viewed) sheets
	sheet_states: HashMap<SheetId, SheetState>,
	/// The currently selected sheet. See [Model::get_sheet] for indexing logic
	selected_sheet: usize,
}

impl View {
	/// Returns a new view. Currently just returns [View::default]
	pub fn new() -> Self {
		Self::default()
	}

	/// Gets the selected sheet from the model, and unwraps it as selected_sheet should always be
	/// valid
	// NOTE: Maybe unwrap or get the main sheet? Not sure how this will interact with deleting
	// sheets
	fn get_selected_sheet<'a>(&self, model: &'a Model) -> &'a Sheet {
		model.get_sheet(self.selected_sheet).unwrap_or_else(|| {
			panic!(
				"Could not get selected sheet with index {} - internal error",
				self.selected_sheet
			)
		})
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

		let controller_text = Text::from(format!("{}", controller_state));
		frame.render_widget(controller_text, footer);
	}

	/// Scroll to the next row
	pub fn next_row(&mut self, model: &Model) {
		let sheet = model.get_sheet(self.selected_sheet).unwrap();
		let sheet_state = self.get_state_of(sheet);

		let len = sheet.transactions.len().max(1);
		let next = match sheet_state.table_state.selected() {
			Some(i) if i < len.saturating_sub(1) => i + 1,
			_ => len.saturating_sub(1), // stay at last row
		};

		sheet_state.scroll_to_row(next);
	}

	/// Scroll to the previous row
	pub fn previous_row(&mut self, model: &Model) {
		let sheet = model.get_sheet(self.selected_sheet).unwrap();
		let sheet_state = self.get_state_of(sheet);

		let prev = match sheet_state.table_state.selected() {
			Some(i) if i > 0 => i - 1,
			_ => 0, // already at the first row
		};

		sheet_state.scroll_to_row(prev);
	}

	/// Scroll to the first row
	pub fn first_row(&mut self, model: &Model) {
		self.get_state_of(self.get_selected_sheet(model))
			.scroll_to_first();
	}

	/// Scroll to the second row
	pub fn last_row(&mut self, model: &Model) {
		self.get_state_of(self.get_selected_sheet(model))
			.scroll_to_last();
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
			.saturating_sub(count);

		state.scroll_to_row(new);
	}

	/// Scroll down by a count
	pub fn down_by(&mut self, count: usize, model: &Model) {
		let state = self.get_state_of(self.get_selected_sheet(model));
		let new = state
			.table_state
			.selected()
			.unwrap_or(0)
			.saturating_add(count);

		state.scroll_to_row(new);
	}

	/// Scroll up by half the screen
	pub fn half_up(&mut self, model: &Model) {
		let state = self.get_state_of(self.get_selected_sheet(model));
		let new = state
			.table_state
			.selected()
			.unwrap_or(0)
			// sub half of visible rows
			.saturating_sub((state.visible_row_num / 2).max(1) as usize);

		state.scroll_to_row(new);
	}

	/// Scroll down by half the screen
	pub fn half_down(&mut self, model: &Model) {
		let state = self.get_state_of(self.get_selected_sheet(model));
		let new = state
			.table_state
			.selected()
			.unwrap_or(0)
			// sub half of visible rows
			.saturating_add((state.visible_row_num / 2).max(1) as usize);

		state.scroll_to_row(new);
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
}
