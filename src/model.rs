use chrono::{Local, NaiveDate};

pub type SheetId = String;

#[derive(Debug)]
pub struct Model {
	pub main_sheet: Sheet,
	pub sheets: Vec<Sheet>,
	pub filename: Option<String>,
}

#[derive(Debug)]
pub struct Sheet {
	pub name: String,
	pub transactions: Vec<Transaction>,
}

impl Sheet {
	fn new(name: String, transactions: Vec<Transaction>) -> Self {
		Self { name, transactions }
	}
}

#[derive(Debug, Default)]
pub struct Transaction {
	pub label: String,
	pub date: NaiveDate,
	pub amount: f64,
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
		self.sheets
			.push(Sheet::new(format!("Sheet{}", self.sheets.len()), vec![]))
	}

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

	pub fn sheet_titles(&self) -> Vec<String> {
		let mut titles = vec![self.main_sheet.name.clone()];
		titles.extend(self.sheets.iter().map(|s| s.name.clone()));
		titles
	}

	pub fn get_sheet(&self, index: usize) -> Option<&Sheet> {
		if index == 0 {
			Some(&self.main_sheet)
		} else {
			self.sheets.get(index - 1)
		}
	}

	pub fn sheet_count(&self) -> usize {
		1 + self.sheets.len()
	}
}
