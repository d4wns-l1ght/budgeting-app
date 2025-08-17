use chrono::NaiveDate;

#[derive(Debug, Default)]
pub struct Model {
	pub main_sheet: Sheet,
	pub sheets: Vec<Sheet>,
	pub active_sheet: ActiveSheet,
	pub filename: Option<String>,
	pub exit: bool,
}

#[derive(Debug, Default)]
pub enum ActiveSheet {
	#[default]
	Main,
	Secondary(usize),
}

#[derive(Debug)]
pub struct Sheet {
	pub name: String,
	pub transactions: Vec<Transaction>,
}

#[derive(Debug, Default)]
pub struct Transaction {
	pub label: String,
	pub date: NaiveDate,
	pub amount: f32,
}

impl Default for Sheet {
	fn default() -> Self {
		Self {
			name: "Sheet0".to_owned(),
			transactions: Default::default(),
		}
	}
}

impl Sheet {
	pub fn new(name: String) -> Self {
		Self {
			name,
			..Default::default()
		}
	}
}

impl Model {
	pub fn new(filename: Option<String>) -> Model {
		match filename {
			// TODO: Open file
			Some(filename) => Model {
				filename: Some(filename),
				..Default::default()
			},
			// TODO: Show recently edited files?
			None => Self::default(),
		}
	}

	/// Pushes a new sheet to the list of secondary sheets, with the name format "Sheet" + the
	/// index of the sheet in the sheets vec + 1 (as the default/main sheet is always sheet 0)
	pub fn create_sheet(&mut self) {
		self.sheets.push(Sheet {
			name: format!("Sheet{}", self.sheets.len() + 1),
			..Default::default()
		})
	}
}
