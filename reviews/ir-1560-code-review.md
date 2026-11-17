---
id: SR-2026111518064800000006
title: "Code review of filament-verification-contracts PR #34: replace chrono with a time-backed Timestamp wire type"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-verification-contracts@cb7f48df51befe1720a9752cefd6f0d715db237f; PR #34 diff against origin/main: Cargo.toml, Cargo.lock, README.md, build.rs, src/lib.rs, src/timestamp.rs, tests/integration.rs"
review_set: subset
---
# Code review of filament-verification-contracts PR #34

## Summary

Ticket: IR-1560. The PR maps every `format: date-time` schema to a new
`crate::timestamp::Timestamp` (newtype over `time::OffsetDateTime` plus a
leap-second flag) via typify `with_conversion`, and drops `chrono`. Rust lane
folded in.

Evidence gathered at cb7f48d (throwaway target dir, temporary worktree):

- `cargo tree -i chrono -e all --target all` with default, `--all-features` and
  `--no-default-features`: chrono matches no package. `time` is built with the
  `parsing` feature only; `time-macros` is in Cargo.lock but not in the build
  graph. New transitive crates: deranged, num-conv, powerfmt, time-core. time
  0.3.55 MSRV 1.88 against toolchain/rust-version 1.98.1.
- `cargo deny check advisories`: ok, exit 0; full `cargo deny check`: advisories,
  bans, licenses, sources ok. deny.toml unchanged (no ignore).
- `cargo test --locked`: all suites pass. `cargo clippy --all-targets
  --all-features -D warnings`: clean.
- Generated code: `e02_contracts.rs` from origin/main and from HEAD differ in
  exactly 7 lines, the 7 `format: date-time` fields (asOf x2, validUntil x2,
  producedAt, assessedAt, observedAt), chrono type to `crate::timestamp::Timestamp`.
  `shared_reference_v1.rs` and `shared_reference_v2.rs` are byte-identical.
  typify's SchemaCache matches the conversion schema exactly (metadata
  stripped), and all 7 schemas are plain `{type: string, format: date-time}`.
- Differential run (throwaway crate, chrono 0.4.45 serde vs the new type through
  serde_json): 57 hand-picked inputs and 400,000 random RFC 3339-shaped inputs
  (T/t/space separators, 0..9-digit fractions, z/Z/-00:00/random offsets, leap
  seconds at any minute, years 0000-0002 and 9997-9999). Random: 392,706
  identical outputs, 7,291 rejected by both, 3 differences, all at the year
  boundaries (FND-001 and the 9999 overflow in FND-003). 0/3/6/9-digit fractions,
  Z normalisation, `:60` with and without offsets, lowercase t/z, space separator
  and `-00:00` all match chrono byte for byte.
- Mutation check: five mutants of `src/timestamp.rs` (drop ms branch, drop us
  branch, drop zero-fraction branch, disable leap detection, skip UTC
  conversion) each fail `wire_timestamps_keep_the_chrono_era_wire_format`.
- No panics on input: `replace_range(17..19)` runs only after `text.get(17..19)`
  proved both char boundaries; no unwrap in production code.
- Ecosystem: filament-contract-ir/filament-contract-model and
  filament-contract-codegen use only `operation_catalog` from this crate, and
  this crate is the only chrono dependent in both lockfiles, so the pin bump
  will empty `cargo tree -i chrono` there. filament-verification validates every
  producer timestamp with chrono `parse_from_rfc3339` and all fixtures use plain
  `YYYY-MM-DDTHH:MM:SSZ` values in 2026, which the new type accepts unchanged. No
  consumer reads the typed date-time fields. filament-verification keeps its own
  direct chrono dependency (out of scope for this PR).
- README dependency sentence and the Cargo.toml comment are accurate.

## Verdict

