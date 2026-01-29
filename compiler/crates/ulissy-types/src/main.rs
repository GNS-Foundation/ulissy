// ulissy-types/src/main.rs
// ULissy Type Checker CLI - Validate ULissy programs

use std::env;
use std::fs;
use std::process;

use ulissy_types::{check_source, Type};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        run_demo();
        return;
    }

    let filename = &args[1];
    
    match fs::read_to_string(filename) {
        Ok(source) => {
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║            ULissy Type Checker v0.1.0                      ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!();
            println!("Source file: {}", filename);
            println!("{}", "─".repeat(60));
            
            match check_source(&source) {
                Ok(program) => {
                    println!("✓ Type checking successful!");
                    println!("  Statements checked: {}", program.statements.len());
                    println!();
                    print_typed_program(&program);
                }
                Err(e) => {
                    eprintln!("✗ Type errors:\n{}", e);
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
    println!("║            ULissy Type Checker v0.1.0 - DEMO               ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // Demo 1: Valid program
    let valid_source = r#"// Valid ULissy Program
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

    println!("Demo 1: Valid Program");
    println!("{}", "─".repeat(60));
    for (i, line) in valid_source.lines().enumerate() {
        println!("{:3} │ {}", i + 1, line);
    }
    println!("{}", "─".repeat(60));
    println!();
    
    match check_source(valid_source) {
        Ok(program) => {
            println!("✓ Type checking successful!");
            println!("  Statements: {}", program.statements.len());
            println!();
            print_typed_program(&program);
        }
        Err(e) => {
            println!("✗ Type errors:\n{}", e);
        }
    }
    
    println!();
    println!();
    
    // Demo 2: Program with type errors
    let invalid_source = r#"// Invalid ULissy Program - Type Errors
identity me = "not an identity"

every "not a duration" {
    let x = undefined_variable
}

when 42 {
    print("condition should be Bool")
}
"#;

    println!("Demo 2: Invalid Program (showing error detection)");
    println!("{}", "─".repeat(60));
    for (i, line) in invalid_source.lines().enumerate() {
        println!("{:3} │ {}", i + 1, line);
    }
    println!("{}", "─".repeat(60));
    println!();
    
    match check_source(invalid_source) {
        Ok(_) => {
            println!("✓ (unexpectedly passed)");
        }
        Err(e) => {
            println!("✗ Type errors detected (as expected):");
            for line in e.lines() {
                println!("  {}", line);
            }
        }
    }
    
    println!();
    println!("{}", "─".repeat(60));
    println!("Usage: ulissy-check <filename.ul>");
    println!();
    println!("ULissy Type System:");
    println!();
    print_type_hierarchy();
}

fn print_typed_program(program: &ulissy_types::TypedProgram) {
    println!("Typed AST:");
    println!("{}", "─".repeat(60));
    
    for (i, stmt) in program.statements.iter().enumerate() {
        println!();
        println!("Statement {}: {:?}", i + 1, stmt.kind);
    }
}

fn print_type_hierarchy() {
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│                  ULissy Type System                     │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│                                                         │");
    println!("│  PRIMITIVES        IDENTITY          SPATIAL            │");
    println!("│  ──────────        ────────          ───────            │");
    println!("│  Int               Identity          H3Cell             │");
    println!("│  Float             PublicKey         Coordinates        │");
    println!("│  Bool              PrivateKey        Distance           │");
    println!("│  String            Signature         Resolution         │");
    println!("│  Nil               Handle                               │");
    println!("│                                                         │");
    println!("│  TEMPORAL          CRYPTO            ENERGY             │");
    println!("│  ────────          ──────            ──────             │");
    println!("│  Moment            Hash              BatteryLevel       │");
    println!("│  Duration          SharedSecret      PowerMode          │");
    println!("│                    Ciphertext                           │");
    println!("│                                                         │");
    println!("│  GNS PROTOCOL                                           │");
    println!("│  ────────────                                           │");
    println!("│  Breadcrumb        Trajectory        Envelope<T>        │");
    println!("│  GnsRecord         FacetAddress                         │");
    println!("│                                                         │");
    println!("└─────────────────────────────────────────────────────────┘");
}
