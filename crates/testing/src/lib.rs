//! Shared contract-test helpers and cross-system validation utilities.

/// Serialize and deserialize a value to verify its JSON wire representation.
pub fn roundtrip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let encoded = serde_json::to_vec(value).expect("test value must serialize to JSON");
    serde_json::from_slice(&encoded).expect("serialized test value must deserialize")
}
