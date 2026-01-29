//! Example: Generating WIT files from Vais types
//!
//! This example demonstrates how to use the wasm_component module
//! to programmatically generate WIT interface definitions.

use vais_codegen::wasm_component::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a WIT package for a calculator application
    let mut package = WitPackage::new("vais", "calculator");
    package.version = Some("0.1.0".to_string());
    package.docs = Some("A simple calculator component".to_string());

    // Define types
    let error_enum = WitEnum {
        name: "error".to_string(),
        cases: vec![
            WitEnumCase {
                name: "division-by-zero".to_string(),
                docs: Some("Attempted to divide by zero".to_string()),
            },
            WitEnumCase {
                name: "overflow".to_string(),
                docs: Some("Arithmetic overflow occurred".to_string()),
            },
        ],
        docs: Some("Calculator errors".to_string()),
    };

    // Create calculator interface
    let mut calculator_interface = WitInterface {
        name: "calculator".to_string(),
        types: vec![WitTypeDefinition::Enum(error_enum)],
        functions: vec![],
        resources: vec![],
        docs: Some("Calculator operations".to_string()),
    };

    // Add arithmetic functions
    calculator_interface.functions.push(WitFunction {
        name: "add".to_string(),
        params: vec![
            WitParam {
                name: "a".to_string(),
                ty: WitType::S32,
            },
            WitParam {
                name: "b".to_string(),
                ty: WitType::S32,
            },
        ],
        results: Some(WitResult::Anon(WitType::S32)),
        docs: Some("Add two numbers".to_string()),
    });

    calculator_interface.functions.push(WitFunction {
        name: "divide".to_string(),
        params: vec![
            WitParam {
                name: "a".to_string(),
                ty: WitType::S32,
            },
            WitParam {
                name: "b".to_string(),
                ty: WitType::S32,
            },
        ],
        results: Some(WitResult::Anon(WitType::Result_ {
            ok: Some(Box::new(WitType::S32)),
            err: Some(Box::new(WitType::Named("error".to_string()))),
        })),
        docs: Some("Divide two numbers".to_string()),
    });

    package.add_interface(calculator_interface);

    // Define a calculator world
    let world = WitWorld {
        name: "calculator-app".to_string(),
        imports: vec![
            WitImport {
                name: "console".to_string(),
                item: WitImportItem::Interface("wasi:cli/stdout@0.2.0".to_string()),
            },
        ],
        exports: vec![
            WitExport {
                name: "calculator".to_string(),
                item: WitExportItem::Interface("calculator".to_string()),
            },
        ],
        docs: Some("Calculator application world".to_string()),
    };

    package.add_world(world);

    // Generate WIT file
    let wit_content = package.to_wit_string();
    println!("Generated WIT:\n{}", wit_content);

    // Save to file
    std::fs::write("calculator.wit", &wit_content)?;
    println!("\nWIT file saved to calculator.wit");

    // Example: Define a more complex record type
    println!("\n--- Complex Type Example ---\n");

    let point_record = WitRecord {
        name: "point".to_string(),
        fields: vec![
            WitField {
                name: "x".to_string(),
                ty: WitType::F64,
                docs: Some("X coordinate".to_string()),
            },
            WitField {
                name: "y".to_string(),
                ty: WitType::F64,
                docs: Some("Y coordinate".to_string()),
            },
        ],
        docs: Some("2D point".to_string()),
    };

    let shape_variant = WitVariant {
        name: "shape".to_string(),
        cases: vec![
            WitVariantCase {
                name: "circle".to_string(),
                ty: Some(WitType::Record("point".to_string())),
                docs: Some("Circle with center point".to_string()),
            },
            WitVariantCase {
                name: "rectangle".to_string(),
                ty: Some(WitType::Tuple(vec![
                    WitType::Record("point".to_string()),
                    WitType::Record("point".to_string()),
                ])),
                docs: Some("Rectangle with two corner points".to_string()),
            },
        ],
        docs: Some("Geometric shape".to_string()),
    };

    let mut geometry_package = WitPackage::new("vais", "geometry");
    geometry_package.version = Some("0.1.0".to_string());

    let mut geometry_interface = WitInterface {
        name: "geometry".to_string(),
        types: vec![
            WitTypeDefinition::Record(point_record),
            WitTypeDefinition::Variant(shape_variant),
        ],
        functions: vec![
            WitFunction {
                name: "area".to_string(),
                params: vec![WitParam {
                    name: "shape".to_string(),
                    ty: WitType::Named("shape".to_string()),
                }],
                results: Some(WitResult::Anon(WitType::F64)),
                docs: Some("Calculate area of a shape".to_string()),
            },
        ],
        resources: vec![],
        docs: Some("Geometric operations".to_string()),
    };

    geometry_package.add_interface(geometry_interface);
    println!("{}", geometry_package.to_wit_string());

    Ok(())
}
