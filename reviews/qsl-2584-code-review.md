---
id: SR-2026111518064800000010
title: "Code review — QVC clock-reference catalog"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-verification-contracts; README.md, contracts/README.md, contracts/checked-operation-catalog-v1.json, src/operation_catalog.rs; external FR-829-AC-5 and FR-772-AC-2 context"
review_set: subset
---

## Summary

Reviewed QSL-2584 PR #38's QVC catalog update against the catalog-relevant parts of FSpec FR-829 and FR-772. The new family, groups, operation records, and `option.slots` operand are internally consistent; one existing trace tag on the updated slot-domain catalog test does not match the criterion it names.

## Verdict

**CONDITIONAL** — one medium test-trace finding remains; catalog values and operation shapes match the stated contract.

## Review Evidence

The catalog declares 195 operations, including the three new clock-reference identities. `clock_reference` is in `families`, `any_value`, and `any_term`; `place_owner` contains only `reference`; `clock_reference.ref` carries `place_identity_terms` and `place_has_id`; and `option.slots` now takes `place_owner`. The tests assert these catalog facts and the full operation shapes. The reviewer inspected the coder's focused logs: clock-reference, slot-domain, and catalog-completeness tests each passed 1/1; clippy completed cleanly; formatting passed with the repository's existing nightly-only import-option warnings. Schema enumeration and reader behavior are handled by the separate FSpec #761 and QSL-2634 work, outside this QVC PR's scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The slot-domain catalog-shape test is tagged to FR-772-AC-2, whose oracle requires actual reader refusals; this test only checks the catalog JSON and cannot discharge those cases. | src/operation_catalog.rs:239-255 |
