// ulissy/src/main.rs
// ULissy CLI - The Unified Compiler Interface
// A Programming Language for Moving Machines
// Version 0.1.0

use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

// ============================================================================
// CLI DEFINITION
// ============================================================================

#[derive(Parser)]
#[command(name = "ulissy")]
#[command(author = "GNS Foundation")]
#[command(version = "0.1.0")]
#[command(about = "ULissy - A Programming Language for Moving Machines", long_about = None)]
#[command(after_help = "EXAMPLES:
    ulissy new my-app           Create a new ULissy project
    ulissy build src/main.ul    Compile ULissy to Rust
    ulissy run src/main.ul      Compile and execute
    ulissy check src/main.ul    Type check without compiling

LEARN MORE:
    https://github.com/cayerbe/ulissy_program

\"The journey is the proof.\" - ULissy")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new ULissy project
    New {
        /// Project name
        name: String,
    },

    /// Compile ULissy source to Rust
    Build {
        /// Source file (.ul)
        file: String,

        /// Output directory
        #[arg(short, long, default_value = "target/ulissy")]
        output: String,

        /// Project name for generated Cargo.toml
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Compile and run ULissy program
    Run {
        /// Source file (.ul)
        file: String,
    },

    /// Type check without generating code
    Check {
        /// Source file (.ul)
        file: String,
    },

    /// Show lexer tokens (debug)
    Lex {
        /// Source file (.ul)
        file: String,
    },

    /// Show parsed AST (debug)
    Parse {
        /// Source file (.ul)
        file: String,
    },

    /// Format ULissy source code
    Fmt {
        /// Source file (.ul)
        file: String,
    },

    /// Show version and compiler info
    Info,
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Build { file, output, name } => cmd_build(&file, &output, name),
        Commands::Run { file } => cmd_run(&file),
        Commands::Check { file } => cmd_check(&file),
        Commands::Lex { file } => cmd_lex(&file),
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Fmt { file } => cmd_fmt(&file),
        Commands::Info => cmd_info(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        process::exit(1);
    }
}

// ============================================================================
// COMMANDS
// ============================================================================

fn cmd_new(name: &str) -> Result<(), String> {
    print_header();
    println!("{} new project '{}'", "Creating".green().bold(), name);
    println!();

    // Create directory structure
    let project_dir = Path::new(name);
    if project_dir.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }

    fs::create_dir_all(project_dir.join("src"))
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Create ulissy.toml
    let ulissy_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]

[dependencies]
# Add ULissy dependencies here

[build]
target = ["ios", "android", "desktop"]
"#,
        name
    );

    fs::write(project_dir.join("ulissy.toml"), ulissy_toml)
        .map_err(|e| format!("Failed to write ulissy.toml: {}", e))?;

    // Create main.ul
    let main_ul = r#"// Welcome to ULissy!
// A Programming Language for Moving Machines

identity me = Keychain.primary

// Collect breadcrumbs to prove humanity
every 10.minutes when battery > 20 {
    let crumb = breadcrumb(
        cell: here.h3(10),
        context: sensors.digest,
        previous: me.trajectory.last
    )
    me.trajectory.append(crumb)
}

