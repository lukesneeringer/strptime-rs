use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// Errors occurring during parsing.
#[derive(Debug)]
pub struct ParseError {
  /// An owned copy of the input string.
  pub src: String,
  /// The index in the input string where the error occurred.
  pub index: Option<usize>,
  /// A machine-readable explanation of the error.
  pub kind: ErrorKind,
}

impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "{}\n{}\n{}",
      self.src,
      match self.index {
        Some(ix) => format!("{}^-----", " ".repeat(ix)),
        None => String::new(),
      },
      self.kind
    )
  }
}

impl Error for ParseError {}

/// Potential errors that occur during parsing.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
  Unexpected,
  InputTooShort,
  InputTooLong,
  IncompleteDate,
  InvalidFormat,
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "{}", match self {
      Self::Unexpected => "Input does not conform to format string",
      Self::InputTooShort => "Input terminated unexpectedly before parsing finished",
      Self::InputTooLong => "Parsing finished, but input remains",
      Self::IncompleteDate => "Date specified, but could not determine year, month, and day",
      Self::InvalidFormat => "Could not parse format string",
    })
  }
}
