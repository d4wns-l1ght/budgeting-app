use std::{fmt::Debug, rc::Rc};

use ratatui::{
	crossterm::event::{KeyCode, KeyEvent},
	widgets::{Block, BorderType, Borders},
};
use tui_textarea::TextArea;

use crate::model::Model;

pub trait InputCallbackFn: Fn(Popup, String, &mut Model) -> Option<Popup> {}
impl<T> InputCallbackFn for T where T: Fn(Popup, String, &mut Model) -> Option<Popup> {}

pub type InputCallback = dyn InputCallbackFn;

pub struct Popup {
	pub text_area: TextArea<'static>,
	pub on_submit: Rc<InputCallback>,
}

impl Debug for Popup {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Popup")
			.field("text_area", &self.text_area)
			.field("on_submit", &"<closure>")
			.finish()
	}
}

impl Popup {
	pub fn block<'a>(title: &str) -> Block<'a> {
		Block::new()
			.borders(Borders::ALL)
			.border_type(BorderType::Rounded)
			.title(title.to_string())
	}
	/// Creates a new text input popup with the given [`InputCallback`]
	pub fn new<F>(f: F) -> Self
	where
		F: InputCallbackFn + 'static,
	{
		Self {
			text_area: TextArea::default(),
			on_submit: Rc::new(f),
		}
	}

	/// Adds some initial text to the text area
	pub fn with_initial(mut self, initial: String) -> Self {
		self.text_area.insert_str(initial);
		self
	}

	/// Sets the [`Block`] of the text area, using [`Self::block`].
	pub fn with_block(mut self, block: Block<'static>) -> Self {
		self.text_area.set_block(block);
		self
	}

	/// Handles the [`KeyEvent`] given.
	/// Calls [`Self::on_submit`] on [`KeyCode::Enter`], returning [`None`]
	/// Returns [`None`] on [`KeyCode::Esc`], discarding the input
	/// Otherwise, returns [`Some<Self>`] with the key event applied to [`Self::text_area`]
	pub fn handle_key_event(mut self, key_event: &KeyEvent, model: &mut Model) -> Option<Self> {
		match key_event.code {
			KeyCode::Enter => {
				let mut text = self.text_area.lines().join(" ");
				text.retain(|c| c != '\n' && c != '\r');
				(self.on_submit.clone())(self, text, model)
			}
			KeyCode::Esc => None,
			_ => {
				self.text_area.input(*key_event);
				Some(self)
			}
		}
	}
}
