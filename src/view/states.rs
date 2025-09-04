use ratatui::{
	layout::{self},
	widgets::{ScrollbarState, TableState},
};

use crate::{
	model::Sheet,
	view::ITEM_HEIGHT,
};

/// A struct to track the view states of sheets
pub struct SheetState {
	/// The state of the table used to display the sheet
	pub table_state: TableState,
	/// The state of the scrollbar displayed alongside the sheet
	pub scroll_state: ScrollbarState,
	/// The number of visible rows on the screen. This is used for scrolling up and down by half
	/// the visible rows
	pub visible_row_num: u16,
}

impl SheetState {
	/// Creates a new `SheetState` with a new table state with the last row selected, a new sheet
	/// state with the last row similarly selected, and the amount of visible rows set to 0 (it
	/// will be updated when the view is rendered for the first time)
	pub fn new(sheet: &Sheet) -> Self {
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
	pub fn scroll_to_row(&mut self, row: usize) {
		self.table_state.select(Some(row));
		self.scroll_state = self.scroll_state.position(row * ITEM_HEIGHT as usize);
	}

	/// updates the number of visible row according to the given areas height - 2 (as the table is
	/// bordered which takes up 2 rows worth of height)
	pub fn update_visible_row_num(&mut self, area: layout::Rect) {
		self.visible_row_num = area.height - 2;
	}
}
