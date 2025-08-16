use ratatui::Frame;

use crate::model::Model;

pub struct View {}

impl View {
	pub fn new() -> View {
		View {}
	}

	pub fn render(&mut self, frame: &mut Frame, model: &Model) {
		todo!();
	}
}
