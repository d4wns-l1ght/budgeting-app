use chrono::{Local, NaiveDate};

use crate::{
	controller::{
		ControllerState,
		popup::{Info, InputCallback, Input, InputInner, Popup, PopupBehaviour},
	},
	model::{Model, ParseTransactionMemberError, Transaction},
	view::View,
};

pub fn help(_view: &mut View, _model: &mut Model, cs: &mut ControllerState) {
	let text = "Keymap help

General
    Press <q> to quit.
    Press <?> to open this window.
    Press <Esc> to close any popup.
        (You can press <q> to close popups without text input, like this one)

Navigation
    [h j k l]/[← ↑ ↓ →] for moving.
    (count)[j k]/[↑ ↓] can be used when moving up and down.
    [H L]/<S-←><S-→> for moving between sheets
    <C-u>/<Pgup> and <C-d>/<Pgdn> for scrolling.
    <gg>/<Home> and <G>/<End> for first and last rows.

Manipulation
    <i> - change the value of the selected cell
    <y> - yank/copy the current line
    <d> - delete the current line
        NOTE: There is currently no undo button.
    <p> - put/paste the last yanked/deleted line below
    <P> - put/paste the last yanked/deleted line above
    <o> - insert new row below
    <O> - insert new row above
    <C-t> - create a new sheet
    <C-r> - rename the current sheet
";
	cs.popup = Some(Info(Box::default()).with_text(text).with_title("Help"));
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
			Input(Box::new(InputInner::new(
				"Insert/Update value",
				move |popup, text, model| match model.update_transaction_member(
					sheet_index,
					row,
					col,
					text,
				) {
					Ok(()) => None,
					Err(ParseTransactionMemberError { message }) => Some(popup.with_error(message)),
				},
			)))
			.with_text(cell_contents),
		);
	}
}

pub fn rename_sheet(view: &mut View, model: &mut Model, cs: &mut ControllerState) {
	let sheet_index = view.selected_sheet;
	cs.popup = Some(
		Input(Box::new(InputInner::new(
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
		Input(Box::new(InputInner::new(
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
		Input(Box::new(InputInner::new(
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
				Input(Box::new(InputInner::new(
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
				Input(Box::new(InputInner::new(
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
			Input(Box::new(InputInner::new(
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
