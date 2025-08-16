use chrono::NaiveDate;

#[derive(Debug, Default)]
pub struct Model {
	pub sheets: Vec<Sheet>,
	pub exit: bool,
}

#[derive(Debug, Default)]
pub struct Sheet {
	pub name: String,
	pub transactions: Vec<Transaction>,
}

#[derive(Debug, Default)]
pub struct Transaction {
	pub label: String,
	pub amount: f32,
	pub date: NaiveDate,
}

impl Model {
	pub fn new(filename: Option<String>) -> Model {
		match filename {
			// TODO: Open file
			Some(filename) => println!("{filename}"),
			// TODO: Show recently edited files?
			None => println!("No filename entered"),
		};
		Self::default()
	}
}
