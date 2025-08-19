use std::collections::HashMap;

use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout},
	style::{Color, Style, Stylize},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedSheet {
	Main,
	Secondary(usize),
}

impl FocusedSheet {
	pub(super) fn to_index(self) -> usize {
		match self {
			Self::Main => 0,
			Self::Secondary(i) => i + 1,
		}
	}

	pub(super) fn from_index(index: usize) -> Self {
		if index == 0 {
			Self::Main
		} else {
			Self::Secondary(index - 1)
		}
	}
}

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
			).position(sheet.transactions.len().saturating_sub(1) * ITEM_HEIGHT),
		}
	}

	fn scroll_to_row(&mut self, row: usize) {
		self.table_state.select(Some(row));
		self.scroll_state = self.scroll_state.position(row * ITEM_HEIGHT);
	}
}

pub struct View {
	sheet_states: HashMap<SheetId, SheetState>,
	focused_sheet: FocusedSheet,
}

impl View {
	pub fn new() -> Self {
		Self {
			sheet_states: HashMap::new(),
			focused_sheet: FocusedSheet::Main,
		}
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

		let sheet = model.get_sheet(self.focused_sheet);

		let sheet_state = self.get_state_of(sheet);

		let sheet_widget = SheetWidget { sheet };

		frame.render_stateful_widget(sheet_widget, chunks[1], sheet_state);

		let t: Vec<&str> = std::iter::once(&model.main_sheet)
			.chain(&model.sheets)
			.map(|sheet| sheet.name.as_str())
			.collect();

		let tabs = Tabs::new(t)
			.block(Block::bordered().title_top("Sheets"))
			.highlight_style(Style::default().yellow())
			.select(0)
			.divider(symbols::DOT)
			.padding(" | ", " | ");

		frame.render_widget(tabs, chunks[2]);
	}

	fn get_state_of(&mut self, sheet: &Sheet) -> &mut SheetState {
		self.sheet_states
			.entry(sheet.name.clone())
			.or_insert_with(|| SheetState::new(sheet))
	}

	pub fn next_row(&mut self, model: &Model) {
		let sheet = model.get_sheet(self.focused_sheet);
		let sheet_state = self.get_state_of(sheet);

		let len = sheet.transactions.len().max(1);
		let next = match sheet_state.table_state.selected() {
			Some(i) if i < len.saturating_sub(1) => i + 1,
			_ => len.saturating_sub(1), // stay at last row
		};

		sheet_state.scroll_to_row(next);
	}

	pub fn previous_row(&mut self, model: &Model) {
		let sheet = model.get_sheet(self.focused_sheet);
		let sheet_state = self.get_state_of(sheet);

		let prev = match sheet_state.table_state.selected() {
			Some(i) if i > 0 => i - 1,
			_ => 0, // already at the first row
		};

		sheet_state.scroll_to_row(prev);
	}

	pub fn next_column(&mut self, model: &Model) {
		self.get_state_of(model.get_sheet(self.focused_sheet))
			.table_state
			.select_next_column();
	}

	pub fn previous_column(&mut self, model: &Model) {
		self.get_state_of(model.get_sheet(self.focused_sheet))
			.table_state
			.select_previous_column();
	}

	pub fn next_sheet(&mut self, model: &Model) {
		let total = model.sheet_count();
		let index = self.focused_sheet.to_index();
		// chooses total - 1 (max range) if out of range
		let next = (index + 1).min(total - 1);
		self.focused_sheet = FocusedSheet::from_index(next);
	}

	pub fn previous_sheet(&mut self) {
		let index = self.focused_sheet.to_index();
		let prev = index.saturating_sub(1);
		self.focused_sheet = FocusedSheet::from_index(prev);
	}
}
