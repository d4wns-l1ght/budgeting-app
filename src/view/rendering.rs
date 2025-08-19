use ratatui::{
	buffer::Buffer,
	layout::{Constraint, Direction, Layout, Margin, Rect},
	style::{Color, Modifier, Style},
	text::Text,
	widgets::{
		Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
		ScrollbarState, StatefulWidget, Table, TableState, Widget,
	},
};

use crate::{model::Sheet, view::SheetState};

pub(super) struct SheetWidget<'a> {
	pub sheet: &'a Sheet,
}

impl<'a> StatefulWidget for SheetWidget<'a> {
	type State = SheetState;

	fn render(
		self,
		area: ratatui::prelude::Rect,
		buf: &mut ratatui::prelude::Buffer,
		state: &mut Self::State,
	) {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(3), Constraint::Min(10)])
			.split(area);

		self.render_title(chunks[0], buf);
		self.render_table(chunks[1], buf, &mut state.table_state);
		self.render_scrollbar(area, buf, &mut state.scroll_state);
	}
}

impl SheetWidget<'_> {
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

	fn render_table(&self, area: Rect, buf: &mut Buffer, state: &mut TableState) {
		let header_style = Style::default().fg(Color::Green);

		let selected_row_style = Style::default()
			.add_modifier(Modifier::REVERSED)
			.fg(Color::Red);

		let selected_cell_style = Style::default()
			.add_modifier(Modifier::REVERSED)
			.bg(Color::Blue);

		let header = ["Date", "Label", "Amount"]
			.into_iter()
			.map(Cell::from)
			.collect::<Row>()
			.style(header_style)
			.height(1);

		let rows = self.sheet.transactions.iter().map(|data| {
			[
				data.date.to_string(),
				data.label.clone(),
				data.amount.to_string(),
			]
			.into_iter()
			.map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
			.collect::<Row>()
			.style(Style::default().fg(Color::Green))
			.height(4)
		});

		let bar = " █ ";

		// TODO: Stateful table, with scrollbar, selecting, etc
		// see https://ratatui.rs/examples/widgets/table/
		StatefulWidget::render(
			Table::new(
				rows,
				[
					Constraint::Percentage(25),
					Constraint::Percentage(50),
					Constraint::Percentage(25),
				],
			)
			.header(header)
			.row_highlight_style(selected_row_style)
			.cell_highlight_style(selected_cell_style)
			.highlight_symbol(Text::from(vec![
				"".into(),
				bar.into(),
				bar.into(),
				"".into(),
			]))
			.highlight_spacing(HighlightSpacing::Always),
			area,
			buf,
			state,
		);
	}

	fn render_scrollbar(&self, area: Rect, buf: &mut Buffer, state: &mut ScrollbarState) {
		StatefulWidget::render(
			Scrollbar::default()
				.orientation(ScrollbarOrientation::VerticalRight)
				.begin_symbol(None)
				.end_symbol(None),
			area.inner(Margin {
				vertical: 1,
				horizontal: 1,
			}),
			buf,
			state,
		)
	}
}
