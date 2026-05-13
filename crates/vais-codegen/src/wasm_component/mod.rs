//! WebAssembly Component Model support (wasi-preview2)
//!
//! This module provides support for the WebAssembly Component Model, which enables
//! composition and interoperability between WebAssembly modules using WIT (WebAssembly
//! Interface Types).
//!
//! Key features:
//! - WIT type definitions (string, list, record, variant, enum, flags, etc.)
//! - Component interface generation from Vais module signatures
//! - Component linking configuration
//! - wasi-preview2 integration
//! - wasm-bindgen JavaScript/TypeScript binding generation
//! - WASI manifest management for imports/exports
//!
//! # Example
//!
//! ```rust
//! use vais_codegen::wasm_component::{
//!     WasmBindgenGenerator, WasiManifest, ComponentLinkConfig,
//!     WitFunction, WitParam, WitResult, WitType
//! };
//!
//! // Create a WASI manifest
//! let mut manifest = WasiManifest::new();
//! manifest.add_import("wasi:filesystem/types");
//! manifest.add_import("wasi:cli/stdio");
//! manifest.add_export("main", &WitType::S32);
//!
//! // Generate WIT content
//! let wit_content = manifest.to_wit_string();
//!
//! // Generate JavaScript bindings
//! let generator = WasmBindgenGenerator::new("my_module");
//! let functions = vec![
//!     WitFunction {
//!         name: "add".to_string(),
//!         params: vec![
//!             WitParam { name: "a".to_string(), ty: WitType::S32 },
//!             WitParam { name: "b".to_string(), ty: WitType::S32 },
//!         ],
//!         results: Some(WitResult::Anon(WitType::S32)),
//!         docs: Some("Add two numbers".to_string()),
//!     }
//! ];
//! let js_bindings = generator.generate_js_bindings(&functions);
//! let ts_declarations = generator.generate_ts_declarations(&functions);
//!
//! // Configure component linking with WASI
//! let config = ComponentLinkConfig::new()
//!     .with_wasi_manifest(manifest)
//!     .with_adapter("wasi_snapshot_preview1.wasm");
//! ```
//!
//! References:
//! - Component Model spec: <https://github.com/WebAssembly/component-model>
//! - WIT IDL spec: <https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md>
//! - wasm-bindgen: <https://rustwasm.github.io/wasm-bindgen/>

mod bindgen;
mod conversion;
mod interface;
mod link_config;
mod package;
mod serialization;
mod types;
mod wasi;

// Re-export public types
pub use bindgen::WasmBindgenGenerator;
pub use conversion::vais_type_to_wit;
pub use interface::{
    WitExport, WitExportItem, WitImport, WitImportItem, WitInterface, WitMethodKind, WitResource,
    WitResourceMethod, WitWorld,
};
pub use link_config::ComponentLinkConfig;
pub use package::WitPackage;
pub use serialization::WasmSerializer;
pub use types::{
    WitEnum, WitEnumCase, WitField, WitFlags, WitFunction, WitParam, WitRecord, WitResult, WitType,
    WitTypeDefinition, WitVariant, WitVariantCase,
};
pub use wasi::WasiManifest;

#[cfg(test)]
mod tests;
