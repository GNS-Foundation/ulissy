// ulissy-types/src/lib.rs
// ULissy Type Checker - Validates AST and enforces type rules
// Version 0.1.0

pub mod types;
pub mod context;
pub mod checker;

pub use types::*;
pub use context::*;
pub use checker::*;

use ulissy_parser::ast;
use std::fmt;

// ============================================================================
// TYPE ERROR
// ============================================================================

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
    pub hint: Option<String>,
    pub span: ast::Span,
}

impl TypeError {
    pub fn new(message: &str, span: ast::Span) -> Self {
        TypeError { message: message.to_string(), hint: None, span }
    }
    
    pub fn with_hint(message: &str, hint: &str, span: ast::Span) -> Self {
        TypeError { message: message.to_string(), hint: Some(hint.to_string()), span }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type error at line {}, column {}: {}", 
            self.span.start_line, self.span.start_column, self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, "\n  hint: {}", hint)?;
        }
        Ok(())
    }
}

impl std::error::Error for TypeError {}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Type check an ULissy program
pub fn check(program: &ast::Program) -> Result<TypedProgram, Vec<TypeError>> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}

/// Type check ULissy source code
pub fn check_source(source: &str) -> Result<TypedProgram, String> {
    let program = ulissy_parser::parse(source)
        .map_err(|e| e.to_string())?;
    
    check(&program).map_err(|errors| {
        errors.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })
}
