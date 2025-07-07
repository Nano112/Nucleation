//! Binary for generating all bindings from API definitions
//! 
//! This ensures feature parity across all export targets.

use nucleation::codegen;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    println!("🚀 Nucleation Binding Generator");
    println!("================================");
    
    match args.get(1).map(|s| s.as_str()) {
        Some("check") => {
            println!("🔍 Checking binding parity...\n");
            match codegen::check_bindings_parity() {
                Ok(true) => {
                    println!("\n✅ All bindings are up to date!");
                    process::exit(0);
                },
                Ok(false) => {
                    println!("\n❌ Some bindings are out of date!");
                    println!("💡 Run without arguments to regenerate all bindings.");
                    process::exit(1);
                },
                Err(e) => {
                    eprintln!("❌ Error checking bindings: {}", e);
                    process::exit(1);
                }
            }
        },
        Some("report") => {
            println!("📊 Generating API report...\n");
            match codegen::generate_api_report() {
                Ok(()) => {
                    println!("✅ API report generated successfully!");
                    process::exit(0);
                },
                Err(e) => {
                    eprintln!("❌ Error generating report: {}", e);
                    process::exit(1);
                }
            }
        },
        Some("help") | Some("-h") | Some("--help") => {
            print_help();
            process::exit(0);
        },
        Some(unknown) => {
            eprintln!("❌ Unknown command: {}", unknown);
            print_help();
            process::exit(1);
        },
        None => {
            println!("🔧 Generating all bindings...\n");
            match codegen::generate_all_bindings() {
                Ok(()) => {
                    println!("✅ All bindings generated successfully!");
                    println!("\n📋 Next steps:");
                    println!("  1. Update your imports to use the generated files");
                    println!("  2. Test each binding target to ensure functionality");
                    println!("  3. Commit the generated files to version control");
                    process::exit(0);
                },
                Err(e) => {
                    eprintln!("❌ Error generating bindings: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

fn print_help() {
    println!("Usage: cargo run --bin generate-bindings [COMMAND]");
    println!();
    println!("Commands:");
    println!("  (none)  Generate all binding files");
    println!("  check   Check if all bindings are up to date");
    println!("  report  Generate an API documentation report");
    println!("  help    Show this help message");
    println!();
    println!("This tool generates binding files for multiple targets:");
    println!("  • WASM (WebAssembly) - JavaScript/TypeScript");
    println!("  • Python - PyO3 bindings and type stubs");
    println!("  • FFI - C/C++ compatible interface");
    println!();
    println!("All bindings are generated from a single source of truth in");
    println!("api_definition.rs to ensure feature parity across targets.");
}
