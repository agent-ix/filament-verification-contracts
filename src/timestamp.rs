// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The `format: date-time` wire type: an RFC 3339 instant, always normalized to UTC.
//!
//! The wire form is unchanged from the previous `chrono::DateTime<Utc>` representation: any
//! RFC 3339 offset is accepted on input, and the value is written back as UTC with a `Z`
//! suffix and a fractional part of zero, three, six or nine digits (the shortest that is
//! exact).

use std::fmt;
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};

/// An RFC 3339 `date-time` instant in UTC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    at: OffsetDateTime,
    /// `true` for an RFC 3339 leap second (`:60`), stored as `:59`.
    leap_second: bool,
}

/// The text is not an RFC 3339 `date-time` that is representable in UTC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimestampError;

impl fmt::Display for TimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid RFC 3339 date-time")
    }
}

impl std::error::Error for TimestampError {}

impl FromStr for Timestamp {
    type Err = TimestampError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // `time` folds an RFC 3339 leap second (`:60`) into `:59.999999999`, which would lose
        // the second and its fraction. Parse the second as `:59` and remember the leap instead.
        let leap_second = text.get(17..19) == Some("60");
        let parsed = if leap_second {
            let mut shifted = text.to_owned();
            shifted.replace_range(17..19, "59");
            OffsetDateTime::parse(&shifted, &Rfc3339)
        } else {
            OffsetDateTime::parse(text, &Rfc3339)
        }
        .map_err(|_| TimestampError)?;
        let at = parsed
            .checked_to_offset(UtcOffset::UTC)
            .ok_or(TimestampError)?;
        Ok(Self { at, leap_second })
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = self.at;
        let second = if self.leap_second { 60 } else { at.second() };
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{second:02}",
            at.year(),
            u8::from(at.month()),
            at.day(),
            at.hour(),
            at.minute(),
        )?;
        let nanos = at.nanosecond();
        if nanos == 0 {
            // No fractional part.
        } else if nanos.is_multiple_of(1_000_000) {
            write!(f, ".{:03}", nanos / 1_000_000)?;
        } else if nanos.is_multiple_of(1_000) {
            write!(f, ".{:06}", nanos / 1_000)?;
        } else {
            write!(f, ".{nanos:09}")?;
        }
        f.write_str("Z")
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(de::Error::custom)
    }
}
