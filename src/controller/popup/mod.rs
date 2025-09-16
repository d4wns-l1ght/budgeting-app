use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
	rc::Rc,
};

use enum_dispatch::enum_dispatch;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::TextArea;

use crate::model::Model;

pub mod defaults;

pub trait InputCallbackFn: Fn(Popup, String, &mut Model) -> Option<Popup> {}
impl<T> InputCallbackFn for T where T: Fn(Popup, String, &mut Model) -> Option<Popup> {}

pub type InputCallback = dyn InputCallbackFn;

#[enum_dispatch(Popup)]
pub trait PopupBehaviour {
	/// Handles the given key events. This is necessary since the popups hijack the controls while
	/// visible
	fn handle_key_event(self, key_event: &KeyEvent, model: &mut Model) -> Option<Popup>;
	/// Adds some text to the popup
	fn with_text<S: Into<String>>(self, text: S) -> Popup;
	/// Adds a title to the popup
	fn with_title<S: Into<String>>(self, title: S) -> Popup;
	/// Adds a subtitle to the popup
	fn with_subtitle<S: Into<String>>(self, subtitle: S) -> Popup;
	/// Adds an error message to the popup
	fn with_error<S: Into<String>>(self, error: S) -> Popup;
}

#[enum_dispatch]
pub enum Popup {
	InputPopup,
	InfoPopup,
	ConfirmPopup,
}

pub struct InfoPopup(Box<InfoPopupInner>);

impl Deref for InfoPopup {
	type Target = InfoPopupInner;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for InfoPopup {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Default, Debug, Clone)]
pub struct InfoPopupInner {
	text: String,
	title: String,
	subtitle: Option<String>,
	error: Option<String>,
}

impl InfoPopupInner {
	pub fn text(&self) -> &String {
		&self.text
	}

	pub fn title(&self) -> &String {
		&self.title
	}

	pub fn subtitle(&self) -> Option<&String> {
		self.subtitle.as_ref()
	}

	pub fn error(&self) -> Option<&String> {
		self.error.as_ref()
	}
}

impl PopupBehaviour for InfoPopup {
	fn handle_key_event(self, key_event: &KeyEvent, _model: &mut Model) -> Option<Popup> {
		match key_event.code {
			KeyCode::Esc | KeyCode::Char('q') => None,
			_ => Some(self.into()),
		}
	}

	fn with_text<S: Into<String>>(mut self, text: S) -> Popup {
		self.text = text.into();
		self.into()
	}

	fn with_title<S: Into<String>>(mut self, title: S) -> Popup {
		self.title = title.into();
		self.into()
	}

	fn with_subtitle<S: Into<String>>(mut self, subtitle: S) -> Popup {
		self.subtitle = Some(subtitle.into());
		self.into()
	}

	fn with_error<S: Into<String>>(mut self, error: S) -> Popup {
		self.error = Some(error.into());
		self.into()
	}
}

pub struct InputPopup(Box<InputPopupInner>);

impl Deref for InputPopup {
	type Target = InputPopupInner;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for InputPopup {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub struct InputPopupInner {
	pub text_area: TextArea<'static>,
	pub on_submit: Rc<InputCallback>,
	title: String,
	subtitle: Option<String>,
	error: Option<String>,
}

impl Debug for InputPopupInner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Popup")
			.field("text_area", &self.text_area)
			.field("on_submit", &"<closure>")
			.field("title", &self.title)
			.field("subtitle", &self.subtitle)
			.field("error", &self.error)
			.finish()
	}
}

impl InputPopupInner {
	/// Creates a new text input popup with the given [`InputCallback`]
	pub fn new<F>(title: &str, f: F) -> Self
	where
		F: InputCallbackFn + 'static,
	{
		Self {
			text_area: TextArea::default(),
			on_submit: Rc::new(f),
			title: title.to_string(),
			subtitle: None,
			error: None,
		}
	}

