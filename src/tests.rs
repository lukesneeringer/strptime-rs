#![cfg(test)]

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
  check!(parser.parse("2012-04-21")?.date()?.ymd() == (2012, 4, 21));
  check!(parser.parse("1776-07-04")?.date()?.ymd() == (1776, 7, 4));
  check!(parser.parse("2012-04-21")?.time().is_err());
  let parser = Parser::new("%-m/%-d/%Y");
  check!(parser.parse("4/21/2012")?.date()?.ymd() == (2012, 4, 21));
  check!(parser.parse("7/4/1776")?.date()?.ymd() == (1776, 7, 4));
  Ok(())
}

#[test]
fn test_parse_weekday() -> ParseResult<()> {
  let parser = Parser::new("%A, %B %-d, %Y");
  check!(parser.parse("Saturday, April 21, 2012")?.date()?.ymd() == (2012, 4, 21));
  let parser = Parser::new("%a, %B %-d, %Y");
  check!(parser.parse("Sat, April 21, 2012")?.date()?.ymd() == (2012, 4, 21));
  Ok(())
}

#[test]
fn test_parse_month_abbr() -> ParseResult<()> {
  let parser = Parser::new("%Y %b %-d");
  for d in ["2012 Apr 21", "2012 apr 21", "2012 APR 21"] {
    check!(parser.parse(d)?.date()?.ymd() == (2012, 4, 21));
  }
  Ok(())
}

#[test]
fn test_parse_month() -> ParseResult<()> {
  let parser = Parser::new("%B %-d, %Y");
  for d in ["April 21, 2012", "Apr 21, 2012", "APRIL 21, 2012"] {
    check!(parser.parse(d)?.date()?.ymd() == (2012, 4, 21));
  }
  Ok(())
}

#[test]
fn test_parse_single_digits() -> ParseResult<()> {
  let parser = Parser::new("%-m/%-d/%Y");
  check!(parser.parse("3/11/2020")?.date()?.ymd() == (2020, 3, 11));
  check!(parser.parse("7/4/1776")?.date()?.ymd() == (1776, 7, 4));
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
  check!(raw.date().ok().is_none());
  check!(raw.time()?.hms() == (15, 30, 45, 0));
  Ok(())
}

#[test]
fn test_12h_time() -> ParseResult<()> {
  check!(Parser::new("%I:%M %P").parse("11:30 am")?.time()?.hms() == (11, 30, 0, 0));
  check!(Parser::new("%I:%M %P").parse("11:30 pm")?.time()?.hms() == (23, 30, 0, 0));
  check!(Parser::new("%I:%M %p").parse("11:30 PM")?.time()?.hms() == (23, 30, 0, 0));
  Ok(())
}

#[test]
fn test_nanos() -> ParseResult<()> {
  check!(
    Parser::new("%H:%M:%S%3f").parse("11:00:00500")?.time()?.hms() == (11, 0, 0, 500_000_000)
  );
  check!(
    Parser::new("%H:%M:%S%.3f").parse("11:00:00.500")?.time()?.hms() == (11, 0, 0, 500_000_000)
  );
  check!(
    Parser::new("%H:%M:%S%.6f").parse("11:00:00.500000")?.time()?.hms() == (11, 0, 0, 500_000_000)
  );
  check!(
    Parser::new("%H:%M:%S%.9f").parse("11:00:00.500000000")?.time()?.hms()
      == (11, 0, 0, 500_000_000)
  );
  Ok(())
}

#[test]
fn test_errors() -> ParseResult<()> {
  check!(Parser::new("%Y-%m-%d").parse("12-14-21").is_err()); // Expected 4 digits
  check!(Parser::new("%C").parse("20").is_err()); // Ambiguous
  check!(Parser::new("%Y %b %d").parse("2012 April 21").is_err()); // Expected "Apr"
  check!(Parser::new("%m/%d/%Y").parse("7/4/1776").is_err()); // Expected 2 digits
  check!(Parser::new("%I:%M").parse("11:30").is_err()); // No AM/PM
  check!(Parser::new("%I:%M %p").parse("11:30 P").is_err()); // Parse error: No trailing M
  check!(Parser::new("%Y-%m-%d").parse("2012-04-21T11:00:00").is_err()); // Trailing input
  Ok(())
}
