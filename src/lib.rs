use serde_json;
use wasm_bindgen::prelude::*;

pub mod error;
pub mod helper;
pub mod manifest;
pub mod reader;
pub mod writer;

pub type ParseResult<T> = Result<T, error::ParseError>;

#[wasm_bindgen]
pub fn parse_manifest(manifest_bytes: &[u8]) -> String {
    // Add some basic validation
    if manifest_bytes.is_empty() {
        return "Error: Empty manifest data".to_string();
    }

    if manifest_bytes.len() < 32 {
        return format!(
            "Error: Manifest data too small ({} bytes, need at least 32)",
            manifest_bytes.len()
        );
    }

    let parser = manifest::FManifestParser::new(manifest_bytes);
    match parser.parse() {
        Ok(parsed) => match serde_json::to_string_pretty(&parsed) {
            Ok(json) => json,
            Err(e) => format!("Failed to serialize to JSON: {:?}", e),
        },
        Err(e) => {
            // Provide more detailed error information
            format!(
                "Failed to parse manifest (size: {} bytes): {:?}",
                manifest_bytes.len(),
                e
            )
        }
    }
}

#[wasm_bindgen]
pub fn create_manifest(json_string: &str) -> Vec<u8> {
    match serde_json::from_str::<manifest::FManifest>(json_string) {
        Ok(manifest) => match manifest.serialize() {
            Ok(bytes) => bytes,
            Err(e) => {
                // Return empty vector on serialization error
                // In a real implementation, you might want to handle this differently
                eprintln!("Failed to serialize manifest: {:?}", e);
                Vec::new()
            }
        },
        Err(e) => {
            // Return empty vector on JSON parsing error
            // In a real implementation, you might want to handle this differently
            eprintln!("Failed to parse JSON: {:?}", e);
            Vec::new()
        }
    }
}