Changes requested. The wire format is preserved for every timestamp a real
producer emits, and the dependency, deny and generated-code claims all hold.
Two medium edge defects in `src/timestamp.rs` should be fixed in this PR:
malformed output for negative UTC years (FND-001), and an undocumented
loosening that accepts any ASCII separator (FND-002). Each is a few lines plus
test rows. The low findings are PR-body accuracy, test pinning and Ord.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Display writes the year with `{:04}`, so a UTC year below 0 comes out as `-001-12-31T23:59:00Z`. Valid RFC 3339 input reaches this, for example `0000-01-01T00:00:00+00:01`. chrono wrote `-0001-12-31T23:59:00Z`. The new output is not RFC 3339, and `Timestamp::from_str` rejects it, so it does not round-trip. Reject `at.year() < 0` after the UTC conversion (symmetric with the year-10000 overflow it already rejects), or format the way chrono did, and add test rows. | src/timestamp.rs:54-57; src/timestamp.rs:67-68 |
| FND-002 | medium | `time`'s RFC 3339 parser accepts any single byte as the date/time separator (time-0.3.55 src/parsing/parsable.rs:648). The new type therefore accepts `2026-01-01_00:00:00Z`, `2026-01-01x00:00:00Z` and similar, which chrono rejected. This loosening is not described; the PR body presents the change as tightening only. Check that byte 10 is `T`, `t` or a space before parsing, and add rejection rows. | src/timestamp.rs:42-53 |
| FND-003 | low | The PR body's "Known tightening" list is incomplete. It omits valid RFC 3339 inputs whose UTC conversion passes year 9999, for example `9999-12-31T23:00:00-02:00`, which chrono serialized as `+10000-01-01T01:00:00Z` and which is now rejected. It also omits chrono-only forms that are now rejected: the U+2212 minus sign in offsets, `+0200` offsets without a colon, a `UTC` suffix, a space before the offset, and signed years. None of these come from an ecosystem producer. | src/timestamp.rs:54-57 |
| FND-004 | low | `TIMESTAMP_REJECTED` does not pin the documented tightening. It has no surrounding-whitespace, unpadded-field, `+10000-` or offset-overflow rows, and no separator rows, so neither a regression to lenient parsing nor the FND-002 loosening fails any test. | tests/integration.rs:260-276 |
| FND-005 | low | The derived `PartialOrd`/`Ord` compares `(at, leap_second)`. A leap second is stored as `:59` with its fraction, so `23:59:60.5Z` (stored as 59.5 + leap) sorts before `23:59:59.7Z`. chrono ordered the leap second after the whole `:59` second. Order on (instant truncated to the second, leap flag, nanosecond), or drop `Ord` if nothing needs it. | src/timestamp.rs:20 |

## Dispositions

Round 1, checked at f6b798e8ac7a4a4c642c5557f68595e5ecd1eff1 (fix commit 1b69d87). Gates rerun: `cargo test --locked` passes, `cargo clippy --all-targets --all-features -D warnings` is clean, `cargo tree -i chrono` is empty, `cargo deny check advisories` passes. Differential against chrono 0.4.45, rerun: the random sweep shows the same 3 year-boundary differences, now all rejections. In the separator sweep over every ASCII byte at offset 10, the new type and chrono both accept exactly ' ', 'T' and 't'. In the boundary sweep (0000-01-01 / 9999-12-31, every hour, 4 minute offsets, both signs, with and without :60), 600 outputs are identical and the 168 inputs where chrono wrote a signed year are rejected; no accepted output fails to parse back. Mutants: removing the separator check, adding '_', dropping ' ' or 't', removing the year check, and changing the bound to < -1 each fail the rejection test (6/6).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `if at.year() < 0 { return Err(TimestampError); }` after the UTC conversion; rows 0000-01-01T00:00:00+00:01 and +05:00 added; mutants killed |
| FND-002 | fixed | `if !matches!(text.as_bytes().get(10), Some(b'T' \| b't' \| b' ')) { return Err(TimestampError); }`; ASCII sweep matches chrono exactly |
| FND-003 | fixed | The PR body lists every class found: non-RFC forms (whitespace, unpadded, signed years, +0200, U+2212, UTC suffix, space before offset), the 9999 overflow, year < 0, and the separator check |
| FND-004 | fixed | 15 rows added to TIMESTAMP_REJECTED, covering whitespace, unpadded, +10000, offset overflow (two 9999 rows), separators, year < 0 and non-RFC offsets; 6/6 mutants of the new checks killed |
| FND-005 | fixed | Derived PartialOrd/Ord removed rather than corrected. Nothing in the crate needs it (it builds), and no downstream crate (filament-contract-ir/model, -codegen, filament-verification, -protocol, -specification, core-data) uses a wire struct that holds a date-time field, let alone orders one, so removal breaks no caller. A leap-second equality test was added |
