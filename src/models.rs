use crate::error::ErrorKind;
use crate::ParseError;
use crate::ParseResult;

/// A representation of a raw date.
#[derive(Debug, Clone, Copy)]
pub struct RawDate {
  pub(crate) year: Option<i16>,
  pub(crate) month: Option<u8>,
  pub(crate) day: Option<u8>,
}

impl RawDate {
  fn new() -> Self {
    Self { year: None, month: None, day: None }
  }

  /// The calendar year.
  #[inline]
  pub fn year(&self) -> i16 {
    self.year.unwrap()
  }

  /// The calendar month, between 1 and 12, inclusive.
  #[inline]
  pub fn month(&self) -> u8 {
    self.month.unwrap()
  }

  /// The day of the month; between 1 and 31, inclusive.
  #[inline]
  pub fn day(&self) -> u8 {
    self.day.unwrap()
  }

  pub(crate) fn assert_complete(&self, src: &str) -> ParseResult<()> {
    if self.year.is_none() || self.month.is_none() || self.day.is_none() {
      Err(ParseError { src: src.into(), index: None, kind: ErrorKind::IncompleteDate })?;
    }
    Ok(())
  }
}

/// A representation of time.
#[derive(Debug, Clone, Copy)]
pub struct RawTime {
  pub(crate) hour: u8,
  pub(crate) minute: u8,
  pub(crate) second: u8,
  pub(crate) nanosecond: u64,
}

impl RawTime {
  pub(crate) fn new() -> Self {
    Self { hour: 0, minute: 0, second: 0, nanosecond: 0 }
  }

  /// The hour; between 0 and 23, inclusive.
  #[inline]
  pub fn hour(&self) -> u8 {
    self.hour
  }

  /// The minute; between 0 and 59, inclusive.
  #[inline]
  pub fn minute(&self) -> u8 {
    self.minute
  }

  /// The second; between 0 and 59, inclusive.
  #[inline]
  pub fn second(&self) -> u8 {
    self.second
  }

  /// The microsecond; between 0 and 999,999, inclusive.
  #[inline]
  pub fn nanosecond(&self) -> u64 {
    self.nanosecond
  }
}

/// A parsed date and time.
#[derive(Debug, Clone)]
pub struct RawDateTime {
  pub(crate) date: Option<RawDate>,
  pub(crate) time: Option<RawTime>,
}

impl RawDateTime {
  /// The date, if one was parsed.
  ///
  /// If any date was parsed, the entire date is guaranteed to be valid (in other words, the month
  /// and day will never be zero).
  pub fn date(&self) -> Option<RawDate> {
    self.date
  }

  /// The time, if a time was parsed. If certain fields within the time were omitted, they will be
  /// set to `0`.
  pub fn time(&self) -> Option<RawTime> {
    self.time
  }

  pub(crate) fn assert_complete(&self, src: &str) -> ParseResult<()> {
    if let Some(date) = &self.date {
      date.assert_complete(src)?;
    }
    Ok(())
  }
}

macro_rules! set_date {
  ($($fn_name:ident($arg:ident: $arg_type:ty)),*) => {
    impl RawDateTime {
      $(pub(crate) fn $fn_name(&mut self, $arg: $arg_type) {
        let mut date = self.date.take().unwrap_or_else(RawDate::new);
        date.$arg = Some($arg);
        self.date = Some(date);
      })*
    }
  }
}
set_date! (
  set_year(year: i16),
  set_month(month: u8),
  set_day(day: u8)
);

macro_rules! set_time {
  ($($fn_name:ident($arg:ident: $arg_type:ty)),*) => {
    impl RawDateTime {
      $(pub(crate) fn $fn_name(&mut self, $arg: $arg_type) {
        let mut time = self.time.take().unwrap_or_else(RawTime::new);
        time.$arg = $arg;
        self.time = Some(time);
      })*
    }
  }
}

set_time!(
  set_hour(hour: u8),
  set_minute(minute: u8),
  set_second(second: u8),
  set_nanosecond(nanosecond: u64)
);
