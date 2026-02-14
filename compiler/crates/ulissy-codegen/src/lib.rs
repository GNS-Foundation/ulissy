// ulissy-codegen/src/lib.rs
// ULissy Code Generator - Transforms typed AST to Rust code
// Version 0.1.0

pub mod generator;
pub mod rust_emitter;

pub use generator::*;
pub use rust_emitter::*;

use std::fmt;
use ulissy_types::TypedProgram;

// ============================================================================
// CODE GENERATION ERROR
// ============================================================================

#[derive(Debug)]
pub struct CodeGenError {
    pub message: String,
}

impl CodeGenError {
    pub fn new(message: &str) -> Self {
        CodeGenError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Code generation error: {}", self.message)
    }
}

impl std::error::Error for CodeGenError {}

// ============================================================================
// GENERATED OUTPUT
// ============================================================================

#[derive(Debug)]
pub struct GeneratedCode {
    /// The main Rust source file
    pub main_rs: String,
    /// Cargo.toml for the generated project
    pub cargo_toml: String,
    /// Project name
    pub name: String,
}

impl GeneratedCode {
    pub fn new(name: &str, main_rs: String, cargo_toml: String) -> Self {
        GeneratedCode {
            name: name.to_string(),
            main_rs,
            cargo_toml,
        }
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Generate Rust code from a typed ULissy program
pub fn generate(program: &TypedProgram, project_name: &str) -> Result<GeneratedCode, CodeGenError> {
    let mut generator = CodeGenerator::new(project_name);
    generator.generate(program)
}

/// Compile ULissy source code directly to Rust
pub fn compile(source: &str, project_name: &str) -> Result<GeneratedCode, String> {
    // Parse
    let ast = ulissy_parser::parse(source).map_err(|e| format!("Parse error: {}", e))?;

    // Type check
    let typed = ulissy_types::check(&ast).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    // Generate
    generate(&typed, project_name).map_err(|e| e.to_string())
}
