# strptime

[![ci](https://github.com/lukesneeringer/strptime-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/lukesneeringer/strptime/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/lukesneeringer/strptime-rs/branch/main/graph/badge.svg?token=8Ej03AkjO3)](https://codecov.io/gh/lukesneeringer/strptime-rs)
[![release](https://img.shields.io/crates/v/strptime.svg)](https://crates.io/crates/strptime)
[![docs](https://img.shields.io/badge/docs-release-blue)](https://docs.rs/strptime/)

The `strptime` crate provides date and time parsing to process strings into dates. It does not
depend on any existing date and time library, and can serve as a stand-alone parser.

The library can parse a date and time together, or either one separately. Dates are required to be
fully-specified, while times are more permissive and will default unspecified components to zero.

## Specifiers

Not all `strptime`/`strftime` specifiers are supported yet. The [`Parser`] struct documents the
list.

[`Parser`]: https://docs.rs/strptime/latest/strptime/struct.Parser.html

## Examples

Parsing a date and time:

```rs
use strptime::Parser;

let parser = Parser::new("%Y-%m-%dT%H:%M:%S");
let raw_date_time = parser.parse("2012-04-21T11:00:00").unwrap();
assert_eq!(raw_date_time.date().unwrap().year(), 2012);
```
