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

  pub(crate) fn parse(self) -> ParseResult<RawDateTime> {
    let mut answer = RawDateTime { date: None, time: None, tz: None };

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
            // We do want to recognize time-based integers, even though this is a date-only parser,
            // because it's entirely reasonable that someone wants to parse a string that includes
            // the time and throw it away.
            'H' | 'I' | 'M' | 'S' => drop(input.parse_int::<u8>(2, padding)?),
            'f' => drop(input.parse_int::<u32>(9, padding)?),
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
    self.peek().ok_or_else(|| self.err(ErrorKind::InputTooShort)).and_then(|c| match *c == ch {
      true => {
        self.next();
        Ok(())
      },
      false => self.fail(ErrorKind::Unexpected),
    })
  }

  /// Parse an integer, usually with the given number of digits, from the input.
  fn parse_int<I: FromStr<Err = ParseIntError>>(
    &mut self, digits: usize, padding: Option<char>,
  ) -> Result<I, ParseError> {
    let e = self.err(ErrorKind::Unexpected);
    match padding {
      Some('-') => self.pop_front_while(|c| c.is_numeric()).parse::<I>().map_err(|_| e),
      Some(' ') => self.pop_front(digits).trim_start().parse::<I>().map_err(|_| e),
      Some('0') | None => self.pop_front(digits).parse::<I>().map_err(|_| e),
      _ => unreachable!("Invalid padding"),
    }
  }

  /// Generate a parse error.
  fn err(&self, kind: ErrorKind) -> ParseError {
    ParseError {
      src: self.src.into(),
      index: Some(self.src.len() - self.chars.collect::<String>().len()),
      kind,
    }
  }

  fn fail(&self, kind: ErrorKind) -> ParseResult<()> {
    Err(self.err(kind))
  }
}

#[derive(Debug, Default)]
struct Partials {
  century: Option<i16>,
  year_modulo: Option<i16>,
}
