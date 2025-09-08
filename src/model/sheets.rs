use std::str::FromStr;

use chrono::{Local, NaiveDate};

use crate::model::Error;
/// A single sheet, representing any series of transactions the user wants to record
#[derive(Debug, Clone)]
pub struct Sheet {
	/// The name of the sheet
	pub name: String,
	/// All of the transactions recorded in the sheet
	pub transactions: Vec<Transaction>,
}

impl Sheet {
	/// A nicer way to create a sheet
	pub(super) fn new(name: String, transactions: Vec<Transaction>) -> Self {
		Self { name, transactions }
	}
}

/// A single transaction that the user can record
#[derive(Debug, Clone)]
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

impl Transaction {
	pub(super) fn update_label(&mut self, new_value: String) {
		self.label = new_value;
	}

	pub(super) fn update_date(&mut self, new_value: &str) -> anyhow::Result<(), Error> {
		self.date = NaiveDate::from_str(new_value)?;
		Ok(())
	}

	pub(super) fn update_amount(&mut self, new_value: &str) -> anyhow::Result<(), Error> {
		self.amount = f64::from_str(new_value)?;
		Ok(())
	}
}
