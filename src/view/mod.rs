use std::collections::HashMap;

use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout},
	style::{Color, Style},
	symbols,
	text::Text,
	widgets::{Block, Borders, Paragraph, ScrollbarState, TableState, Tabs},
};

use crate::{
	model::{Model, Sheet, SheetId},
	view::rendering::SheetWidget,
};

mod rendering;

const ITEM_HEIGHT: usize = 4;

pub struct SheetState {
	pub table_state: TableState,
	pub scroll_state: ScrollbarState,
}

impl SheetState {
	fn new(sheet: &Sheet) -> Self {
		Self {
			table_state: TableState::default()
				.with_selected(sheet.transactions.len().saturating_sub(1)),
			scroll_state: ScrollbarState::new(
				(sheet.transactions.len().saturating_sub(1)) * ITEM_HEIGHT,
			)
			.position(sheet.transactions.len().saturating_sub(1) * ITEM_HEIGHT),
		}
	}

	fn scroll_to_row(&mut self, row: usize) {
		self.table_state.select(Some(row));
		self.scroll_state = self.scroll_state.position(row * ITEM_HEIGHT);
	}
}

pub struct View {
	sheet_states: HashMap<SheetId, SheetState>,
	selected_sheet: usize,
}

impl View {
	pub fn new() -> Self {
		Self {
			sheet_states: HashMap::new(),
			selected_sheet: 0,
		}
	}

	fn get_state_of(&mut self, sheet: &Sheet) -> &mut SheetState {
		self.sheet_states
			.entry(sheet.name.clone())
			.or_insert_with(|| SheetState::new(sheet))
	}

	pub fn active_sheet<'a>(&self, model: &'a Model) -> Option<&'a Sheet> {
		model.get_sheet(self.selected_sheet)
	}

	pub fn render(&mut self, frame: &mut Frame, model: &Model) {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Length(3),
				Constraint::Min(5),
				Constraint::Length(3),
			])
			.split(frame.area());

		let title_block = Block::default()
			.borders(Borders::ALL)
			.style(Style::default());
		let title = Paragraph::new(Text::styled(
			model.filename.as_deref().unwrap_or("scratch"),
			Style::default().fg(Color::Green),
		))
		.block(title_block);

		frame.render_widget(title, chunks[0]);

		let sheet = model.get_sheet(self.selected_sheet).unwrap();

		let sheet_state = self.get_state_of(sheet);

		let sheet_widget = SheetWidget { sheet };

		frame.render_stateful_widget(sheet_widget, chunks[1], sheet_state);

		let tabs = Tabs::new(model.sheet_titles())
			.block(Block::bordered().title_top("Sheets"))
			.highlight_style(Style::default().fg(Color::Yellow))
			.select(self.selected_sheet)
			.divider(symbols::DOT)
			.padding(" | ", " | ");

		frame.render_widget(tabs, chunks[2]);
	}

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

	pub fn previous_row(&mut self, model: &Model) {
		let sheet = model.get_sheet(self.selected_sheet).unwrap();
		let sheet_state = self.get_state_of(sheet);

		let prev = match sheet_state.table_state.selected() {
			Some(i) if i > 0 => i - 1,
			_ => 0, // already at the first row
		};

		sheet_state.scroll_to_row(prev);
	}

	pub fn next_column(&mut self, model: &Model) {
		self.get_state_of(model.get_sheet(self.selected_sheet).unwrap())
			.table_state
			.select_next_column();
	}

	pub fn previous_column(&mut self, model: &Model) {
		self.get_state_of(model.get_sheet(self.selected_sheet).unwrap())
			.table_state
			.select_previous_column();
	}

	pub fn next_sheet(&mut self, model: &Model) {
		let count = model.sheet_count();
		if count > 0 {
			self.selected_sheet = (self.selected_sheet + 1) % count;
		}
	}

	pub fn previous_sheet(&mut self, model: &Model) {
		let count = model.sheet_count();
		if count > 0 {
			self.selected_sheet = (self.selected_sheet + count - 1) % count;
		}
	}
}
