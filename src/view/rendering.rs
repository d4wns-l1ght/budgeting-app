use ratatui::{
	buffer::Buffer,
	layout::{Alignment, Constraint, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Text},
	widgets::{
		Block, Borders, Cell, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation,
		ScrollbarState, StatefulWidget, Table, TableState, Widget,
	},
};

use crate::{
	model::Sheet,
	view::{ITEM_HEIGHT, SheetState},
};

const NUMBER_PADDING_RIGHT: u16 = 2;
const DATE_FORMAT_STRING: &str = "%d/%m/%Y";

/// A temporary wrapper around a [Sheet], for the purpose of rendering
pub(super) struct SheetWidget<'a> {
	pub sheet: &'a Sheet,
}

impl StatefulWidget for SheetWidget<'_> {
	type State = SheetState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let [header, table] =
			Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);
		let [table, scrollbar] =
			Layout::horizontal([Constraint::Fill(1), Constraint::Length(2)]).areas(table);

		state.update_visible_row_num(table);
		self.render_header(header, buf, &state.table_state);
		self.render_table(table, buf, &mut state.table_state);
		Self::render_scrollbar(scrollbar, buf, &mut state.scroll_state);
	}
}

#[allow(clippy::cast_possible_truncation)]
impl SheetWidget<'_> {
	/// Renders the title of the sheet
	fn render_header(&self, area: Rect, buf: &mut Buffer, state: &TableState) {
		// Display the contents of the selected cell, or nothing
		let title_block = Block::default()
			.borders(Borders::ALL)
			.style(Style::default());

		let text = if let Some((row, col)) = state.selected_cell() {
			let t = match self.sheet.transactions.get(row) {
				Some(t) => t,
				None => &crate::model::Transaction::default(),
			};
			crate::view::get_string_of_transaction_member(t, col)
		} else {
			String::new()
		};

		Paragraph::new(Text::styled(text, Style::default().fg(Color::Green)))
			.block(title_block)
			.render(area, buf);
	}

	/// Renders the table portion of the sheet.
	/// This is the most complicated method, as it has to be very reactive to both the state of
	/// the view and the state of the model
	fn render_table(&self, area: Rect, buf: &mut Buffer, state: &mut TableState) {
		let header_style = Style::default().fg(Color::Green);

		let selected_row_style = Style::default().bg(Color::Black);

		let selected_cell_style = Style::default()
			.add_modifier(Modifier::BOLD)
			.bg(Color::DarkGray)
			.fg(Color::Red);

		let header = Row::new(vec![
			Cell::from("Date"),
			Cell::from("Label"),
			Cell::from(Text::from("Amount").alignment(Alignment::Right)),
		])
		.style(header_style)
		.height(1);

		let [number_area, sheet_area] = Layout::horizontal([
			// line number
			Constraint::Length({
				let len = self.sheet.transactions.len();
				if len == 0 {
					1
				} else {
					// +1 for extra digit, +1 again for border
					u16::try_from(len.checked_ilog10().unwrap_or(0)).unwrap_or(u16::MAX)
						+ 2 + NUMBER_PADDING_RIGHT
				}
			}),
			Constraint::Fill(1),
		])
		.areas(area);

		let rows: Vec<Row> = self
			.sheet
			.transactions
			.iter()
			.map(|data| {
				Row::new(vec![
					Cell::from(data.date.format(DATE_FORMAT_STRING).to_string()),
					Cell::from(data.label.clone()),
					Cell::from(
						Text::from(crate::view::format_currency(data.amount))
							.alignment(Alignment::Right),
					),
				])
				.style(Style::default().fg(Color::Green))
				.height(ITEM_HEIGHT)
			})
			.collect();

		// TODO: Stateful table, with scrollbar, selecting, etc
		// see https://ratatui.rs/examples/widgets/table/
		let widths = [
			// date
			Constraint::Length(10),
			// label
			Constraint::Fill(1),
			// amount
			Constraint::Length(
				(u16::try_from(
					format!(
						"{:05.2}",
						self.sheet
							.transactions
							.iter()
							.map(|t| t.amount)
							.max_by(f64::total_cmp)
							.unwrap_or(0.0)
					)
					.len(),
				)
				// +1 for currency symbol, +2 for parens on negatives
				.unwrap_or(u16::MAX)
					+ 3)
				.min(10),
			),
		];
		StatefulWidget::render(
			Table::new(rows, widths)
				.header(header)
				.block(Block::default().borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
				.row_highlight_style(selected_row_style)
				.cell_highlight_style(selected_cell_style),
			sheet_area,
			buf,
			state,
		);

		self.render_numbers(number_area, buf, state, selected_row_style);
	}

	/// Renders the numbers
	// WARN: This HAS to be called after the table is actually rendered otherwise the indices
	// get messed up
	fn render_numbers(
		&self,
		area: Rect,
		buf: &mut Buffer,
		state: &TableState,
		selected_row_style: Style,
	) {
		let start = state.offset();
		let end = self
			.sheet
			.transactions
			.len()
			.min(start + area.height as usize - 3);
		assert!(
			end - start == area.height as usize - 3 || end - start == self.sheet.transactions.len()
		);
		let cursor_position = state.selected();
		let mut row_numbers: Vec<Line> = Vec::with_capacity(self.sheet.transactions.len());

		for i in start..end {
			row_numbers.push({
				match cursor_position {
					Some(pos) if pos == i => {
						let text = (i + 1).to_string();
						let padded = format!("{:<width$}", text, width = area.width as usize);
						Line::from(padded).style(selected_row_style)
					}
					Some(pos) => Line::from((i.abs_diff(pos)).to_string()),
					None => Line::from((i + 1).to_string()),
				}
			});
		}
		Paragraph::new(row_numbers)
			.block(
				Block::default()
					.borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
					.padding(Padding::top(1)),
			)
			.render(area, buf);
	}

	/// Renders the scrollbar of the table
	fn render_scrollbar(area: Rect, buf: &mut Buffer, state: &mut ScrollbarState) {
		StatefulWidget::render(
			Scrollbar::default()
				.orientation(ScrollbarOrientation::VerticalRight)
				.begin_symbol(Some("↑"))
				.end_symbol(Some("↓")),
			area,
			buf,
			state,
		);
	}
}
