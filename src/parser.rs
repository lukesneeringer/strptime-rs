use std::iter::Peekable;
use std::num::ParseIntError;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::Chars;
use std::str::FromStr;

use crate::error::ErrorKind;
use crate::ParseError;
use crate::ParseOptions;
use crate::ParseResult;
use crate::RawDateTime;

/// An object that parses one and exactly one date and time string, and is consumed.
#[must_use]
pub(crate) struct OnceParser<'a> {
  fmt: &'static str,
  date_str: &'a str,
  opts: ParseOptions,
  partials: Partials,
}

impl<'a> OnceParser<'a> {
  #[inline]
  pub(crate) fn new(fmt: &'static str, date_str: &'a str, opts: ParseOptions) -> Self {
    Self { fmt, date_str, opts, partials: Partials::default() }
  }

  pub(crate) fn parse(mut self) -> ParseResult<RawDateTime> {
    let mut answer = RawDateTime { date: None, time: None };

    // Begin iterating over the format string, and incrementally "chew" characters from the
    // beginning of the date string.
    let mut input = Input::new(self.date_str);
    let mut flag = false;
    let mut padding = None;
    for ch in self.fmt.chars() {
      match flag {
        true => {
          flag = false;
          match ch {
            // Date: Year
            'Y' => answer.set_year(input.parse_int::<i16>(4, padding)?),
            'C' => self.partials.century = Some(input.parse_int::<i16>(2, padding)?),
            'y' => self.partials.year_modulo = Some(input.parse_int::<i16>(2, padding)?),
            // Date: Month
            'm' => answer.set_month(input.parse_int::<u8>(2, padding)?),
            'b' | 'h' => answer.set_month(input.parse_month_abbr()?),
            'B' => answer.set_month(input.parse_month()?),
            // Date: Day
            'd' => answer.set_day(input.parse_int::<u8>(2, padding)?),
            'e' => answer.set_day(input.parse_int::<u8>(2, Some(padding.unwrap_or(' ')))?),
            // Time: Hour
            'H' => answer.set_hour(input.parse_int::<u8>(2, padding)?),
            'k' => answer.set_hour(input.parse_int::<u8>(2, Some(padding.unwrap_or(' ')))?),
            'I' => self.partials.hour_12 = Some(input.parse_int::<u8>(2, padding)?),
            // Time: Minute
            'M' => answer.set_minute(input.parse_int::<u8>(2, padding)?),
            // Time: Second
            'S' => answer.set_second(input.parse_int::<u8>(2, padding)?),
            // Time: Nanosecond
            'f' => answer.set_nanosecond(input.parse_int::<u64>(9, padding)?),
            // Padding change modifiers.
            '-' | '0' | ' ' => {
              padding = Some(ch);
              flag = true;
            },
            _ => input.fail(ErrorKind::InvalidFormat)?,
          }
        },
        false => match ch {
          '%' => flag = true,
          ch => input.expect_char(ch)?,
        },
      };
    }

    // Process partials.
    if let Some(year) = self.partials.year(self.date_str, &self.opts)? {
      answer.set_year(year);
    }
    if let Some(hour) = self.partials.hour(self.date_str)? {
      answer.set_hour(hour);
    }

    // Assert that our answer is complete.
    answer.assert_complete(self.date_str)?;
    Ok(answer)
  }
}

/// A wrapper around the original input, capable of easily handling errors.
struct Input<'a> {
  src: &'a str,
  chars: Peekable<Chars<'a>>,
}

impl<'a> Input<'a> {
  fn new(date_str: &'a str) -> Self {
    Self { src: date_str, chars: date_str.chars().peekable() }
  }
}

impl<'a> Deref for Input<'a> {
  type Target = Peekable<Chars<'a>>;

  fn deref(&self) -> &Self::Target {
    &self.chars
  }
}

impl<'a> DerefMut for Input<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.chars
  }
}

impl<'a> Input<'a> {
  /// Pop characters off of the beginning and yield them.
  fn pop_front(&mut self, n: usize) -> String {
    let mut s = String::with_capacity(n);
    while s.len() < n {
      let Some(ch) = self.next() else { return s };
      s.push(ch);
    }
    s
  }

