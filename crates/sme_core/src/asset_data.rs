//! Portable authored-data parsing. Hosts acquire bytes via disk, fetch or embedding;
//! parsing is identical across native and WASM. No network or filesystem is used here.
use serde::de::DeserializeOwned;

pub const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;

pub(crate) fn parse_json<T: DeserializeOwned>(bytes: &[u8], kind: &str) -> Result<T, String> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err(format!("{kind} JSON exceeds {MAX_JSON_BYTES}-byte limit"));
    }
    serde_json::from_slice(bytes).map_err(|e| format!("Failed to parse {kind} JSON: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_and_oversized_data_return_errors() {
        assert!(parse_json::<serde_json::Value>(b"{", "scene").is_err());
        assert!(parse_json::<serde_json::Value>(&vec![b' '; MAX_JSON_BYTES + 1], "scene").is_err());
    }
}
