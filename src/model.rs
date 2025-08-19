use chrono::NaiveDate;

use crate::view::FocusedSheet;

pub type SheetId = String;

#[derive(Debug)]
pub struct Model {
	pub main_sheet: Sheet,
	pub sheets: Vec<Sheet>,
	pub filename: Option<String>,
	pub exit: bool,
}

#[derive(Debug)]
pub struct Sheet {
	pub name: String,
	pub transactions: Vec<Transaction>,
}

impl Sheet {
	fn new(name: String, transactions: Vec<Transaction>) -> Self {
		Self {
			name,
			transactions,
		}
	}
}

#[derive(Debug, Default)]
pub struct Transaction {
	pub label: String,
	pub date: NaiveDate,
	pub amount: f32,
}

impl Model {
	pub fn new(filename: Option<String>) -> Model {
		match filename {
			// TODO: Open file
			Some(filename) => {
				let (main_sheet, sheets) = Self::load_sheets(filename.as_str());
				Model {
					main_sheet,
					sheets,
					filename: Some(filename),
					exit: false,
				}
			}
			// TODO: Show recently edited files?
			None => Model {
				main_sheet: Sheet::new("Sheet0".to_string(), vec![Transaction::default()]),
				sheets: vec![],
				filename: None,
				exit: false,
			},
		}
	}

	/// Pushes a new sheet to the list of secondary sheets, with the name format "Sheet" + the
	/// index of the sheet in the sheets vec + 1 (as the default/main sheet is always sheet 0)
	pub fn create_sheet(&mut self) {
		self.sheets
			.push(Sheet::new(format!("Sheet{}", self.sheets.len()), vec![]))
	}

	fn load_sheets(filename: &str) -> (Sheet, Vec<Sheet>) {
		let mut t = vec![];
		for _ in 0..20 {
			t.push(Transaction::default());
		}
		(Sheet::new("Sheet0".to_string(), t), vec![])
	}

	pub fn get_sheet(&self, focus: FocusedSheet) -> &Sheet {
		match focus {
			FocusedSheet::Main => &self.main_sheet,
			FocusedSheet::Secondary(i) => &self.sheets[i],
		}
	}

	pub fn get_sheet_mut(&mut self, focus: FocusedSheet) -> &mut Sheet {
		match focus {
			FocusedSheet::Main => &mut self.main_sheet,
			FocusedSheet::Secondary(i) => &mut self.sheets[i],
		}
	}

	pub fn sheet_count(&self) -> usize {
		1 + self.sheets.len()
	}
}