  /// Pop characters off of the beginning while they satisfy the given condition.
  fn pop_front_while(&mut self, pred: impl Fn(&char) -> bool) -> String {
    let mut s = String::new();
    while self.peek().map(&pred).unwrap_or_default() {
      s.push(self.next().unwrap());
    }
    s
  }

  /// Parse a static character.
  fn expect_char(&mut self, ch: char) -> ParseResult<()> {
    self.peek().cloned().ok_or_else(|| self.err(ErrorKind::InputTooShort)).and_then(|c| {
      match c == ch {
        true => {
          self.next();
          Ok(())
        },
        false => self.fail(ErrorKind::Unexpected),
      }
    })
  }

  /// Parse an integer, usually with the given number of digits, from the input.
  fn parse_int<I: FromStr<Err = ParseIntError>>(
    &mut self, digits: usize, padding: Option<char>,
  ) -> ParseResult<I> {
    let e = self.err(ErrorKind::Unexpected);
    match padding {
      Some('-') => self.pop_front_while(|c| c.is_numeric()).parse::<I>().map_err(|_| e),
      Some(' ') => self.pop_front(digits).trim_start().parse::<I>().map_err(|_| e),
      Some('0') | None => self.pop_front(digits).parse::<I>().map_err(|_| e),
      _ => unreachable!("Invalid padding"),
    }
  }

  /// Parse a month abbreviation.
  fn parse_month_abbr(&mut self) -> ParseResult<u8> {
    let abbr = self.pop_front(3).to_lowercase();
    Ok(match abbr.as_str() {
      "jan" => 1,
      "feb" => 2,
      "mar" => 3,
      "apr" => 4,
      "may" => 5,
      "jun" => 6,
      "jul" => 7,
      "aug" => 8,
      "sep" => 9,
      "oct" => 10,
      "nov" => 11,
      "dec" => 12,
      _ => self.fail(ErrorKind::Unexpected)?,
    })
  }

  /// Parse a full month name. This succeeds if at least the three-letter abbreviation is present,
  /// and continues to parse until the month name is completed or it ceases to find a match
  /// (therefore matching "Sept", for example).
  fn parse_month(&mut self) -> ParseResult<u8> {
    let month = self.parse_month_abbr()?;
    self.trim_front_seq(match month {
      1 => "uary",
      2 => "ruary",
      3 => "ch",
      4 => "il",
      6 => "e",
      7 => "y",
      8 => "ust",
      9 => "tember",
      10 => "ober",
      11 | 12 => "ember",
      _ => "",
    });
    Ok(month)
  }

  /// Trim all or part of the given sequence, case-insensitive. Stop at the first non-match found.
  fn trim_front_seq(&mut self, seq: &'static str) {
    for seq_char in seq.to_lowercase().chars() {
      if self.expect_char(seq_char).is_ok() {
        continue;
      };
      if self.expect_char(seq_char.to_ascii_uppercase()).is_ok() {
        continue;
      }
      break;
    }
  }

  /// Generate a parse error.
  fn err(&self, kind: ErrorKind) -> ParseError {
    ParseError::new(self.src, kind)
      .at_index(self.src.len() - self.chars.clone().collect::<String>().len())
  }

  fn fail<T>(&self, kind: ErrorKind) -> ParseResult<T> {
    Err(self.err(kind))
  }
}

#[derive(Debug, Default)]
struct Partials {
  century: Option<i16>,
  year_modulo: Option<i16>,
  hour_12: Option<u8>,
  pm: Option<u8>, // 0 or 12
}

impl Partials {
  /// Return the full hour.
  fn hour(&self, src: &str) -> ParseResult<Option<u8>> {
    match (self.hour_12, self.pm) {
      (Some(12), Some(pm)) => Ok(Some(12 - pm)),
      (Some(h), Some(pm)) if h != 12 => Ok(Some(h + pm)),
      (None, None) => Ok(None),
      _ => Err(ParseError::new(src, ErrorKind::Ambiguous))?,
    }
  }

