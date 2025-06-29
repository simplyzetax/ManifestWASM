> [!WARNING]  
> Most of the code in this project is from [@ramok0/epic_manifest_parser_rs](https://github.com/ramok0/epic_manifest_parser_rs). This repository provides WebAssembly bindings for that excellent Rust library and adds a reversal function for turning the JSON back into Manifest format

# ManifestWASM

A WebAssembly library for parsing and creating Epic Games Store manifest files.

## Overview

ManifestWASM is a Rust-based WASM library that enables parsing and manipulation of Epic Games Store manifest files in web browsers and JavaScript environments. These manifest files contain metadata about game installations, file chunks, and download information used by the Epic Games Launcher.

## Features

- **Parse manifest files**: Convert binary manifest data to structured JSON
- **Create manifest files**: Generate binary manifest files from JSON data
- **Compression support**: Handle both compressed and uncompressed manifests
- **Version compatibility**: Support for multiple manifest format versions
- **Browser-ready**: WebAssembly bindings for JavaScript/TypeScript projects

## Building

### Prerequisites

- Rust (latest stable)
- `wasm-pack` for building WebAssembly packages

### Build for WebAssembly

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build the WASM package
wasm-pack build --target web --out-dir pkg
```

## Usage

### In JavaScript/TypeScript

```javascript
import init, {
  parse_manifest,
  create_manifest,
} from "./pkg/epic_manifest_wasm.js";

await init();

// Parse a manifest file
const manifestBytes = new Uint8Array(/* your manifest data */);
const jsonResult = parse_manifest(manifestBytes);
const manifest = JSON.parse(jsonResult);

console.log("App Name:", manifest.meta.app_name);
console.log("Build Version:", manifest.meta.build_version);

// Create a manifest file
const manifestJson = JSON.stringify(manifest);
const binaryData = create_manifest(manifestJson);
```

### Manifest Structure

The parsed manifest contains the following main sections:

- **header**: File format metadata and version information
- **meta**: Application metadata (name, version, launch info, etc.)
- **chunk_list**: Information about data chunks for file reconstruction
- **file_list**: List of files with their properties and chunk references
- **custom_fields**: Additional key-value data

## API Reference

### `parse_manifest(manifest_bytes: Uint8Array): string`

Parses binary manifest data and returns a JSON string representation.

**Parameters:**

- `manifest_bytes`: Binary manifest data as Uint8Array

**Returns:** JSON string containing the parsed manifest structure

### `create_manifest(json_string: string): Uint8Array`

Creates binary manifest data from a JSON string.

**Parameters:**

- `json_string`: JSON representation of a manifest

**Returns:** Binary manifest data as Uint8Array

## Error Handling

Both functions return error messages as strings when parsing fails:

```javascript
const result = parse_manifest(invalidData);
if (result.startsWith("Error:")) {
  console.error("Parse failed:", result);
}
```

## Development

```bash
# Run tests
cargo test

# Build native version
cargo build --release

# Format code
cargo fmt

# Check with clippy
cargo clippy
```

## License

This project is available under the terms specified in the repository.
