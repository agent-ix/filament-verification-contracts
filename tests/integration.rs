// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! End-to-end tests for the public E02 verification contract boundary.

use filament_verification_contracts::{
    MAX_ARRAY_ITEMS, MAX_JSON_BYTES, MAX_JSON_DEPTH, PublicContract, VerificationErrorCode,
    contract_version, input_kinds, jcs_canonicalize, jcs_equal, jcs_sha256, parse_bounded_json,
    validate_contract, validate_resource_envelope, validate_schema_definition,
};
use serde_json::{Value, json};
use sha2::Digest;

fn artifact_ref() -> Value {
    let digest = format!(
        "sha256:{:x}",
        sha2::Sha256::digest(b"runtime-only test reference")
    );
    json!({
        "refVersion": "ix.artifact-ref/3-draft",
        "kind": "source",
        "authority": "ix.test",
        "identity": "test-run",
        "digest": digest,
        "wire": {"identity": "ix.test.wire", "version": "1"}
    })
}

fn artifact_envelope() -> Value {
    json!({
        "wireVersion": "ix.shared-reference/2-draft",
        "kind": "artifact",
        "value": artifact_ref()
    })
}

fn minimal_technique_result() -> Value {
    let metadata: Value =
        serde_json::from_str(include_str!("../fixtures/e02-draft-2/result-metadata.json"))
            .expect("result metadata fixture is JSON");
    json!({
        "contractVersion": "filament-verification/e02-draft-2",
        "kind": "TechniqueResult",
        "resultId": "assigned-result-1",
        "planId": "plan-1",
        "taskId": "task-1",
        "runRef": artifact_envelope(),
        "techniqueId": "technique-1",
        "producedAt": "2026-01-01T00:00:00Z",
        "propertyRef": artifact_envelope(),
        "subjectRefs": [],
        "profile": "profile-1",
        "bindingRefs": [],
        "dependencyClosure": "closure-1",
        "executionScope": {},
        "execution": {"status": "completed"},
        "logical": {"outcome": "unknown"},
        "activation": {"status": "activated"},
        "participation": {"status": "complete"},
        "assumptions": [],
        "adequacy": {"status": "adequate"},
        "provenance": metadata["provenance"].clone(),
        "nativeArtifacts": [metadata["nativeArtifact"].clone()],
        "oracleContributions": []
    })
}

fn minimal_tool_capability() -> serde_json::Value {
    json!({
        "contractVersion": "filament-verification/e02-draft-2",
        "kind": "ToolCapability",
        "capabilityId": "test-capability",
        "techniqueId": "test-technique",
        "qualification": "provisional",
        "inputKinds": ["state"],
        "profiles": ["test-profile"],
        "prerequisites": [],
        "runner": {
            "kind": "manual",
            "adapterId": "test-adapter",
            "adapterVersion": "0.1.0",
        },
        "environmentPermissions": [],
        "nativeArtifactInterface": {},
    })
}

#[test]
fn contract_version_matches_schema_generated_constant() {
    assert_eq!(contract_version(), "filament-verification/e02-draft-2");
}

#[test]
fn input_kinds_lists_the_five_public_kinds() {
    let kinds = input_kinds();
    for expected in ["state", "relational", "trace", "fault", "manual"] {
        assert!(kinds.contains(&expected), "missing input kind {expected}");
    }
}

#[test]
fn valid_tool_capability_round_trips_through_validate_contract() {
    let value = minimal_tool_capability();
    let contract = validate_contract(value).expect("minimal tool capability must validate");
    assert!(matches!(contract, PublicContract::ToolCapability(_)));
}

#[test]
fn technique_result_accepts_assigned_string_id_without_artifact_or_revision_digests() {
    let value = minimal_technique_result();
    let contract = validate_contract(value).expect("result with digest-free metadata validates");
    assert!(matches!(contract, PublicContract::TechniqueResult(_)));
}

#[test]
fn technique_result_refuses_removed_artifact_digest_and_source_revision_fields() {
    let mut value = minimal_technique_result();
    value["nativeArtifacts"][0]["digest"] = json!("obsolete");
    let error = validate_contract(value.clone()).expect_err("artifact digest is not admitted");
    assert_eq!(error.code(), VerificationErrorCode::SchemaViolation);

    let mut value = minimal_technique_result();
    value["provenance"]["sourceRevision"] = json!("obsolete");
    let error = validate_contract(value).expect_err("source revision is not admitted");
    assert_eq!(error.code(), VerificationErrorCode::SchemaViolation);
}

