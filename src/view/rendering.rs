use ratatui::{
	buffer::Buffer,
	layout::{Alignment, Constraint, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Text, ToText},
	widgets::{
		Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
		StatefulWidget, Table, TableState, Widget,
	},
};

use crate::{
	model::Sheet,
	view::{ITEM_HEIGHT, SheetState},
};

/// A temporary wrapper around a [Sheet], for the purpose of rendering
pub(super) struct SheetWidget<'a> {
	pub sheet: &'a Sheet,
}

impl StatefulWidget for SheetWidget<'_> {
	type State = SheetState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let [title, table] =
			Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);
		let [table, scrollbar] =
			Layout::horizontal([Constraint::Fill(1), Constraint::Length(2)]).areas(table);

		state.update_visible_row_num(table);
		self.render_title(title, buf);
		self.render_table(table, buf, &mut state.table_state);
		Self::render_scrollbar(scrollbar, buf, &mut state.scroll_state);
	}
}

impl SheetWidget<'_> {
	/// Renders the title of the sheet
	fn render_title(&self, area: Rect, buf: &mut Buffer) {
		// Display the title of the Sheet
		let title_block = Block::default()
			.borders(Borders::ALL)
			.style(Style::default());

		Paragraph::new(Text::styled(
			&self.sheet.name,
			Style::default().fg(Color::Green),
		))
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
			Cell::from("#"),
			Cell::from("Date"),
			Cell::from("Label"),
			Cell::from(Text::from("Amount").alignment(Alignment::Right)),
		])
		.style(header_style)
		.height(1);

		let cursor_position = state.selected();

		let rows: Vec<Row> = self
			.sheet
			.transactions
			.iter()
			.enumerate()
			.map(|(i, data)| {
				Row::new(vec![
					Cell::from({
						Text::from(
							(match cursor_position {
								Some(pos) if pos == i => i + 1,
								Some(pos) => i.abs_diff(pos),
								None => panic!(),
							})
							.to_string(),
						)
					}),
					Cell::from(data.date.to_text()),
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

		let selection_indicator = " * ";

		// TODO: Stateful table, with scrollbar, selecting, etc
		// see https://ratatui.rs/examples/widgets/table/
		StatefulWidget::render(
			Table::new(
				rows,
				[
					// line number
					Constraint::Length({
						let len = self.sheet.transactions.len();
						if len == 0 {
							1
						} else {
							u16::try_from(len.checked_ilog10().unwrap_or(0)).unwrap_or(u16::MAX) + 1
						}
					}),
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
							// +1 for currency symbol, +2 for parens on negatives
							.len(),
						)
						.unwrap_or(u16::MAX) + 3)
							.min(10),
					),
				],
			)
			.header(header)
			.block(Block::default().borders(Borders::ALL))
			.row_highlight_style(selected_row_style)
			.cell_highlight_style(selected_cell_style)
			.highlight_symbol(selection_indicator),
			area,
			buf,
			state,
		);
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
