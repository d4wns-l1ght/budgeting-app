use ratatui::Frame;

use crate::model::Model;

mod rendering;

pub struct View {}

impl View {
	pub fn new() -> View {
		View {}
	}

	pub fn render(&mut self, frame: &mut Frame, model: &Model) {
		frame.render_widget(model, frame.area());
	}
}