#[test]
fn unknown_contract_kind_is_rejected() {
    let mut value = minimal_tool_capability();
    value["kind"] = json!("NotAContract");
    let error = validate_contract(value).expect_err("unknown kind must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::SchemaViolation);
}

#[test]
fn oversized_json_document_is_rejected() {
    let oversized = vec![b'a'; MAX_JSON_BYTES + 1];
    let error = parse_bounded_json(&oversized).expect_err("oversized input must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::JsonDocumentTooLarge);
}

#[test]
fn deeply_nested_json_is_rejected() {
    let mut value = json!(1);
    for _ in 0..=MAX_JSON_DEPTH + 1 {
        value = json!([value]);
    }
    let error = validate_resource_envelope(&value).expect_err("deep nesting must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::JsonNestingTooDeep);
}

#[test]
fn oversized_array_is_rejected() {
    let value = json!(vec![0_u32; MAX_ARRAY_ITEMS + 1]);
    let error = validate_resource_envelope(&value).expect_err("oversized array must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::JsonArrayTooLarge);
}

#[test]
fn jcs_canonicalize_is_key_order_independent() {
    let left = json!({"b": 1, "a": 2});
    let right = json!({"a": 2, "b": 1});
    assert!(jcs_equal(&left, &right).expect("both values are in the JCS domain"));
    assert_eq!(
        jcs_canonicalize(&left).expect("left canonicalizes"),
        b"{\"a\":2,\"b\":1}"
    );
    assert_eq!(
        jcs_canonicalize(&right).expect("right canonicalizes"),
        b"{\"a\":2,\"b\":1}"
    );
}

#[test]
fn jcs_sha256_is_deterministic_and_domain_separated() {
    let value = json!({"kind": "example"});
    let digest = jcs_sha256(&value).expect("value canonicalizes");
    assert!(digest.starts_with("sha256-jcs:"));
    assert_eq!(
        digest,
        jcs_sha256(&value).expect("value canonicalizes again")
    );
}

#[test]
fn every_verification_error_code_round_trips_through_its_stable_string() {
    for code in VerificationErrorCode::all() {
        assert_eq!(VerificationErrorCode::from_code(code.as_str()), Some(*code));
    }
}

/// `SharedArtifactEnvelope.value` is a `$ref` into the externally retained
/// `urn:ix:shared-reference:2-draft` schema, resolved at build time (typify codegen, via
/// `wire::SharedArtifactEnvelope`) and independently at runtime through the
/// `jsonschema::Registry` this crate builds in `build_contract_validator`. An empty envelope
/// must fail on the *external* schema's own required properties, proving the registry
/// resolved the cross-schema reference rather than treating it as opaque or absent.
#[test]
fn shared_artifact_envelope_resolves_the_external_shared_reference_registry() {
    let value = json!({
        "wireVersion": "ix.shared-reference/2-draft",
        "kind": "artifact",
        "value": {},
    });
    let error = validate_schema_definition("SharedArtifactEnvelope", &value)
        .expect_err("an empty artifact value must fail the external ArtifactRef schema");
    assert_eq!(error.code(), VerificationErrorCode::SchemaViolation);
    let message = error.to_string();
    assert!(
        message.contains("authority") || message.contains("required"),
        "expected a property-level failure from the external ArtifactRef schema, got: {message}"
    );
}

#[test]
fn unknown_schema_definition_is_rejected() {
    let error = validate_schema_definition("NotADefinition", &json!({}))
        .expect_err("an unknown definition name must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::UnknownSchemaDefinition);
}

// Wire format of `format: date-time` fields. Expected strings were recorded from the previous
// `chrono::DateTime<Utc>` representation: offsets normalize to UTC, and the fractional part is
// written as 0, 3, 6 or 9 digits.
const TIMESTAMP_WIRE_CASES: &[(&str, &str)] = &[
    ("2026-01-01T00:00:00Z", "2026-01-01T00:00:00Z"),
    ("2026-01-01T00:00:00z", "2026-01-01T00:00:00Z"),
    ("2026-01-01t00:00:00Z", "2026-01-01T00:00:00Z"),
    ("2026-01-01 00:00:00Z", "2026-01-01T00:00:00Z"),
    ("2026-01-01T00:00:00.5Z", "2026-01-01T00:00:00.500Z"),
    ("2026-01-01T00:00:00.123Z", "2026-01-01T00:00:00.123Z"),
    ("2026-01-01T00:00:00.1234Z", "2026-01-01T00:00:00.123400Z"),
    ("2026-01-01T00:00:00.123456Z", "2026-01-01T00:00:00.123456Z"),
    (
        "2026-01-01T00:00:00.1234567Z",
        "2026-01-01T00:00:00.123456700Z",
    ),
    (
        "2026-01-01T00:00:00.123456789Z",
        "2026-01-01T00:00:00.123456789Z",
    ),
    ("2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00Z"),
    ("2026-01-01T00:00:00.000000000Z", "2026-01-01T00:00:00Z"),
    ("2026-01-01T02:00:00+02:00", "2026-01-01T00:00:00Z"),
    ("2026-01-01T02:00:00-00:00", "2026-01-01T02:00:00Z"),
    ("2026-01-01T00:00:00+00:00", "2026-01-01T00:00:00Z"),
    ("2026-01-01T00:00:00+23:59", "2025-12-31T00:01:00Z"),
    ("2026-12-31T23:59:60Z", "2026-12-31T23:59:60Z"),
    ("2026-12-31T23:59:60.5Z", "2026-12-31T23:59:60.500Z"),
    (
        "2026-12-31T23:59:59.999999999Z",
        "2026-12-31T23:59:59.999999999Z",
    ),
    ("0000-01-01T00:00:00Z", "0000-01-01T00:00:00Z"),
    ("9999-12-31T23:59:59Z", "9999-12-31T23:59:59Z"),
    ("0001-01-01T00:00:00+05:00", "0000-12-31T19:00:00Z"),
];

const TIMESTAMP_REJECTED: &[&str] = &[
    "",
    "not a date",
    "2026-01-01",
    "2026-01-01T00:00:00",
    "2026-01-01T02:00+02:00",
    "2026-13-01T00:00:00Z",
    "2026-02-30T00:00:00Z",
    "2026-01-01T24:00:00Z",
    "2026-01-01T00:00:00+24:00",
    "2026-01-01T00:00:00,5Z",
    "2026-01-01T00:00:00.Z",
    "2026-01-01T00:00:00+02",
    "20260101T000000Z",
    "2026-01-01T00:00:00+02:30:15",
    "12026-01-01T00:00:00Z",
];

#[test]
fn existing_technique_result_fixture_round_trips_byte_identically() {
    let fixture = minimal_technique_result();
    let typed: filament_verification_contracts::wire::TechniqueResult =
        serde_json::from_value(fixture.clone()).expect("fixture decodes into wire types");
    let reserialized = serde_json::to_value(&typed).expect("wire type serializes");
    assert_eq!(reserialized["producedAt"], "2026-01-01T00:00:00Z");
    assert_eq!(
        serde_json::to_string(&reserialized).expect("serializes"),
        serde_json::to_string(&fixture).expect("serializes"),
    );
}

#[test]
fn wire_timestamps_keep_the_chrono_era_wire_format() {
    for (input, expected) in TIMESTAMP_WIRE_CASES {
        let mut fixture = minimal_technique_result();
        fixture["producedAt"] = json!(input);
        let typed: filament_verification_contracts::wire::TechniqueResult =
            serde_json::from_value(fixture).unwrap_or_else(|e| panic!("{input} rejected: {e}"));
        let out = serde_json::to_value(&typed).expect("wire type serializes");
        assert_eq!(out["producedAt"], *expected, "input {input}");
    }
}

#[test]
fn wire_timestamps_reject_invalid_date_times() {
    for input in TIMESTAMP_REJECTED {
        let mut fixture = minimal_technique_result();
        fixture["producedAt"] = json!(input);
        assert!(
            serde_json::from_value::<filament_verification_contracts::wire::TechniqueResult>(
                fixture
            )
            .is_err(),
            "{input:?} must be rejected"
        );
    }
}
