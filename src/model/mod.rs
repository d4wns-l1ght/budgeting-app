//! This module handles the internal state of the program, and has no interaction with the
//! controller or state modules
use chrono::{Local, NaiveDate};

/// The id of a sheet - currently a string, which is the sheets name
pub type SheetId = String;

mod sheets;

pub use sheets::{ParseTransactionMemberError, Sheet, Transaction};

/// The internal state of the program
#[derive(Debug)]
pub struct Model {
	/// The main sheet - this is one that all other sheets feed into, and is where the user will
	/// handle high-level details
	pub main_sheet: Sheet,
	// All the secondary/non-main sheets of the model - these represent individual
	// accounts/events/etc that can feed into other sheets or the main sheet
	pub sheets: Vec<Sheet>,
	// The name of the file currently being worked on. Can be None, in which case the work will not
	// be saved
	pub filename: Option<String>,
}

impl Model {
	/// Loads the model from a file if given Some(filename), or creates a new "scratch" session
	/// with no associated file
	pub fn new(filename: Option<String>) -> Model {
		match filename {
			// TODO: Open file
			Some(filename) => {
				let (main_sheet, sheets) = Self::load_sheets(filename.as_str());
				Model {
					main_sheet,
					sheets,
					filename: Some(filename),
				}
			}
			// TODO: Show recently edited files?
			None => Model {
				main_sheet: Sheet::new("Sheet0".to_string(), vec![Transaction::default()]),
				sheets: vec![],
				filename: None,
			},
		}
	}

	/// Pushes a new sheet to the list of secondary sheets, with the name format "Sheet" + the
	/// index of the sheet in the sheets vec + 1 (as the default/main sheet is always sheet 0)
	pub fn create_sheet(&mut self) {
		self.sheets.push(Sheet::new(
			format!("Sheet{}", self.sheets.len() + 1),
			vec![Transaction::default()],
		));
	}

	pub fn delete_sheet(&mut self, index: usize) {
		assert!(index != 0, "Cannot delete main sheet");
		self.sheets.remove(index - 1);
	}

	/// Returns cloned titles of all the sheets
	pub fn sheet_titles(&self) -> Vec<String> {
		let mut titles = vec![self.main_sheet.name.clone()];
		titles.extend(self.sheets.iter().map(|s| s.name.clone()));
		titles
	}

	/// Gets a sheet by index, where 0 is the main sheet, and 1..MAX is the index of the secondary
	/// sheet - 1. So an index of 3 would give the secondary sheet at self.sheets(2)
	pub fn get_sheet(&self, index: usize) -> Option<&Sheet> {
		if index == 0 {
			Some(&self.main_sheet)
		} else {
			self.sheets.get(index - 1)
		}
	}

	pub fn get_sheet_mut(&mut self, index: usize) -> Option<&mut Sheet> {
		if index == 0 {
			Some(&mut self.main_sheet)
		} else {
			self.sheets.get_mut(index - 1)
		}
	}

	pub fn get_main_sheet(&self) -> &Sheet {
		&self.main_sheet
	}

	pub fn get_main_sheet_mut(&mut self) -> &mut Sheet {
		&mut self.main_sheet
	}

	/// Returns the amount of sheets
	pub fn sheet_count(&self) -> usize {
		1 + self.sheets.len()
	}

	pub fn update_transaction_member(
		&mut self,
		sheet_index: usize,
		row: usize,
		col: usize,
		new: String,
	) -> anyhow::Result<(), sheets::ParseTransactionMemberError> {
		let sheet = self.get_sheet_mut(sheet_index).unwrap();
		let transaction = sheet.transactions.get_mut(row).unwrap();

		match col {
			0 => transaction.update_date(&new),
			1 => {
				transaction.update_label(new);
				Ok(())
			}
			2 => transaction.update_amount(&new),
			_ => Ok(()),
		}
	}

	pub fn move_transaction_up(&mut self, sheet_index: usize, row: usize) {
		self.get_sheet_mut(sheet_index)
			.unwrap()
			.transactions
			.swap(row, row.saturating_sub(1));
	}

	pub fn move_transaction_down(&mut self, sheet_index: usize, row: usize) {
		let sheet = self.get_sheet_mut(sheet_index).unwrap();
		let max = sheet.transactions.len() - 1;
		sheet.transactions.swap(row, row.saturating_add(1).min(max));
	}

	pub fn delete_row(&mut self, sheet_index: usize, row: usize) -> Transaction {
		self.get_sheet_mut(sheet_index)
			.unwrap()
			.transactions
			.remove(row)
	}

	pub fn insert_row(&mut self, sheet_index: usize, row: usize, value: Transaction) {
		self.get_sheet_mut(sheet_index)
			.unwrap()
			.transactions
			.insert(row, value);
	}

	pub fn copy_row(&mut self, sheet_index: usize, row: usize) -> Transaction {
		self.get_sheet(sheet_index)
			.unwrap()
			.transactions
			.get(row)
			.unwrap()
			.clone()
	}

	/// Loads the sheets from a file
	// TODO: SQL? JSON? Some other serialization?
	fn load_sheets(filename: &str) -> (Sheet, Vec<Sheet>) {
		let mut t_m = vec![];
		let mut t_s = vec![];
		for _ in 0..=20 {
			t_m.push(Transaction::default());
			t_s.push(Transaction {
				label: "foo".to_string(),
				date: NaiveDate::from(Local::now().naive_local()),
				amount: 15.0,
			});
			t_s.push(Transaction {
				label: "bar".to_string(),
				date: NaiveDate::from(Local::now().naive_local()),
				amount: 20.0,
			});
			t_s.push(Transaction {
				label: "baz".to_string(),
				date: NaiveDate::from(Local::now().naive_local()),
				amount: 1_294.439_8,
			});
			t_s.push(Transaction {
				label: "baz".to_string(),
				date: NaiveDate::from(Local::now().naive_local()),
				amount: -1_294.439_8,
			});
			t_s.push(Transaction {
				label: "baz".to_string(),
				date: NaiveDate::from(Local::now().naive_local()),
				amount: 1_294.439_8,
			});
		}
		(
			Sheet::new("Sheet0".to_string(), t_m),
			vec![Sheet::new("Sheet1".to_string(), t_s)],
		)
	}
}
