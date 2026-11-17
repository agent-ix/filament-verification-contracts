---
id: SR-2026111518064800000007
title: "Gap analysis of filament-verification-contracts PR #34 (IR-1560)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-verification-contracts@cb7f48df51befe1720a9752cefd6f0d715db237f; PR #34 diff: src/timestamp.rs, build.rs, src/lib.rs, tests/integration.rs"
review_set: subset
---
# Gap analysis of filament-verification-contracts PR #34

## Summary

Ticket: IR-1560. Planless run, scoped to the PR diff. `quoin matrix --repo .
--json` lists two requirements, FR-001 (JCS encode contract) and NFR-001 (JCS
depth safety), with 12 criteria. None of them concerns the generated wire types
or `format: date-time`. The PR does not touch any criterion. Three criteria
(FR-001-AC-8, FR-001-AC-9, NFR-001-M-3) are `method-without-symbol` on main as
well; they predate this PR and fall outside the diff scope.

The ticket's acceptance criteria are not spec artifacts. Checked by hand against
the code:

- "`cargo tree -i chrono` is empty in filament-verification-contracts": met,
  under default, all and no-default features with `-e all`.
- "Generated wire types round-trip existing fixtures unchanged": met. The repo
  has no stored fixture with a date-time value. The inline TechniqueResult
  fixture round-trips, and a 400,000-input differential against chrono 0.4.45
  matched except at year boundaries (see SR-2026111518064800000006).
- CG deny green after the pin bump: outside this repo. The PR enables it, since
  this crate is the only chrono dependent in the IR and CG lockfiles.

Stub/inflation check: the three new tests assert exact output strings and
`is_err`. Five mutants of `src/timestamp.rs` are each killed. No stubs.

## Verdict

CONDITIONAL. There is no untagged criterion and no hollow evidence. One low
reverse gap: the new public `timestamp` module and its parse/format contract
have no owning requirement and its tests carry no trace tag. This matches the
pre-existing state of the generated wire module, which is also unspecified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Reverse gap: no requirement in spec/ owns the public `timestamp` module and its wire contract (accepted forms, UTC normalisation, 0/3/6/9-digit fractions, leap seconds, rejected forms). The three new tests carry no criterion trace tag, so the computed matrix cannot show this contract or its tightening. | src/timestamp.rs:1-99; tests/integration.rs:223-316 |

## Coverage

- Criteria in matrix: 12 (FR-001: 9, NFR-001: 3). In diff scope: 0.
- Untagged criteria introduced by this PR: 0.
- Reverse gaps: 1 (FND-001).
- Stubs / coverage inflation: none found.
- Semantic review: skipped (not requested; the reviewer's differential run covers intent for the date-time contract).
- Plan completion: not assessed
