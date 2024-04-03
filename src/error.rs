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

impl ParseError {
  pub(crate) fn new(src: &str, kind: ErrorKind) -> Self {
    Self { src: src.into(), index: None, kind }
  }

  pub(crate) fn at_index(mut self, ix: usize) -> Self {
    self.index = Some(ix);
    self
  }
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
  Ambiguous,
  IncompleteDate,
  InputTooLong,
  InputTooShort,
  InvalidFormat,
  Unexpected,
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "{}", match self {
      Self::Ambiguous => "Parsing succeeded, but the final date was ambiguous",
      Self::IncompleteDate => "Date specified, but could not determine year, month, and day",
      Self::InputTooLong => "Parsing finished, but input remains",
      Self::InputTooShort => "Input terminated unexpectedly before parsing finished",
      Self::InvalidFormat => "Could not parse format string",
      Self::Unexpected => "Input does not conform to format string",
    })
  }
}
