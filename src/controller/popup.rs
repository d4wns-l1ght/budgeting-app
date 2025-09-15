use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
	rc::Rc,
};

use enum_dispatch::enum_dispatch;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::TextArea;

use crate::model::Model;

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
	/// Gets the title of the popup
	fn title(&self) -> &String;
	/// Gets the subtitle of the popup
	fn subtitle(&self) -> Option<&String>;
	/// Gets the error message of the popup
	fn error(&self) -> Option<&String>;
}

#[enum_dispatch]
pub enum Popup {
	InputPopup,
	InfoPopup,
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

	fn title(&self) -> &String {
		&self.title
	}

	fn subtitle(&self) -> Option<&String> {
		self.subtitle.as_ref()
	}

	fn error(&self) -> Option<&String> {
		self.error.as_ref()
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

	fn title(&self) -> &String {
		&self.title
	}

	fn subtitle(&self) -> Option<&String> {
		self.subtitle.as_ref()
	}

	fn error(&self) -> Option<&String> {
		self.error.as_ref()
	}
}

pub mod defaults {
	use chrono::{Local, NaiveDate};

	use crate::{
		controller::{
			ControllerState,
			popup::{InfoPopup, InputCallback, InputPopup, InputPopupInner, Popup, PopupBehaviour},
		},
		model::{Model, ParseTransactionMemberError, Transaction},
		view::View,
	};

	pub fn help(_view: &mut View, _model: &mut Model, cs: &mut ControllerState) {
		let text = "Keymap help

General
    Press <q> to quit.
    Press <?> to open this window.
    Press <ESC> to close any popup.
        (You can press <q> to close popups without text input, like this one)

Navigation
    hjkl/←↑↓→ for moving.
    [count]jk/↑↓ can be used when moving up and down.
    HL/<S-←><S-→> for moving between sheets
    <C-u>/[pgup] and <C-d>/[pgdn] for scrolling.
    gg/[home] and G/[end] for first and last rows.

Manipulation
    i - change the value of the selected cell
    y - yank/copy the current line
    d - delete the current line
        NOTE: There is currently no undo button.
    p - put/paste the last yanked/deleted line below
    P - put/paste the last yanked/deleted line above
    o - insert new row below
    O - insert new row above
    <C-t> - create a new sheet
    <C-r> - rename the current sheet
";
		cs.popup = Some(InfoPopup(Box::default()).with_text(text).with_title("Help"));
	}

	pub fn insert_action(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
		let sheet_index = view.selected_sheet;
		let sheet = view.get_selected_sheet(model);

		if let Some((row, col)) = view.get_selected_cell(sheet) {
			// Get current value of cell
			let cell_contents = crate::view::get_string_of_transaction_member(
				sheet
					.transactions
					.get(row)
					.expect("Invalid row from table state"),
				col,
			);
			// This is a popup that will return Some(self) (with some modifications) if the user's
			// input is not valid/accepted by the model
			cs.popup = Some(
				InputPopup(Box::new(InputPopupInner::new(
					"Insert/Update value",
					move |popup, text, model| match model.update_transaction_member(
						sheet_index,
						row,
						col,
						text,
					) {
						Ok(()) => None,
						Err(ParseTransactionMemberError { message }) => {
							Some(popup.with_error(message))
						}
					},
				)))
				.with_text(cell_contents),
			);
		}
	}

	pub fn rename_sheet(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
		let sheet_index = view.selected_sheet;
		cs.popup = Some(
			InputPopup(Box::new(InputPopupInner::new(
				"Rename sheet",
				move |_popup, text, model| {
					let sheet = model
						.get_sheet_mut(sheet_index)
						.unwrap_or_else(|| panic!("Couldnt get sheet with index {sheet_index}"));
					sheet.name = text;
					None
				},
			)))
			.with_text(view.get_selected_sheet(model).name.clone()),
		);
	}

	pub fn new_row_below(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
		let sheet_index = view.selected_sheet;
		let sheet = view.get_selected_sheet(model);
		let row = view.get_selected_row(sheet).unwrap_or(0);
		cs.popup = Some(
			InputPopup(Box::new(InputPopupInner::new(
				"Insert row",
				new_row_date(sheet_index, (row + 1).min(sheet.transactions.len())),
			)))
			.with_subtitle("(Date - leave blank for today)"),
		);
	}

	pub fn new_row_above(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
		let sheet_index = view.selected_sheet;
		let sheet = view.get_selected_sheet(model);
		let row = view.get_selected_row(sheet).unwrap_or(0);
		cs.popup = Some(
			InputPopup(Box::new(InputPopupInner::new(
				"Insert row",
				new_row_date(sheet_index, row),
			)))
			.with_subtitle("(Date - leave blank for today)"),
		);
	}

	fn new_row_date(sheet_index: usize, row: usize) -> Box<InputCallback> {
		Box::new(move |popup: Popup, text: String, _model: &mut Model| {
			if text.is_empty() {
				return Some(
					InputPopup(Box::new(InputPopupInner::new(
						"Insert row",
						new_row_label(
							sheet_index,
							row,
							NaiveDate::from(Local::now().naive_local()),
						),
					)))
					.with_subtitle("(Label)"),
				);
			}
			match Transaction::parse_date(&text) {
				Ok(date) => Some(
					InputPopup(Box::new(InputPopupInner::new(
						"Insert row",
						new_row_label(sheet_index, row, date),
					)))
					.with_subtitle("(Label)"),
				),
				Err(ParseTransactionMemberError { message }) => Some(popup.with_error(&message)),
			}
		})
	}

	fn new_row_label(sheet_index: usize, row: usize, date: NaiveDate) -> Box<InputCallback> {
		Box::new(move |_popup, text: String, _model| {
			let label = text;
			Some(
				InputPopup(Box::new(InputPopupInner::new(
					"Insert row",
					new_row_amount(sheet_index, row, date, label),
				)))
				.with_subtitle("(Amount)"),
			)
		})
	}

	fn new_row_amount(
		sheet_index: usize,
		row: usize,
		date: NaiveDate,
		label: String,
	) -> Box<InputCallback> {
		Box::new(move |popup: Popup, text: String, model: &mut Model| {
			match Transaction::parse_amount(&text) {
				Ok(amount) => {
					let transaction = Transaction {
						label: label.clone(),
						date,
						amount,
					};
					model.insert_row(sheet_index, row, transaction);
					None
				}
				Err(ParseTransactionMemberError { message }) => Some(popup.with_error(message)),
			}
		})
	}
}
