//! A date and time parser independent of any Rust date and time library.
//!
//! This library can be used to parse a date and time string into a [`RawDateTime`], which the user
//! can then convert into whatever final type is needed, without taking on a larger time library as
//! a dependency.

mod error;
mod models;
mod parser;
mod tests;

pub use error::ParseError;
pub use models::RawDate;
pub use models::RawDateTime;
pub use models::RawTime;

/// A result returned from date and time parsing.
pub type ParseResult<T> = Result<T, ParseError>;

/// A date and time parser object.
///
/// This parser is able to take a `strptime` format string and parse a string into a
/// [`RawDateTime`] struct, which the user can then use to convert into whatever format is most
/// convenient.
///
/// The following symbols are currently recognized:
///
/// ## Year
///
/// | Code | Example | Description                                            |
/// | ---- | ------- | ------------------------------------------------------ |
/// | `Y`  | `2012`  | The year, zero-padded to 4 digits.                     |
/// | `C`  | `20`    | The year divided by 100, with the remainder discarded. |
/// | `y`  | `12`    | The year modulo 100, zero-padded to 2 digits.          |
///
/// ## Month
///
/// | Code       | Example | Description                                                       |
/// | ---------- | ------- | ----------------------------------------------------------------- |
/// | `m`        | `04`    | The month number, zero-padded to two digits (`01`` = January)     |
/// | `b` or `h` | `Apr`   | The month's English name, abbreviated to three characters         |
/// | `B`        | `April` | The month's English name (abbreviations >= 3 chars also accepted) |
///
/// ## Day
///
/// | Code | Example | Description                                    |
/// | ---- | ------- | ---------------------------------------------- |
/// | `d`  | `21`    | The day of the month, zero padded to 2 digits. |
/// | `e`  | `21`    | Same as `% d`.                                 |
///
/// ## Weekday
///
/// | Code | Example  | Description                                           |
/// | ---- | -------- | ----------------------------------------------------- |
/// | `a`  | `Sun`    | The English weekday, abbreviated to three characters. |
/// | `A`  | `Sunday` | The English weekday (full name).                      |
///
/// ## Hour
///
/// | Code | Example | Description                                                 |
/// | ---- | ------- | ----------------------------------------------------------- |
/// | `H`  | `17`    | The hour, zero-padded to 2 digits, using the 24-hour clock. |
/// | `I`  | `05`    | The hour, zero-padded to 2 digits, using the 12-hour clock. |
/// | `k`  | `17`    | Same as `% H`.                                              |
/// | `p`  | `PM`    | `AM` or `PM`                                                |
/// | `P`  | `pm`    | `am` or `PM`                                                |
///
/// ## Minute
///
/// | Code | Example | Description                          |
/// | ---- | ------- | ------------------------------------ |
/// | `M`  | `30`    | The minute, zero-padded to 2 digits. |
///
/// ## Second
///
/// | Code | Example | Description                          |
/// | ---- | ------- | ------------------------------------ |
/// | `S`  | `45`    | The second, zero-padded to 2 digits. |
///
/// ## Nanosecond
///
/// | Code | Example     | Description                              |
/// | ---- | ----------- | ---------------------------------------- |
/// | `f`  | `500000000` | The nanosecond, zero-padded to 9 digits. |
///
/// ## Time Zone Offset
///
/// | Code | Example | Description                      |
/// | ---- | ------- | -------------------------------- |
/// | `z`  | `-0400` | The offset, as `MMSS`, from UTC. |
///
/// **Note:** The parser does not currently check for certain impossible combinations (such as
/// declaring that April 21, 2012 was a Tuesday, when it was actually a Saturday). Currently,
/// non-conclusive input (such as weekdays) are discarded. This will change in the future.
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
  /// The returned [`RawDateTime`] struct stores the date and time separately, and either can be
  /// `None` in the (common) case when one is parsing only a date or only a time. If a date is
  /// provided at all, it's guaranteed to be "complete enough" (e.g. it won't come back with a year
  /// and day and no month). Times are more permissive, with missing elements defaulting to 0.
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
  /// # fn main() -> strptime::ParseResult<()> {
  /// use strptime::Parser;
  /// let parser = Parser::new("%m/%d/%y"); // Default behavior.
  /// assert_eq!(parser.parse("04/21/12")?.date()?.year(), 2012);
  /// let parser = Parser::new("%m/%d/%y").modulo_year_resolution(|y| 2200 + y);
  /// assert_eq!(parser.parse("04/21/12")?.date()?.year(), 2212);
  /// # Ok(())
  /// # }
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