// Ready to claim handle after 100 breadcrumbs
when me.trajectory.count >= 100 {
    print("Ready to claim @handle!")
}
"#;

    fs::write(project_dir.join("src/main.ul"), main_ul)
        .map_err(|e| format!("Failed to write main.ul: {}", e))?;

    // Create README.md
    let readme = format!(
        r#"# {}

An ULissy project for moving machines.

## Getting Started

```bash
# Type check
ulissy check src/main.ul

# Build
ulissy build src/main.ul

# Run
ulissy run src/main.ul
```

## Learn More

- [ULissy Documentation](https://github.com/cayerbe/ulissy_program)
- [GNS Protocol](https://gns.foundation)
"#,
        name
    );

    fs::write(project_dir.join("README.md"), readme)
        .map_err(|e| format!("Failed to write README.md: {}", e))?;

    // Create .gitignore
    let gitignore = r#"# ULissy build artifacts
target/
*.rs.bk

# IDE
.idea/
.vscode/
*.swp
"#;

    fs::write(project_dir.join(".gitignore"), gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {}", e))?;

    println!("{}", "✓ Project created successfully!".green());
    println!();
    println!("  {}", format!("cd {}", name).cyan());
    println!("  {}", "ulissy check src/main.ul".cyan());
    println!("  {}", "ulissy build src/main.ul".cyan());
    println!();

    Ok(())
}

fn cmd_build(file: &str, output: &str, name: Option<String>) -> Result<(), String> {
    print_header();
    println!("{} {}", "Compiling".green().bold(), file);
    println!();

    // Read source
    let source =
        fs::read_to_string(file).map_err(|e| format!("Failed to read '{}': {}", file, e))?;

    // Derive project name from file if not provided
    let project_name = name.unwrap_or_else(|| {
        Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("ulissy_app")
            .to_string()
    });

    // Compile
    println!("  {} Lexing...", "→".blue());
    println!("  {} Parsing...", "→".blue());
    println!("  {} Type checking...", "→".blue());
    println!("  {} Generating Rust...", "→".blue());

    let generated = ulissy_codegen::compile(&source, &project_name)
        .map_err(|e| format!("Compilation failed:\n{}", e))?;

    // Create output directory
    let output_path = Path::new(output);
    fs::create_dir_all(output_path.join("src"))
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Write generated files
    fs::write(output_path.join("src/main.rs"), &generated.main_rs)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    fs::write(output_path.join("Cargo.toml"), &generated.cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    println!();
    println!("{}", "✓ Build successful!".green().bold());
    println!();
    println!("  Output: {}", output_path.display().to_string().cyan());
    println!("  Files:");
    println!("    {} src/main.rs", "•".blue());
    println!("    {} Cargo.toml", "•".blue());
    println!();
    println!("  To compile to native:");
    println!(
        "    {}",
        format!("cd {} && cargo build --release", output).cyan()
    );
    println!();

    Ok(())
}

fn cmd_run(file: &str) -> Result<(), String> {
    print_header();
    println!("{} {}", "Running".green().bold(), file);
    println!();

    // First build
    let output = "target/ulissy/run";
    let project_name = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("ulissy_app")
        .to_string();

    // Read and compile
    let source =
        fs::read_to_string(file).map_err(|e| format!("Failed to read '{}': {}", file, e))?;

    let generated = ulissy_codegen::compile(&source, &project_name)
        .map_err(|e| format!("Compilation failed:\n{}", e))?;

    // Write files
    let output_path = Path::new(output);
    fs::create_dir_all(output_path.join("src"))
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    fs::write(output_path.join("src/main.rs"), &generated.main_rs)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    fs::write(output_path.join("Cargo.toml"), &generated.cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    println!("  {} Compiled to Rust", "✓".green());
    println!("  {} Running cargo build...", "→".blue());
    println!();

    // Run cargo
    let status = Command::new("cargo")
        .args(["run", "--manifest-path", &format!("{}/Cargo.toml", output)])
        .status()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;

    if !status.success() {
        return Err("Cargo build failed".to_string());
    }

    Ok(())
}

fn cmd_check(file: &str) -> Result<(), String> {
    print_header();
    println!("{} {}", "Checking".green().bold(), file);
    println!();

    let source =
        fs::read_to_string(file).map_err(|e| format!("Failed to read '{}': {}", file, e))?;

    // Parse
    println!("  {} Parsing...", "→".blue());
    let ast = ulissy_parser::parse(&source).map_err(|e| format!("Parse error: {}", e))?;

    println!("    {} statements parsed", ast.statements.len());

    // Type check
    println!("  {} Type checking...", "→".blue());
    let typed = ulissy_types::check(&ast).map_err(|errors| {
        errors
            .iter()
            .map(|e| format!("  {}", e))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    println!("    {} statements validated", typed.statements.len());

    println!();
    println!("{}", "✓ No errors found!".green().bold());
    println!();

    Ok(())
}

fn cmd_lex(file: &str) -> Result<(), String> {
    print_header();
    println!("{} {}", "Lexing".green().bold(), file);
    println!();

    let source =
        fs::read_to_string(file).map_err(|e| format!("Failed to read '{}': {}", file, e))?;

    let tokens = ulissy_lexer::tokenize(&source).map_err(|e| format!("Lexer error: {}", e))?;

    println!("{} tokens:", tokens.len().to_string().cyan());
    println!("{}", "─".repeat(60));

    for token in &tokens {
        println!(
            "  [{:3}:{:<3}] {}",
            token.line,
            token.column,
            format!("{:?}", token.kind).blue()
        );
    }

    println!();
    Ok(())
}

fn cmd_parse(file: &str) -> Result<(), String> {
    print_header();
    println!("{} {}", "Parsing".green().bold(), file);
    println!();

    let source =
        fs::read_to_string(file).map_err(|e| format!("Failed to read '{}': {}", file, e))?;

    let ast = ulissy_parser::parse(&source).map_err(|e| format!("Parse error: {}", e))?;

    println!("{}", "Abstract Syntax Tree:".cyan());
    println!("{}", "─".repeat(60));
    println!("{}", ast.pretty_print());

    Ok(())
}

fn cmd_fmt(file: &str) -> Result<(), String> {
    print_header();
    println!("{} {}", "Formatting".green().bold(), file);
    println!();

    // TODO: Implement formatter
    println!("{}", "Formatter not yet implemented.".yellow());
    println!("Coming in ULissy 0.2.0");
    println!();

    Ok(())
}

fn cmd_info() -> Result<(), String> {
    print_logo();
    println!();
    println!("  {}: 0.1.0", "Version".bold());
    println!("  {}: Rust → gns-crypto-core", "Target".bold());
    println!("  {}: iOS, Android, Desktop, WASM", "Platform".bold());
    println!();
    println!("  {}", "Components:".bold());
    println!("    {} ulissy-lexer    Tokenization", "•".blue());
    println!("    {} ulissy-parser   AST generation", "•".blue());
    println!("    {} ulissy-types    Type checking", "•".blue());
    println!("    {} ulissy-codegen  Rust emission", "•".blue());
    println!();
    println!("  {}", "GNS Protocol Types:".bold());
    println!("    Identity, Handle, Breadcrumb, Trajectory,");
    println!("    H3Cell, Duration, Distance, Envelope<T>");
    println!();
    println!(
        "  {}: https://github.com/cayerbe/ulissy_program",
        "Repository".bold()
    );
    println!("  {}: MIT", "License".bold());
    println!();
    println!("  {}", "\"The journey is the proof.\"".italic());
    println!();

    Ok(())
}

// ============================================================================
// HELPERS
// ============================================================================

fn print_header() {
    println!();
    println!("{}", "╭─────────────────────────────────────╮".blue());
    println!("{}", "│  ULissy Compiler v0.1.0             │".blue());
    println!("{}", "│  A Language for Moving Machines     │".blue());
    println!("{}", "╰─────────────────────────────────────╯".blue());
    println!();
}

fn print_logo() {
    println!();
    println!(
        "{}",
        r#"
    ██╗   ██╗██╗     ██╗███████╗███████╗██╗   ██╗
    ██║   ██║██║     ██║██╔════╝██╔════╝╚██╗ ██╔╝
    ██║   ██║██║     ██║███████╗███████╗ ╚████╔╝
    ██║   ██║██║     ██║╚════██║╚════██║  ╚██╔╝
    ╚██████╔╝███████╗██║███████║███████║   ██║
     ╚═════╝ ╚══════╝╚═╝╚══════╝╚══════╝   ╚═╝
    "#
        .cyan()
    );
    println!(
        "    {}",
        "A Programming Language for Moving Machines".bold()
    );
    println!(
        "    {}",
        "Built on GNS Protocol • Proof-of-Trajectory".dimmed()
    );
}
