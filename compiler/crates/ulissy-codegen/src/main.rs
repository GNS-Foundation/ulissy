// ulissy-codegen/src/main.rs
// ULissy Code Generator CLI - Compile ULissy to Rust

use std::env;
use std::fs;
use std::process;

use ulissy_codegen::compile;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        run_demo();
        return;
    }

    let filename = &args[1];
    let project_name = args.get(2)
        .map(|s| s.as_str())
        .unwrap_or("ulissy_app");
    
    match fs::read_to_string(filename) {
        Ok(source) => {
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║          ULissy Code Generator v0.1.0                      ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!();
            println!("Source file: {}", filename);
            println!("Project name: {}", project_name);
            println!("{}", "─".repeat(60));
            
            match compile(&source, project_name) {
                Ok(generated) => {
                    println!("✓ Code generation successful!");
                    println!();
                    println!("Generated main.rs:");
                    println!("{}", "═".repeat(60));
                    println!("{}", generated.main_rs);
                    println!("{}", "═".repeat(60));
                    println!();
                    println!("Generated Cargo.toml:");
                    println!("{}", "═".repeat(60));
                    println!("{}", generated.cargo_toml);
                    println!("{}", "═".repeat(60));
                }
                Err(e) => {
                    eprintln!("✗ Compilation error:\n{}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    }
}

fn run_demo() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          ULissy Code Generator v0.1.0 - DEMO               ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    let demo_source = r#"// ULissy Demo Program
identity me = Keychain.primary

every 10.minutes when battery > 20 {
    let crumb = breadcrumb(
        cell: here.h3(10),
        context: sensors.digest,
        previous: me.trajectory.last
    )
    me.trajectory.append(crumb)
}

when me.trajectory.count >= 100 {
    print("Ready to claim @handle!")
}

send to @alice {
    message: "Meeting at 3pm"
}
"#;

    println!("ULissy Source Code:");
    println!("{}", "─".repeat(60));
    for (i, line) in demo_source.lines().enumerate() {
        println!("{:3} │ {}", i + 1, line);
    }
    println!("{}", "─".repeat(60));
    println!();
    
    match compile(demo_source, "gns_breadcrumb_app") {
        Ok(generated) => {
            println!("✓ Compilation successful!");
            println!();
            println!("{}",  "═".repeat(60));
            println!("                    GENERATED RUST CODE");
            println!("{}", "═".repeat(60));
            println!();
            println!("{}", generated.main_rs);
            println!("{}", "═".repeat(60));
            println!();
            println!("{}",  "═".repeat(60));
            println!("                   GENERATED Cargo.toml");
            println!("{}", "═".repeat(60));
            println!();
            println!("{}", generated.cargo_toml);
            println!("{}", "═".repeat(60));
            
            println!();
            println!("The ULissy compiler transforms high-level GNS protocol code");
            println!("into production-ready Rust that calls gns-crypto-core.");
            println!();
            println!("Pipeline: ULissy → Lexer → Parser → Types → CodeGen → Rust");
        }
        Err(e) => {
            eprintln!("✗ Compilation error:\n{}", e);
            process::exit(1);
        }
    }
    
    println!();
    println!("{}", "─".repeat(60));
    println!("Usage: ulissy-gen <filename.ul> [project_name]");
}
