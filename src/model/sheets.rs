use std::{num::ParseFloatError, str::FromStr};

use chrono::{Local, NaiveDate, ParseError, format::ParseErrorKind};
use thiserror::Error;

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

	pub(super) fn update_date(
		&mut self,
		new_value: &str,
	) -> anyhow::Result<(), ParseTransactionMemberError> {
		self.date = NaiveDate::from_str(new_value)?;
		Ok(())
	}

	pub(super) fn update_amount(
		&mut self,
		new_value: &str,
	) -> anyhow::Result<(), ParseTransactionMemberError> {
		self.amount = f64::from_str(new_value)?;
		Ok(())
	}

	pub fn parse_date(s: &str) -> anyhow::Result<NaiveDate, ParseTransactionMemberError> {
		Ok(NaiveDate::from_str(s)?)
	}

	pub fn parse_amount(s: &str) -> anyhow::Result<f64, ParseTransactionMemberError> {
		Ok(f64::from_str(s)?)
	}
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct ParseTransactionMemberError {
	pub message: String,
}

impl From<ParseError> for ParseTransactionMemberError {
	fn from(value: ParseError) -> Self {
		Self {
			message: match value.kind() {
				ParseErrorKind::OutOfRange => "Date is out of range".to_string(),
				ParseErrorKind::Impossible => "Date is impossible".to_string(),
				ParseErrorKind::NotEnough => "Not enough information".to_string(),
				ParseErrorKind::Invalid => "Invalid characters found".to_string(),
				ParseErrorKind::TooShort => "Input too short".to_string(),
				ParseErrorKind::TooLong => "Input too long".to_string(),
				ParseErrorKind::BadFormat => "Bad format".to_string(),
				// kind is non-exhaustive
				_ => "Error parsing date from input".to_string(),
			},
		}
	}
}

impl From<ParseFloatError> for ParseTransactionMemberError {
	fn from(value: ParseFloatError) -> Self {
		Self {
			message: format!("{value}"),
		}
	}
}
