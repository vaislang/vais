use std::path::PathBuf;
use std::process;
use vais_bindgen::{Bindgen, BindgenConfig};

fn print_usage() {
    eprintln!("Usage: vais-bindgen [OPTIONS] <HEADER_FILE>");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -o, --output <FILE>       Write output to FILE (default: stdout)");
    eprintln!("  -l, --library <NAME>      Library name for extern block");
    eprintln!("  -t, --type-map <C=VAIS>   Add custom type mapping (can be used multiple times)");
    eprintln!("  -h, --help                Show this help message");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  vais-bindgen mylib.h");
    eprintln!("  vais-bindgen -o bindings.vais mylib.h");
    eprintln!("  vais-bindgen -l mylib -o bindings.vais mylib.h");
    eprintln!("  vais-bindgen -t size_t=u64 -t custom_t=MyType mylib.h");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut config = BindgenConfig::default();
    let mut output_file: Option<PathBuf> = None;
    let mut input_file: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --output requires an argument");
                    process::exit(1);
                }
                output_file = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "-l" | "--library" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --library requires an argument");
                    process::exit(1);
                }
                config.set_library_name(&args[i + 1]);
                i += 2;
            }
            "-t" | "--type-map" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --type-map requires an argument");
                    process::exit(1);
                }
                let mapping = &args[i + 1];
                if let Some((c_type, vais_type)) = mapping.split_once('=') {
                    config.add_type_mapping(c_type, vais_type);
                } else {
                    eprintln!("Error: invalid type mapping format: {}", mapping);
                    eprintln!("Expected format: C_TYPE=VAIS_TYPE");
                    process::exit(1);
                }
                i += 2;
            }
            arg => {
                if arg.starts_with('-') {
                    eprintln!("Error: unknown option: {}", arg);
                    print_usage();
                    process::exit(1);
                }
                if input_file.is_some() {
                    eprintln!("Error: multiple input files specified");
                    process::exit(1);
                }
                input_file = Some(PathBuf::from(arg));
                i += 1;
            }
        }
    }

    let input_file = match input_file {
        Some(file) => file,
        None => {
            eprintln!("Error: no input file specified");
            print_usage();
            process::exit(1);
        }
    };

    if !input_file.exists() {
        eprintln!("Error: file not found: {}", input_file.display());
        process::exit(1);
    }

    let mut bindgen = Bindgen::with_config(config);

    if let Err(e) = bindgen.header(&input_file) {
        eprintln!("Error parsing header: {}", e);
        process::exit(1);
    }

    match output_file {
        Some(path) => {
            if let Err(e) = bindgen.generate_to_file(&path) {
                eprintln!("Error writing output: {}", e);
                process::exit(1);
            }
            eprintln!("Generated bindings written to: {}", path.display());
        }
        None => {
            match bindgen.generate() {
                Ok(output) => print!("{}", output),
                Err(e) => {
                    eprintln!("Error generating bindings: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
