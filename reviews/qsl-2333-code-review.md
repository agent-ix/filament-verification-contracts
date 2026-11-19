---
id: SR-2026111518064800000008
title: "Code review — QSL-2333 catalog slot-domain operation"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-verification-contracts; contracts/checked-operation-catalog-v1.json, src/operation_catalog.rs, README.md, contracts/README.md; external FR-772; PR #37"
review_set: subset
---

## Summary

Reviewed PR #37's QSL-2333 catalog slice against FR-772 and its catalog test. The new entry matches the specified catalog shape; one documentation count remains inaccurate.

## Verdict

**CONDITIONAL** — one low-severity documentation mismatch: both READMEs report 189 operation rows, while the catalog contains 192 unique identities.

## Findings

| ID      | Severity | Summary                                                              | Refs                                  |
| ------- | -------- | -------------------------------------------------------------------- | ------------------------------------- |
| FND-001 | low      | Both READMEs say 189 operations; the catalog contains 192 identities. | README.md:134; contracts/README.md:42 |

## Review scope

- FR-772-AC-1 and FR-772-AC-2 were checked against the catalog row and test.
- The row's identity, class, operand, rest, result, constraints, and null fields match FR-772's catalog entry.
- `slot_domain_operation_declares_its_operand_and_consumer_constraints` asserts the new row's complete relevant metadata and that both constraint kinds are declared.
- This repository only publishes catalog bytes; the consuming reader behavior is reviewed in the separate contract-ir/model PR.
- The V2 schema and `tc_233` fixture are outside this PR and have been routed to arch.

## Rust review

The diff adds a deterministic unit test beside the catalog constant. It does not add a public API, deserialization path, unsafe code, shared state, or asynchronous work. The test has meaningful equality assertions and the FR-772-AC-2 trace tag.

## Verification

- `CARGO_HOME=/home/peter/dev/.cargo-home-qsl CARGO_TARGET_DIR=/home/peter/dev/worktrees/qsl-2333-catalog/target cargo test -p filament-verification-contracts slot_domain_operation_declares_its_operand_and_consumer_constraints` — passed.
- `CARGO_HOME=/home/peter/dev/.cargo-home-qsl CARGO_TARGET_DIR=/home/peter/dev/worktrees/qsl-2333-catalog/target cargo test -p filament-verification-contracts the_embedded_document_carries_every_operation_in_full` — passed.
- `CARGO_HOME=/home/peter/dev/.cargo-home-qsl CARGO_TARGET_DIR=/home/peter/dev/worktrees/qsl-2333-catalog/target cargo fmt --all -- --check` — passed; stable rustfmt emitted warnings for the repository's nightly-only import options.
- `CARGO_HOME=/home/peter/dev/.cargo-home-qsl CARGO_TARGET_DIR=/home/peter/dev/worktrees/qsl-2333-catalog/target cargo clippy -p filament-verification-contracts --all-targets -- -D warnings` — passed.
- `CARGO_HOME=/home/peter/dev/.cargo-home-qsl CARGO_TARGET_DIR=/home/peter/dev/worktrees/qsl-2333-catalog/target cargo deny check` — passed; it emitted three `license-not-encountered` warnings for unused allow-list entries.
- `jq '.operations | length' contracts/checked-operation-catalog-v1.json` — 192; unique identity count — 192.
- `quoin matrix --repo /home/peter/dev/worktrees/qsl-2333-catalog --json` — passed; the local requirements are FR-001 and NFR-001, so it does not include external FR-772.
- No applicable AssuranceProfile was found under `spec/`.

## Environment

Resolved binaries on `terra`: quoin 0.36.7 and quire 0.52.0 (engine 0.62.2), with `spec-artifacts-process@v0.36.0`. The environment notice expected quoin 0.30.0 and quire 0.40.0 (engine 0.56.0); the resolved binaries were used without fallback.

## Test binding

`slot_domain_operation_declares_its_operand_and_consumer_constraints` is tagged `FR-772-AC-2`; its assertions cover the catalog-metadata portion of that criterion. The runtime refusal cases belong to the consuming contract-ir/model slice.
