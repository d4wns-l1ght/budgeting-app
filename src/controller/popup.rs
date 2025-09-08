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
	pub title: String,
	pub subtitle: Option<String>,
	pub error: Option<String>,
}

impl Debug for Popup {
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

impl Popup {
	pub fn block<'a>(title: &str) -> Block<'a> {
		Block::new()
			.borders(Borders::ALL)
			.border_type(BorderType::Rounded)
			.title(title.to_string())
	}
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

	/// Adds some initial text to the text area
	pub fn with_initial(mut self, initial: String) -> Self {
		self.text_area.insert_str(initial);
		self
	}

	pub fn with_subtitle<S>(mut self, subtitle: &S) -> Self
	where
		S: ToString + ?Sized,
	{
		self.subtitle = Some(subtitle.to_string());
		self
	}

	pub fn with_error<S>(mut self, error: &S) -> Self
	where
		S: ToString,
	{
		self.error = Some(error.to_string());
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

pub mod defaults {
	use chrono::{Local, NaiveDate};

	use crate::{
		controller::{
			ControllerState,
			popup::{InputCallback, Popup},
		},
		model::{Model, ParseTransactionMemberError, Transaction},
		view::View,
	};

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
				Popup::new("Insert/Update value", move |popup, text, model| match model
					.update_transaction_member(sheet_index, row, col, text)
				{
					Ok(()) => None,
					Err(ParseTransactionMemberError { message }) => {
						Some(popup.with_error(&message))
					}
				})
				.with_initial(cell_contents),
			);
		}
	}

	pub fn rename_sheet(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
		let sheet_index = view.selected_sheet;
		cs.popup = Some(
			Popup::new("Rename sheet", move |_popup, text, model| {
				let sheet = model
					.get_sheet_mut(sheet_index)
					.unwrap_or_else(|| panic!("Couldnt get sheet with index {sheet_index}"));
				sheet.name = text;
				None
			})
			.with_initial(view.get_selected_sheet(model).name.clone()),
		);
	}

	pub fn new_row_below(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
		let sheet_index = view.selected_sheet;
		let sheet = view.get_selected_sheet(model);
		let row = view.get_selected_row(sheet).unwrap_or(0);
		cs.popup = Some(
			Popup::new(
				"Insert row",
				new_row_date(sheet_index, (row + 1).min(sheet.transactions.len())),
			)
			.with_subtitle("(Date - leave blank for today)"),
		);
	}

	pub fn new_row_above(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
		let sheet_index = view.selected_sheet;
		let sheet = view.get_selected_sheet(model);
		let row = view.get_selected_row(sheet).unwrap_or(0);
		cs.popup = Some(
			Popup::new("Insert row", new_row_date(sheet_index, row))
				.with_subtitle("(Date - leave blank for today)"),
		);
	}

	fn new_row_date(sheet_index: usize, row: usize) -> Box<InputCallback> {
		Box::new(move |popup: Popup, text: String, _model: &mut Model| {
			if text.is_empty() {
				return Some(
					Popup::new(
						"Insert row",
						new_row_label(
							sheet_index,
							row,
							NaiveDate::from(Local::now().naive_local()),
						),
					)
					.with_subtitle("(Label)"),
				);
			}
			match Transaction::parse_date(&text) {
				Ok(date) => Some(
					Popup::new("Insert row", new_row_label(sheet_index, row, date))
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
				Popup::new("Insert row", new_row_amount(sheet_index, row, date, label))
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
				Err(ParseTransactionMemberError { message }) => Some(popup.with_error(&message)),
			}
		})
	}
}
