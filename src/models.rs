use crate::error::ErrorKind;
use crate::ParseError;
use crate::ParseResult;

/// A representation of a raw date.
#[derive(Clone, Copy, Debug)]
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
#[derive(Copy, Clone, Debug, Default)]
pub struct RawTime {
  pub(crate) hour: u8,
  pub(crate) minute: u8,
  pub(crate) second: u8,
  pub(crate) nanosecond: u64,
  pub(crate) utc_offset: Option<i64>,
}

impl RawTime {
  /// The hour; between 0 and 23, inclusive.
  #[inline]
  pub const fn hour(&self) -> u8 {
    self.hour
  }

  /// The minute; between 0 and 59, inclusive.
  #[inline]
  pub const fn minute(&self) -> u8 {
    self.minute
  }

  /// The second; between 0 and 59, inclusive.
  #[inline]
  pub const fn second(&self) -> u8 {
    self.second
  }

  /// The microsecond; between 0 and 999,999, inclusive.
  #[inline]
  pub const fn nanosecond(&self) -> u64 {
    self.nanosecond
  }

  /// The UTC offset, in seconds, if one was parsed.
  #[inline]
  pub const fn utc_offset(&self) -> Option<i64> {
    self.utc_offset
  }
}

/// A parsed date and time.
#[derive(Debug, Clone)]
pub struct RawDateTime {
  pub(crate) src: String,
  pub(crate) date: Option<RawDate>,
  pub(crate) time: Option<RawTime>,
}

impl RawDateTime {
  /// The date, if one was parsed.
  ///
  /// If any date was parsed, the entire date is guaranteed to be valid (in other words, the month
  /// and day will never be zero).
  ///
  /// For convenience, this method sends `Result` rather than `Option` so that methods that want to
  /// handle `ParseResult` can do so here easily also. The only error this ever sends is
  /// `MissingDate`, and it's safe to send to `.ok()` if you want an `Option` instead.
  pub fn date(&self) -> ParseResult<RawDate> {
    self.date.ok_or_else(|| ParseError::new(self.src.as_str(), ErrorKind::MissingDate))
  }

  /// The time, if a time was parsed. If certain fields within the time were omitted, they will be
  /// set to `0`.
  ///
  /// For convenience, this method sends `Result` rather than `Option` so that methods that want to
  /// handle `ParseResult` can do so here easily also. The only error this ever sends is
  /// `MissingTime`, and it's safe to send to `.ok()` if you want an `Option` instead.
  pub fn time(&self) -> ParseResult<RawTime> {
    self.time.ok_or_else(|| ParseError::new(self.src.as_str(), ErrorKind::MissingTime))
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
        let time = self.time.get_or_insert_with(RawTime::default);
        time.$arg = $arg;
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

impl RawDateTime {
  pub(crate) fn set_utc_offset(&mut self, hhmm: i64) {
    let hours = hhmm / 100;
    let minutes = hhmm % 100;
    let time = self.time.get_or_insert_with(RawTime::default);
    time.utc_offset = Some(hours * 3600 + minutes * 60);
  }
}
