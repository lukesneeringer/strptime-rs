//! A date and time parser independent of any Rust date and time library.
//!
//! This library can be used to parse a date and time string into a [`RawDateTime`], which the user
//! can then convert into whatever final type is needed, without taking on a larger time library as
//! a dependency.

mod error;
mod models;
mod parser;

pub use error::ParseError;
pub use models::RawDate;
pub use models::RawDateTime;
pub use models::RawTime;

/// A result returned from date and time parsing.
type ParseResult<T> = Result<T, ParseError>;

/// A date and time parser object.
///
/// This parser is able to take a `strptime` format string and parse a string into a
/// [`RawDateTime`] struct, which the user can then use to convert into whatever format is most
/// convenient.
pub struct Parser {
  fmt: &'static str,
  opts: ParseOptions,
}

impl Parser {
  /// Create a new date and time parser.
  pub const fn new(fmt: &'static str) -> Self {
    Self { fmt, opts: ParseOptions::new() }
  }

  /// Parse the date and time provided.
  ///
  /// The following symbols are recognized:
  ///
  ///
  ///
  /// **Note:** The parser does not currently check for certain impossible combinations (such as
  /// declaring that April 21, 2012 was a Tuesday, when it was actually a Saturday). Currently,
  /// non-conclusive input (such as weekdays) are discarded. This will change in the future.
  pub fn parse(&self, date_str: impl AsRef<str>) -> ParseResult<RawDateTime> {
    parser::OnceParser::new(self.fmt, date_str.as_ref(), self.opts).parse()
  }

  /// Provide a custom function to be used if only a modulo of 100 is provided for the year (as in
  /// `4/21/12` or similar).
  ///
  /// The default behavior is:
  /// - `[00, 70)`: 21st century
  /// - `[70, 99]`: 20th century
  ///
  /// ## Example
  ///
  /// ```
  /// use strptime::Parser;
  /// let parser = Parser::new("%m/%d/%y"); // Default behavior.
  /// assert_eq!(parser.parse("04/21/12")?.date().year(), 2012);
  /// let parser = Parser::new("%m/%d/%y").modulo_year_resolution(|y| 2200 + y);
  /// assert_eq!(parser.parse("04/21/12")?.date().year(), 2212);
  /// ```
  pub const fn modulo_year_resolution(mut self, modulo_year_resolution: fn(i16) -> i16) -> Self {
    self.opts.modulo_year_resolution = modulo_year_resolution;
    self
  }
}

/// Options for date and time parsing.
#[derive(Clone, Copy)]
pub(crate) struct ParseOptions {
  modulo_year_resolution: fn(i16) -> i16,
}

impl ParseOptions {
  /// Create a new parse options object.
  pub const fn new() -> Self {
    Self { modulo_year_resolution: |y| if y >= 70 { 1900 + y } else { 2000 + y } }
  }
}