	pub fn title(&self) -> &String {
		&self.title
	}
	pub fn subtitle(&self) -> Option<&String> {
		self.subtitle.as_ref()
	}
	pub fn error(&self) -> Option<&String> {
		self.error.as_ref()
	}
}
impl PopupBehaviour for InputPopup {
	/// Handles the [`KeyEvent`] given.
	/// Calls [`Self::on_submit`] on [`KeyCode::Enter`], returning [`None`]
	/// Returns [`None`] on [`KeyCode::Esc`], discarding the input
	/// Otherwise, returns [`Some<Self>`] with the key event applied to [`Self::text_area`]
	fn handle_key_event(mut self, key_event: &KeyEvent, model: &mut Model) -> Option<Popup> {
		match key_event.code {
			KeyCode::Enter => {
				let mut text = self.text_area.lines().join(" ");
				text.retain(|c| c != '\n' && c != '\r');
				(self.on_submit.clone())(self.into(), text, model)
			}
			KeyCode::Esc => None,
			_ => {
				self.text_area.input(*key_event);
				Some(self.into())
			}
		}
	}

	fn with_text<S: Into<String>>(mut self, initial: S) -> Popup {
		self.text_area.insert_str(initial.into());
		self.into()
	}

	fn with_subtitle<S: Into<String>>(mut self, subtitle: S) -> Popup {
		self.subtitle = Some(subtitle.into());
		self.into()
	}

	fn with_error<S: Into<String>>(mut self, error: S) -> Popup {
		self.error = Some(error.into());
		self.into()
	}

	fn with_title<S: Into<String>>(mut self, title: S) -> Popup {
		self.title = title.into();
		self.into()
	}
}

pub struct ConfirmPopup(Box<ConfirmPopupInner>);

impl Deref for ConfirmPopup {
	type Target = ConfirmPopupInner;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ConfirmPopup {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub trait ConfirmCallbackFn: Fn(bool, &mut Model) {}
impl<T> ConfirmCallbackFn for T where T: Fn(bool, &mut Model) {}

pub type ConfirmCallback = dyn ConfirmCallbackFn;

pub struct ConfirmPopupInner {
	prompt: String,
	on_submit: Rc<ConfirmCallback>,
	title: String,
	subtitle: Option<String>,
	error: Option<String>,
}

impl ConfirmPopupInner {
	pub fn new<F>(title: &str, prompt: &str, f: F) -> Self
	where
		F: ConfirmCallbackFn + 'static,
	{
		Self {
			prompt: prompt.to_string(),
			on_submit: Rc::new(f),
			title: title.to_string(),
			subtitle: None,
			error: None,
		}
	}
	pub fn prompt(&self) -> &String {
		&self.prompt
	}
	pub fn title(&self) -> &String {
		&self.title
	}
	pub fn subtitle(&self) -> Option<&String> {
		self.subtitle.as_ref()
	}
	pub fn error(&self) -> Option<&String> {
		self.error.as_ref()
	}
}

impl PopupBehaviour for ConfirmPopup {
	/// Handles the given key events. This is necessary since the popups hijack the controls while
	/// visible
	fn handle_key_event(self, key_event: &KeyEvent, model: &mut Model) -> Option<Popup> {
		match key_event.code {
			KeyCode::Char('y') | KeyCode::Enter => {
				(self.on_submit)(true, model);
				None
			}
			KeyCode::Char('n') => {
				(self.on_submit)(false, model);
				None
			}
			KeyCode::Char('q') | KeyCode::Esc => None,
			_ => Some(self.into()),
		}
	}
	/// Adds some text to the popup
	fn with_text<S: Into<String>>(mut self, text: S) -> Popup {
		self.prompt = text.into();
		self.into()
	}
	/// Adds a title to the popup
	fn with_title<S: Into<String>>(mut self, title: S) -> Popup {
		self.title = title.into();
		self.into()
	}
	/// Adds a subtitle to the popup
	fn with_subtitle<S: Into<String>>(mut self, subtitle: S) -> Popup {
		self.subtitle = Some(subtitle.into());
		self.into()
	}
	/// Adds an error message to the popup
	fn with_error<S: Into<String>>(mut self, error: S) -> Popup {
		self.error = Some(error.into());
		self.into()
	}
}
