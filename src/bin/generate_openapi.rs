use mmo_api::openapi::ApiDoc;
use std::fs;
use std::path::PathBuf;
use utoipa::OpenApi;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut output_path = PathBuf::from("swagger/openapi.json");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_path = PathBuf::from(&args[i + 1]);
                    i += 2;
                    continue;
                }
            }
            "--help" => {
                println!("Generate OpenAPI JSON from utoipa annotations");
                println!();
                println!("USAGE:");
                println!("    cargo run --bin generate_openapi [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!(
                    "    -o, --output <PATH>  Output file path (default: swagger/openapi.json)"
                );
                println!("        --help           Show this help message");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    let openapi_json = ApiDoc::openapi()
        .to_pretty_json()
        .expect("Failed to generate OpenAPI JSON");

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    fs::write(&output_path, &openapi_json).expect("Failed to write OpenAPI JSON");

    println!(
        "✓ OpenAPI specification generated: {}",
        output_path.display()
    );
    println!("  Size: {} bytes", openapi_json.len());
}