  /// Return the full year.
  fn year(&self, src: &str, opts: &ParseOptions) -> ParseResult<Option<i16>> {
    match (self.century, self.year_modulo) {
      (Some(c), Some(m)) => Ok(Some(c * 100 + m)),
      (Some(_), None) => Err(ParseError::new(src, ErrorKind::Ambiguous))?,
      (None, Some(m)) => Ok(Some((opts.modulo_year_resolution)(m))),
      (None, None) => Ok(None),
    }
  }
}

#[cfg(test)]
mod tests {
  use assert2::check;

  use crate::ParseResult;
  use crate::Parser;
  use crate::RawDate;
  use crate::RawTime;

  impl RawDate {
    pub(crate) fn ymd(&self) -> (i16, u8, u8) {
      (self.year.unwrap(), self.month.unwrap(), self.day.unwrap())
    }
  }

  impl RawTime {
    pub(crate) fn hms(&self) -> (u8, u8, u8, u64) {
      (self.hour, self.minute, self.second, self.nanosecond)
    }
  }

  #[test]
  fn test_parse_ymd() -> ParseResult<()> {
    let parser = Parser::new("%Y-%m-%d");
    check!(parser.parse("2012-04-21")?.date().unwrap().ymd() == (2012, 4, 21));
    check!(parser.parse("1776-07-04")?.date().unwrap().ymd() == (1776, 7, 4));
    check!(parser.parse("2012-04-21")?.time().is_none());
    let parser = Parser::new("%-m/%-d/%Y");
    check!(parser.parse("4/21/2012")?.date().unwrap().ymd() == (2012, 4, 21));
    check!(parser.parse("7/4/1776")?.date().unwrap().ymd() == (1776, 7, 4));
    Ok(())
  }

  #[test]
  fn test_parse_month_abbr() -> ParseResult<()> {
    let parser = Parser::new("%Y %b %-d");
    for d in ["2012 Apr 21", "2012 apr 21", "2012 APR 21"] {
      check!(parser.parse(d)?.date().unwrap().ymd() == (2012, 4, 21));
    }
    Ok(())
  }

  #[test]
  fn test_parse_month() -> ParseResult<()> {
    let parser = Parser::new("%B %-d, %Y");
    for d in ["April 21, 2012", "Apr 21, 2012", "APRIL 21, 2012"] {
      check!(parser.parse(d)?.date().unwrap().ymd() == (2012, 4, 21));
    }
    Ok(())
  }

  #[test]
  fn test_parse_single_digits() -> ParseResult<()> {
    let parser = Parser::new("%-m/%-d/%Y");
    check!(parser.parse("3/11/2020")?.date().unwrap().ymd() == (2020, 3, 11));
    check!(parser.parse("7/4/1776")?.date().unwrap().ymd() == (1776, 7, 4));
    Ok(())
  }

  #[test]
  fn test_parse_time() -> ParseResult<()> {
    let parser = Parser::new("%Y-%m-%d %H:%M:%S");
    let raw = parser.parse("2012-04-21 11:00:00")?;
    check!(raw.date().unwrap().ymd() == (2012, 4, 21));
    check!(raw.time().unwrap().hms() == (11, 0, 0, 0));
    Ok(())
  }

  #[test]
  fn test_parse_time_alone() -> ParseResult<()> {
    let parser = Parser::new("%H:%M:%S");
    let raw = parser.parse("15:30:45")?;
    check!(raw.date().is_none());
    check!(raw.time().unwrap().hms() == (15, 30, 45, 0));
    Ok(())
  }

  #[test]
  fn test_errors() -> ParseResult<()> {
    check!(Parser::new("%Y-%m-%d").parse("12-14-21").is_err()); // Expected 4 digits
    check!(Parser::new("%C").parse("20").is_err()); // Ambiguous
    check!(Parser::new("%Y %b %d").parse("2012 April 21").is_err()); // Expected "Apr"
    check!(Parser::new("%m/%d/%Y").parse("7/4/1776").is_err()); // Expected 2 digits
    Ok(())
  }
}
