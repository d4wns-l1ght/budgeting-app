use ratatui::{
	buffer::Buffer,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Color, Style},
	text::Text,
	widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget},
};

use crate::model::{ActiveSheet, Model, Sheet};
impl Widget for &Model {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Length(3),
				Constraint::Min(5),
				Constraint::Length(3),
			])
			.split(area);

		let words_block = Block::default()
			.borders(Borders::ALL)
			.style(Style::default());

		let words = Paragraph::new(Text::styled(
			self.filename.as_deref().unwrap_or("scratch"),
			Style::default().fg(Color::Green),
		))
		.block(words_block);

		words.render(chunks[0], buf);

		let sheet = match self.active_sheet {
			ActiveSheet::Main => &self.main_sheet,
			ActiveSheet::Secondary(index) => {
				assert!(
					index < self.sheets.len(),
					"Active Sheet index was set to a number outside the bounds of the Vec"
				);
				&self.sheets[index]
			}
		};

		sheet.render(area, buf);
	}
}

impl Widget for &Sheet {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(3), Constraint::Min(10)])
			.split(area);

		let title_block = Block::default()
			.borders(Borders::ALL)
			.style(Style::default());

		Paragraph::new(Text::styled(&self.name, Style::default().fg(Color::Green)))
			.block(title_block)
			.render(chunks[0], buf);

		// TODO: Stateful table, with scrollbar, selecting, etc
		// see https://ratatui.rs/examples/widgets/table/
		let table_block = Block::default()
			.borders(Borders::ALL)
			.style(Style::default().fg(Color::Blue));

		let header_style = Style::default().fg(Color::Green);

		let header = ["Date", "Label", "Amount"]
			.into_iter()
			.map(Cell::from)
			.collect::<Row>()
			.style(header_style)
			.height(1);

		let rows = self.transactions.iter().map(|data| {
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

		Table::new(
			rows,
			[
				Constraint::Percentage(25),
				Constraint::Percentage(50),
				Constraint::Percentage(25),
			],
		)
		.header(header)
		.block(table_block)
		.render(chunks[1], buf);
	}
}
