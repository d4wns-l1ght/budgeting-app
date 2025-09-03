//! This module handles the internal state of the program, and has no interaction with the
//! controller or state modules
use chrono::{Local, NaiveDate};

/// The id of a sheet - currently a string, which is the sheets name
pub type SheetId = String;

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

/// A single sheet, representing any series of transactions the user wants to record
#[derive(Debug)]
pub struct Sheet {
	/// The name of the sheet
	pub name: String,
	/// All of the transactions recorded in the sheet
	pub transactions: Vec<Transaction>,
}

impl Sheet {
	/// A nicer way to create a sheet
	fn new(name: String, transactions: Vec<Transaction>) -> Self {
		Self { name, transactions }
	}
}

/// A single transaction that the user can record
#[derive(Debug)]
pub struct Transaction {
	/// Whatever label the user chooses to give it
	pub label: String,
	/// The date of the transaction
	pub date: NaiveDate,
	/// The amount of the transaction
	pub amount: f64,
}

impl Default for Transaction {
	fn default() -> Self {
		Self {
			label: String::new(),
			date: NaiveDate::from(Local::now().naive_local()),
			amount: 0.0,
		}
	}
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

	/// Returns the amount of sheets
	pub fn sheet_count(&self) -> usize {
		1 + self.sheets.len()
	}
}
