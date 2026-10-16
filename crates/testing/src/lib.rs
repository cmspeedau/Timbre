//! Cross-system test support for Timbre.
//!
//! Phase 0 provides the contract round-trip helper used by the contract tests.
//! Property, performance, and audio-fixture support land with the
//! [Testing Roadmap](../../Testing%20Roadmap.md).

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Serializes `value` to JSON and reads it back, asserting the contract is
/// unchanged, and returns the decoded value.
///
/// # Panics
///
/// Panics if serialization or deserialization fails, or if the value changes
/// across the round trip.
pub fn roundtrip<T>(value: &T) -> T
where
    T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("contract should serialize");
    let decoded: T = serde_json::from_str(&json).expect("contract should deserialize");
    assert_eq!(value, &decoded, "contract should survive a JSON round trip");
    decoded
}
