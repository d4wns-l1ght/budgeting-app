use std::num::ParseFloatError;

use chrono::{format::ParseErrorKind, ParseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("{message}")]
	UpdateTransactionMemberError { message: String },
	#[error("There was a problem when moving the row")]
	MoveRowError,
}

impl From<ParseError> for Error {
	fn from(value: ParseError) -> Self {
		Self::UpdateTransactionMemberError {
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

impl From<ParseFloatError> for Error {
	fn from(value: ParseFloatError) -> Self {
		Self::UpdateTransactionMemberError {
			message: format!("{value}"),
		}
	}
}
