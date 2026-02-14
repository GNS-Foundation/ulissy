// ulissy-codegen/src/generator.rs
// ULissy Code Generator - Main Generation Logic
// Version 0.1.2 - COMPREHENSIVE FIX: 67 remaining errors
//
// Changes from v0.1.1:
//   FIX A: Box<dyn Any> → omit type annotation for Any/Unknown (let Rust infer)
//   FIX B: String literals → strip .to_string() for &str params (facet, starts_with)
//   FIX C: to_h3 → add `as u8` cast and `?` operator
//   FIX D: sign/signed → add `&` reference and `?` for builder pattern
//   FIX E: Function calls → keep original camelCase names, add .await?
//   FIX F: Division → cast i64 operands to f64
//   FIX G: VarDecl → move to main body (not static mut)
//   FIX H: Enum variant lookup → track variant→enum mapping
//   FIX I: Nil init for String → use String::new()
//   FIX J: Keychain::facet in Call handler → strip .to_string(), add ?

use crate::{CodeGenError, GeneratedCode, RustEmitter};
use std::collections::{HashMap, HashSet};
use ulissy_parser::ast::{self, BinaryOp, Literal, UnaryOp};
use ulissy_types::{
    Type, TypedComputedBody, TypedConfigField, TypedEnumVariant, TypedExpr, TypedExprKind,
    TypedInterpolatedPart, TypedMatchCase, TypedObjectField, TypedParam, TypedProgram,
    TypedStatement, TypedStatementKind,
};

// ============================================================================
// CODE GENERATOR
// ============================================================================

pub struct CodeGenerator {
    project_name: String,
    emitter: RustEmitter,
    // FIX H: Track enum variant → enum name mapping
    enum_variant_map: HashMap<String, String>,
    // Track module-level static variables
    static_vars: HashSet<String>,
    // Track function signatures for call-site type casting
    fn_signatures: HashMap<String, Vec<Type>>,
}

impl CodeGenerator {
    pub fn new(project_name: &str) -> Self {
        CodeGenerator {
            project_name: project_name.to_string(),
            emitter: RustEmitter::new(),
            enum_variant_map: HashMap::new(),
            static_vars: HashSet::new(),
            fn_signatures: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &TypedProgram) -> Result<GeneratedCode, CodeGenError> {
        let main_rs = self.generate_main(program)?;
        let cargo_toml = self.generate_cargo_toml();
        Ok(GeneratedCode::new(&self.project_name, main_rs, cargo_toml))
    }

    fn generate_main(&mut self, program: &TypedProgram) -> Result<String, CodeGenError> {
        self.emitter.emit_file_header();
        self.emitter
            .emit_line("#![allow(static_mut_refs, non_snake_case)]");
        self.emitter.emit_imports();
        self.emitter.newline();
        self.emitter.newline();

        // ====================================================================
        // PRE-PASS: Build enum variant lookup table (FIX H)
        // ====================================================================
        for stmt in &program.statements {
            if let TypedStatementKind::EnumDecl { name, variants, .. } = &stmt.kind {
                for variant in variants {
                    let pascal = to_pascal_case(&variant.name);
                    self.enum_variant_map.insert(pascal, name.clone());
                }
            }
        }

        // ====================================================================
        // PASS 1: Generate module statements (outside main)
        // ====================================================================

        for stmt in &program.statements {
            match &stmt.kind {
                TypedStatementKind::TypeDecl { name, fields } => {
                    self.generate_type_decl(name, fields)?;
                }
                TypedStatementKind::EnumDecl {
                    name,
                    type_params,
                    variants,
                } => {
                    self.generate_enum_decl(name, type_params, variants)?;
                }
                TypedStatementKind::FnDecl { .. } => {
                    self.generate_statement(stmt)?;
                }
                TypedStatementKind::ConfigBlock { fields } => {
                    self.generate_config_static(fields)?;
                }
                // VarDecl at module level: generate as static mut (accessible from functions)
                TypedStatementKind::VarDecl {
                    name,
                    inferred_type,
                    init,
                    ..
                } => {
                    self.static_vars.insert(name.clone());
                    self.generate_static_var(name, inferred_type, init)?;
                }
                _ => {}
            }
        }

        self.emitter.newline();

        // ====================================================================
        // PASS 2: Generate main execution logic
        // ====================================================================

        self.emitter.emit_line("#[tokio::main]");
        self.emitter
            .emit_line("async fn main() -> Result<(), GnsError> {");
        self.emitter.indent();

        for stmt in &program.statements {
            match &stmt.kind {
                TypedStatementKind::TypeDecl { .. }
                | TypedStatementKind::EnumDecl { .. }
                | TypedStatementKind::FnDecl { .. }
                | TypedStatementKind::ConfigBlock { .. } => {
                    continue;
                }
                // FIX G: VarDecl now generated inside main as let mut
                _ => {
                    // Skip VarDecls already generated as statics in Pass 1
                    if let TypedStatementKind::VarDecl { name, .. } = &stmt.kind {
                        if self.static_vars.contains(name) {
                            continue;
                        }
                    }
                    self.generate_statement(stmt)?;
                }
            }
        }

        self.emitter.newline();
        self.emitter.emit_line("Ok(())");
        self.emitter.dedent();
        self.emitter.emit_line("}");

        Ok(self.emitter.output())
    }

    fn generate_cargo_toml(&self) -> String {
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

# AUTO-GENERATED BY ULISSY COMPILER
# Do not edit manually

[dependencies]
gns-runtime = {{ path = "../../../gns-runtime" }}
gns-search = {{ path = "../../../gns-search" }}
tokio = {{ version = "1", features = ["full"] }}
thiserror = "1.0"

# GNS Protocol dependencies
ed25519-dalek = "2"
x25519-dalek = "2"
chacha20poly1305 = "0.10"
h3o = "0.6"
sha2 = "0.10"
hkdf = "0.12"
hex = "0.4"
"#,
            self.project_name
        )
    }

    // ========================================================================
    // STATEMENT GENERATION
    // ========================================================================

    fn generate_statement(&mut self, stmt: &TypedStatement) -> Result<(), CodeGenError> {
        match &stmt.kind {
            TypedStatementKind::IdentityDecl { name, init } => {
                self.generate_identity_decl(name, init)
            }
            TypedStatementKind::LetDecl {
                name,
                inferred_type,
                init,
                ..
            } => self.generate_let_decl(name, inferred_type, init),
            TypedStatementKind::VarDecl {
                name,
                inferred_type,
                init,
                ..
            } => self.generate_var_decl(name, inferred_type, init),
            TypedStatementKind::EveryBlock {
                interval,
                condition,
                body,
            } => self.generate_every_block(interval, condition, body),
            TypedStatementKind::WhenBlock { condition, body } => {
                self.generate_when_block(condition, body)
            }
            TypedStatementKind::SendStatement { recipient, fields } => {
                self.generate_send_statement(recipient, fields)
            }
            TypedStatementKind::EnumDecl {
                name,
                type_params,
                variants,
            } => self.generate_enum_decl(name, type_params, variants),
            TypedStatementKind::ConfigBlock { fields } => self.generate_config_block(fields),
            TypedStatementKind::TypeDecl { name, fields } => self.generate_type_decl(name, fields),
            TypedStatementKind::ComputedPropertyDecl {
                name,
                inferred_type,
                body,
            } => self.generate_computed_property(name, inferred_type, body),
            TypedStatementKind::FnDecl {
                name,
                params,
                return_type,
                body,
            } => self.generate_fn_decl(name, params, return_type, body),
            TypedStatementKind::ReturnStatement(expr) => self.generate_return_statement(expr),
            TypedStatementKind::IfStatement {
                condition,
                then_block,
                else_block,
            } => self.generate_if_statement(condition, then_block, else_block),
            TypedStatementKind::IfLetStatement {
                binding,
                binding_type: _,
                value,
                then_block,
                else_block,
            } => self.generate_if_let_statement(binding, value, then_block, else_block),
            TypedStatementKind::ForStatement {
                item_name,
                collection,
                body,
            } => self.generate_for_statement(item_name, collection, body),
            TypedStatementKind::MatchStatement { expr, cases } => {
                self.generate_match_statement(expr, cases)
            }
            TypedStatementKind::AfterBlock { delay, body } => {
                self.generate_after_block(delay, body)
            }
            TypedStatementKind::ExpressionStatement(expr) => {
                let code = self.generate_expression(expr)?;
                if code == "None" {
                    self.emitter.emit_line("let _ = Option::<()>::None;");
                } else {
                    self.emitter.emit_line(&format!("{};", code));
                }
                Ok(())
            }
        }
    }

    fn generate_identity_decl(&mut self, name: &str, init: &TypedExpr) -> Result<(), CodeGenError> {
        let init_code = self.generate_expression(init)?;
        // Don't add ? if expression already ends with ? (prevents double ??)
        if init_code.ends_with('?') {
            self.emitter
                .emit_line(&format!("let {} = {};", name, init_code));
        } else {
            self.emitter
                .emit_line(&format!("let {} = {}?;", name, init_code));
        }
        self.emitter.newline();
        Ok(())
    }

    fn generate_let_decl(
        &mut self,
        name: &str,
        ty: &Type,
        init: &TypedExpr,
    ) -> Result<(), CodeGenError> {
        let init_code = self.generate_expression(init)?;

        // FIX A: When type is Any/Unknown, omit annotation and let Rust infer
        let type_annotation = if matches!(ty, Type::Any | Type::Unknown) {
            "auto".to_string()
        } else {
            self.type_to_rust(ty)
        };

        // FIX I: Handle Nil init for non-Optional types
        let final_init = if init_code == "None" && !matches!(ty, Type::Optional(_) | Type::Nil) {
            match ty {
                Type::String => "String::new()".to_string(),
                Type::Int => "0".to_string(),
                Type::Float => "0.0".to_string(),
                Type::Bool => "false".to_string(),
                _ => init_code,
            }
        } else {
            init_code
        };

        if type_annotation != "auto" {
            self.emitter.emit_line(&format!(
                "let {}: {} = {};",
                name, type_annotation, final_init
            ));
        } else {
            self.emitter
                .emit_line(&format!("let {} = {};", name, final_init));
        }
        Ok(())
    }

    fn generate_var_decl(
        &mut self,
        name: &str,
        ty: &Type,
        init: &Option<TypedExpr>,
    ) -> Result<(), CodeGenError> {
        // FIX A: When type is Any/Unknown, omit annotation
        let type_annotation = if matches!(ty, Type::Any | Type::Unknown) {
            "auto".to_string()
        } else {
            self.type_to_rust(ty)
        };

        if let Some(init_expr) = init {
            let init_code = self.generate_expression(init_expr)?;

            if type_annotation != "auto" {
                self.emitter.emit_line(&format!(
                    "let mut {}: {} = {};",
                    name, type_annotation, init_code
                ));
            } else {
                self.emitter
                    .emit_line(&format!("let mut {} = {};", name, init_code));
            }
        } else if type_annotation != "auto" {
            self.emitter
                .emit_line(&format!("let mut {}: {};", name, type_annotation));
        } else {
            self.emitter
                .emit_line(&format!("let mut {} = Default::default();", name));
        }
        Ok(())
    }

    fn generate_static_var(
        &mut self,
        name: &str,
        ty: &Type,
        init: &Option<TypedExpr>,
    ) -> Result<(), CodeGenError> {
        let type_annotation = self.type_to_rust(ty);

        let init_code = if let Some(init_expr) = init {
            self.generate_expression(init_expr)?
        } else {
            match ty {
                Type::Int => "0".to_string(),
                Type::Float => "0.0".to_string(),
                Type::Bool => "false".to_string(),
                Type::String => "String::new()".to_string(),
                _ => "Default::default()".to_string(),
            }
        };

        self.emitter.emit_line(&format!(
            "static mut {}: {} = {};",
            name, type_annotation, init_code
        ));
        Ok(())
    }

    fn generate_config_static(&mut self, fields: &[TypedConfigField]) -> Result<(), CodeGenError> {
        self.emitter.emit_comment("ULissy: Config struct");
        self.emitter.emit_line("#[derive(Debug, Clone, PartialEq)]");
        self.emitter.emit_line("pub struct Config {");
        self.emitter.indent();
        for field in fields {
            let rust_type = self.type_to_rust(&field.value.ty);
            let field_name = to_snake_case(&field.name);
            self.emitter
                .emit_line(&format!("pub {}: {},", field_name, rust_type));
        }
        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        self.emitter.emit_comment("ULissy: Global config");
        self.emitter.emit_line("static CONFIG: Config = Config {");
        self.emitter.indent();
        for field in fields {
            let val = self.generate_expression(&field.value)?;
            let field_name = to_snake_case(&field.name);
            self.emitter.emit_line(&format!("{}: {},", field_name, val));
        }
        self.emitter.dedent();
        self.emitter.emit_line("};");
        self.emitter.newline();
        Ok(())
    }

    fn generate_every_block(
        &mut self,
        interval: &TypedExpr,
        condition: &Option<TypedExpr>,
        body: &[TypedStatement],
    ) -> Result<(), CodeGenError> {
        let interval_code = self.generate_expression(interval)?;

        self.emitter
            .emit_comment("ULissy: every block - scheduled task");
        self.emitter.emit_line(&format!(
            "gns_runtime::schedule_every({}, move || {{",
            interval_code
        ));
        self.emitter.indent();

        if let Some(cond) = condition {
            let cond_code = self.generate_expression(cond)?;
            self.emitter.emit_line(&format!("if {} {{", cond_code));
            self.emitter.indent();
        }

        for stmt in body {
            self.generate_statement(stmt)?;
        }

        if condition.is_some() {
            self.emitter.dedent();
            self.emitter.emit_line("}");
        }

        self.emitter.dedent();
        self.emitter.emit_line("})?;");
        self.emitter.newline();

        Ok(())
    }

    fn generate_when_block(
        &mut self,
        condition: &TypedExpr,
        body: &[TypedStatement],
    ) -> Result<(), CodeGenError> {
        let cond_code = self.generate_expression(condition)?;

        self.emitter
            .emit_comment("ULissy: when block - conditional trigger");
        self.emitter.emit_line(&format!(
            "gns_runtime::watch_condition(|| {}, move || {{",
            cond_code
        ));
        self.emitter.indent();

        for stmt in body {
            self.generate_statement(stmt)?;
        }

        self.emitter.dedent();
        self.emitter.emit_line("})?;");
        self.emitter.newline();

        Ok(())
    }

    fn generate_send_statement(
        &mut self,
        recipient: &TypedExpr,
        fields: &[(String, TypedExpr)],
    ) -> Result<(), CodeGenError> {
        let recipient_code = self.generate_expression(recipient)?;

        self.emitter
            .emit_comment("ULissy: send statement - encrypted message");
        self.emitter.emit_line("{");
        self.emitter.indent();

        self.emitter
            .emit_line("let mut message = gns_runtime::Message::new();");

        for (name, value) in fields {
            let value_code = self.generate_expression(value)?;
            self.emitter
                .emit_line(&format!("message.set(\"{}\", {});", name, value_code));
        }

        self.emitter.newline();
        self.emitter.emit_line(&format!(
            "gns_runtime::send_encrypted(&{}, message)?;",
            recipient_code
        ));

        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        Ok(())
    }

    fn generate_enum_decl(
        &mut self,
        name: &str,
        type_params: &[String],
        variants: &[TypedEnumVariant],
    ) -> Result<(), CodeGenError> {
        self.emitter.emit_line("#[derive(Debug, Clone, PartialEq)]");

        if type_params.is_empty() {
            self.emitter.emit_line(&format!("pub enum {} {{", name));
        } else {
            let params = type_params.join(", ");
            self.emitter
                .emit_line(&format!("pub enum {}<{}> {{", name, params));
        }

        self.emitter.indent();

        for variant in variants {
            let variant_name = to_pascal_case(&variant.name);

            if let Some(associated) = &variant.associated_types {
                if associated.is_empty() {
                    self.emitter.emit_line(&format!("{},", variant_name));
                } else {
                    let types: Vec<String> =
                        associated.iter().map(|t| self.type_to_rust(t)).collect();
                    self.emitter
                        .emit_line(&format!("{}({}),", variant_name, types.join(", ")));
                }
            } else {
                self.emitter.emit_line(&format!("{},", variant_name));
            }
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        // Generate Display impl
        self.generate_enum_display_impl(name, type_params, variants)?;

        // Generate Default impl if applicable
        self.generate_enum_default_impl(name, type_params, variants)?;

        Ok(())
    }

    fn generate_enum_display_impl(
        &mut self,
        name: &str,
        type_params: &[String],
        _variants: &[TypedEnumVariant],
    ) -> Result<(), CodeGenError> {
        if type_params.is_empty() {
            self.emitter
                .emit_line(&format!("impl std::fmt::Display for {} {{", name));
        } else {
            let params = type_params.join(", ");
            self.emitter.emit_line(&format!(
                "impl<{}> std::fmt::Display for {}<{}> {{",
                params, name, params
            ));
        }
        self.emitter.indent();
        self.emitter
            .emit_line("fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {");
        self.emitter.indent();
        self.emitter.emit_line("write!(f, \"{:?}\", self)");
        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();
        Ok(())
    }

    fn generate_enum_default_impl(
        &mut self,
        name: &str,
        type_params: &[String],
        variants: &[TypedEnumVariant],
    ) -> Result<(), CodeGenError> {
        let default_variant = variants
            .iter()
            .find(|v| (v.name == "default" || v.name == "none") && v.associated_types.is_none());

        if let Some(variant) = default_variant {
            if type_params.is_empty() {
                self.emitter
                    .emit_line(&format!("impl Default for {} {{", name));
            } else {
                let params = type_params.join(", ");
                self.emitter.emit_line(&format!(
                    "impl<{}> Default for {}<{}> {{",
                    params, name, params
                ));
            }

            self.emitter.indent();
            self.emitter.emit_line("fn default() -> Self {");
            self.emitter.indent();
            self.emitter
                .emit_line(&format!("Self::{}", to_pascal_case(&variant.name)));
            self.emitter.dedent();
            self.emitter.emit_line("}");
            self.emitter.dedent();
            self.emitter.emit_line("}");
            self.emitter.newline();
        }

        Ok(())
    }

    fn generate_type_decl(
        &mut self,
        name: &str,
        fields: &[(String, Type)],
    ) -> Result<(), CodeGenError> {
        let skip_list = [
            "Breadcrumb",
            "TIT",
            "Epoch",
            "Identity",
            "Location",
            "SensorContext",
        ];
        if skip_list.contains(&name) {
            self.emitter.emit_comment(&format!(
                "ULissy context: type {} (using gns-runtime definition)",
                name
            ));
            self.emitter.newline();
            return Ok(());
        }

        self.emitter.emit_comment(&format!("ULissy: type {}", name));
        self.emitter.emit_line("#[derive(Debug, Clone, PartialEq)]");
        self.emitter.emit_line(&format!("pub struct {} {{", name));
        self.emitter.indent();

        for (f_name, f_type) in fields {
            let rust_type = self.type_to_rust(f_type);
            let snake_name = to_snake_case(f_name);
            self.emitter
                .emit_line(&format!("pub {}: {},", snake_name, rust_type));
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        self.emitter.emit_line("#[allow(non_snake_case)]");
        self.emitter.emit_line(&format!("impl {} {{", name));
        self.emitter.indent();

        let params: Vec<String> = fields
            .iter()
            .map(|(n, t)| format!("{}: {}", to_snake_case(n), self.type_to_rust(t)))
            .collect();
        self.emitter
            .emit_line(&format!("pub fn new({}) -> Self {{", params.join(", ")));
        self.emitter.indent();
        self.emitter.emit_line("Self {");
        self.emitter.indent();
        for (n, _) in fields {
            self.emitter.emit_line(&format!("{},", to_snake_case(n)));
        }
        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        for (n, t) in fields {
            let rust_type = self.type_to_rust(t);
            let snake_name = to_snake_case(n);
            self.emitter
                .emit_line(&format!("pub fn {}(&self) -> {} {{", snake_name, rust_type));
            self.emitter.indent();
            self.emitter
                .emit_line(&format!("self.{}.clone()", snake_name));
            self.emitter.dedent();
            self.emitter.emit_line("}");
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();
        Ok(())
    }

    fn generate_config_block(&mut self, fields: &[TypedConfigField]) -> Result<(), CodeGenError> {
        self.emitter
            .emit_comment("ULissy: config block - module configuration");
        self.emitter.emit_line("mod config {");
        self.emitter.indent();

        for field in fields {
            let value_code = self.generate_expression(&field.value)?;
            let const_name = to_screaming_snake_case(&field.name);
            let type_annotation = self.type_to_rust(&field.value.ty);

            if type_annotation != "auto" {
                self.emitter.emit_line(&format!(
                    "pub const {}: {} = {};",
                    const_name, type_annotation, value_code
                ));
            } else {
                self.emitter
                    .emit_line(&format!("pub const {} = {};", const_name, value_code));
            }
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        Ok(())
    }

    fn generate_computed_property(
        &mut self,
        name: &str,
        inferred_type: &Type,
        body: &TypedComputedBody,
    ) -> Result<(), CodeGenError> {
        let rust_name = to_snake_case(name);
        let type_annotation = self.type_to_rust(inferred_type);

        self.emitter
            .emit_comment("ULissy: computed property - reactive value");
        self.emitter
            .emit_line(&format!("fn {}() -> {} {{", rust_name, type_annotation));
        self.emitter.indent();

        match body {
            TypedComputedBody::Expression(expr) => {
                let expr_code = self.generate_expression(expr)?;
                self.emitter.emit_line(&expr_code);
            }
            TypedComputedBody::ObjectFields(fields) => {
                self.emitter.emit_line(&format!("{} {{", type_annotation));
                self.emitter.indent();

                for field in fields {
                    let value_code = self.generate_expression(&field.value)?;
                    let rust_field_name = to_snake_case(&field.name);
                    self.emitter
                        .emit_line(&format!("{}: {},", rust_field_name, value_code));
                }

                self.emitter.dedent();
                self.emitter.emit_line("}");
            }
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        Ok(())
    }

    fn generate_fn_decl(
        &mut self,
        name: &str,
        params: &[TypedParam],
        return_type: &Type,
        body: &[TypedStatement],
    ) -> Result<(), CodeGenError> {
        // Record function signature for call-site type casting
        let param_types: Vec<Type> = params.iter().map(|p| p.param_type.clone()).collect();
        self.fn_signatures.insert(name.to_string(), param_types);

        // Detect if function is recursive (calls itself)
        let is_recursive = body_contains_call(body, name);

        let params_str: Vec<String> = params
            .iter()
            .map(|p| {
                let rust_type = self.type_to_rust(&p.param_type);
                format!("{}: {}", p.name, rust_type)
            })
            .collect();

        let return_str = self.type_to_rust(return_type);

        if is_recursive {
            // Recursive async fn needs BoxFuture return type
            if return_str == "()" {
                self.emitter.emit_line(&format!(
                    "fn {}({}) -> std::pin::Pin<Box<dyn std::future::Future<Output = gns_runtime::RuntimeResult<()>> + Send>> {{",
                    name, params_str.join(", ")
                ));
            } else {
                self.emitter.emit_line(&format!(
                    "fn {}({}) -> std::pin::Pin<Box<dyn std::future::Future<Output = gns_runtime::RuntimeResult<{}>> + Send>> {{",
                    name, params_str.join(", "), return_str
                ));
            }
            self.emitter.indent();
            self.emitter.emit_line("Box::pin(async move {");
            self.emitter.indent();
        } else {
            if return_str == "()" {
                self.emitter.emit_line(&format!(
                    "async fn {}({}) -> gns_runtime::RuntimeResult<()> {{",
                    name,
                    params_str.join(", ")
                ));
            } else if !return_str.starts_with("RuntimeResult") {
                self.emitter.emit_line(&format!(
                    "async fn {}({}) -> gns_runtime::RuntimeResult<{}> {{",
                    name,
                    params_str.join(", "),
                    return_str
                ));
            } else {
                self.emitter.emit_line(&format!(
                    "async fn {}({}) -> {} {{",
                    name,
                    params_str.join(", "),
                    return_str
                ));
            }
            self.emitter.indent();
        }

        for (i, stmt) in body.iter().enumerate() {
            if i == body.len() - 1 && return_str != "()" {
                match &stmt.kind {
                    TypedStatementKind::ExpressionStatement(expr) => {
                        let code = self.generate_expression(expr)?;
                        self.emitter.emit_line(&format!("Ok({})", code));
                        continue;
                    }
                    TypedStatementKind::ReturnStatement(_) => {
                        self.generate_statement(stmt)?;
                    }
                    _ => {
                        self.generate_statement(stmt)?;
                    }
                }
            } else {
                self.generate_statement(stmt)?;
            }
        }

        if return_str == "()" {
            self.emitter.newline();
            self.emitter.emit_line("Ok(())");
        }

        if body.is_empty() && return_str != "()" {
            self.emitter.emit_line(&format!(
                "Ok(Default::default()) // TODO: implement {}",
                name
            ));
        }

        if is_recursive {
            self.emitter.dedent();
            self.emitter.emit_line("})"); // close Box::pin(async move {
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        self.emitter.newline();

        Ok(())
    }

    fn generate_return_statement(&mut self, expr: &Option<TypedExpr>) -> Result<(), CodeGenError> {
        if let Some(e) = expr {
            let code = self.generate_expression(e)?;
            self.emitter.emit_line(&format!("return Ok({});", code));
        } else {
            self.emitter.emit_line("return Ok(());");
        }
        Ok(())
    }

    fn generate_if_statement(
        &mut self,
        condition: &TypedExpr,
        then_block: &[TypedStatement],
        else_block: &Option<Vec<TypedStatement>>,
    ) -> Result<(), CodeGenError> {
        let cond_code = self.generate_expression(condition)?;
        self.emitter.emit_line(&format!("if {} {{", cond_code));
        self.emitter.indent();

        for stmt in then_block {
            self.generate_statement(stmt)?;
        }

        self.emitter.dedent();

        if let Some(else_stmts) = else_block {
            self.emitter.emit_line("} else {");
            self.emitter.indent();

            for stmt in else_stmts {
                self.generate_statement(stmt)?;
            }

            self.emitter.dedent();
        }

        self.emitter.emit_line("}");
        Ok(())
    }

    fn generate_if_let_statement(
        &mut self,
        binding: &str,
        value: &TypedExpr,
        then_block: &[TypedStatement],
        else_block: &Option<Vec<TypedStatement>>,
    ) -> Result<(), CodeGenError> {
        let value_code = self.generate_expression(value)?;
        let rust_binding = self.map_identifier(&to_snake_case(binding));

        self.emitter.emit_line(&format!(
            "if let Some({}) = {} {{",
            rust_binding, value_code
        ));
        self.emitter.indent();

        for stmt in then_block {
            self.generate_statement(stmt)?;
        }

        self.emitter.dedent();

        if let Some(else_stmts) = else_block {
            self.emitter.emit_line("} else {");
            self.emitter.indent();

            for stmt in else_stmts {
                self.generate_statement(stmt)?;
            }

            self.emitter.dedent();
        }

        self.emitter.emit_line("}");
        Ok(())
    }

    fn generate_for_statement(
        &mut self,
        item_name: &str,
        collection: &TypedExpr,
        body: &[TypedStatement],
    ) -> Result<(), CodeGenError> {
        let mut collection_code = self.generate_expression(collection)?;

        if collection.ty == Type::String {
            collection_code = format!("{}.chars()", collection_code);
        }

        let rust_item = self.map_identifier(&to_snake_case(item_name));

        self.emitter
            .emit_line(&format!("for {} in {} {{", rust_item, collection_code));
        self.emitter.indent();

        for stmt in body {
            self.generate_statement(stmt)?;
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        Ok(())
    }

    fn generate_match_statement(
        &mut self,
        expr: &TypedExpr,
        cases: &[TypedMatchCase],
    ) -> Result<(), CodeGenError> {
        let expr_code = self.generate_expression(expr)?;
        self.emitter.emit_line(&format!("match {} {{", expr_code));
        self.emitter.indent();

        for case in cases {
            let pat_code = self.pattern_to_rust(&case.pattern, &expr.ty);

            if let Some(guard) = &case.guard {
                let guard_code = self.generate_expression(guard)?;
                self.emitter
                    .emit_line(&format!("{} if {} => {{", pat_code, guard_code));
            } else {
                self.emitter.emit_line(&format!("{} => {{", pat_code));
            }

            self.emitter.indent();
            for stmt in &case.body {
                self.generate_statement(stmt)?;
            }
            self.emitter.dedent();
            self.emitter.emit_line("},");
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        Ok(())
    }

    fn pattern_to_rust(&self, pat: &ast::Pattern, subject_type: &Type) -> String {
        match pat {
            ast::Pattern::Literal(lit) => match lit {
                ast::Literal::Int(i) => i.to_string(),
                ast::Literal::Float(f) => f.to_string(),
                ast::Literal::Bool(b) => b.to_string(),
                ast::Literal::String(s) => format!("\"{}\"", s),
                ast::Literal::Nil => "None".to_string(),
            },
            ast::Pattern::Identifier(name) => {
                if name == "_" {
                    return "_".to_string();
                }
                // FIX H: Use enum_variant_map for lookup
                let pascal = to_pascal_case(name);
                if let Type::Named(enum_name) = subject_type {
                    format!("{}::{}", enum_name, pascal)
                } else if let Type::Enum {
                    name: enum_name, ..
                } = subject_type
                {
                    format!("{}::{}", enum_name, pascal)
                } else if let Some(enum_name) = self.enum_variant_map.get(&pascal) {
                    format!("{}::{}", enum_name, pascal)
                } else {
                    name.to_string()
                }
            }
            ast::Pattern::EnumVariant { name, bindings } => {
                let path = if let Type::Named(enum_name) = subject_type {
                    format!("{}::{}", enum_name, name)
                } else {
                    name.clone()
                };

                if bindings.is_empty() {
                    path
                } else {
                    let binds: Vec<String> = bindings.iter().map(|b| to_snake_case(b)).collect();
                    format!("{}({})", path, binds.join(", "))
                }
            }
            ast::Pattern::Wildcard => "_".to_string(),
            ast::Pattern::Tuple(pats) => {
                let parts: Vec<String> = pats
                    .iter()
                    .map(|p| self.pattern_to_rust(p, subject_type))
                    .collect();
                format!("({})", parts.join(", "))
            }
        }
    }

    fn generate_after_block(
        &mut self,
        delay: &TypedExpr,
        body: &[TypedStatement],
    ) -> Result<(), CodeGenError> {
        let delay_code = self.generate_expression(delay)?;

        self.emitter
            .emit_line(&format!("tokio::time::sleep({}).await;", delay_code));
        self.emitter.emit_line("{");
        self.emitter.indent();

        for stmt in body {
            self.generate_statement(stmt)?;
        }

        self.emitter.dedent();
        self.emitter.emit_line("}");
        Ok(())
    }

    // ========================================================================
    // EXPRESSION GENERATION
    // ========================================================================

    fn generate_expression(&self, expr: &TypedExpr) -> Result<String, CodeGenError> {
        match &expr.kind {
            TypedExprKind::Literal(lit) => self.generate_literal(lit),

            TypedExprKind::Identifier(name) => {
                if let Some(variant) = name.strip_prefix('.') {
                    // Enum shorthand: .public -> EnumType::Public
                    let capitalized = to_pascal_case(variant);

                    // FIX H: Use enum_variant_map for type lookup
                    let type_name = if let Type::Enum {
                        name: enum_name, ..
                    } = &expr.ty
                    {
                        enum_name.clone()
                    } else if let Type::Named(n) = &expr.ty {
                        n.clone()
                    } else if let Some(enum_name) = self.enum_variant_map.get(&capitalized) {
                        enum_name.clone()
                    } else {
                        // Last resort: use capitalized as-is (will error at compile time)
                        "UnknownEnum".to_string()
                    };

                    Ok(format!("{}::{}", type_name, capitalized))
                } else {
                    let mapped = self.map_identifier(name);
                    // Wrap static mut variable access in unsafe
                    if self.static_vars.contains(name) {
                        Ok(format!("unsafe {{ {} }}", mapped))
                    } else {
                        Ok(mapped)
                    }
                }
            }

            TypedExprKind::Handle(h) => Ok(format!("Handle::from_str(\"@{}\")?", h)),

            TypedExprKind::FacetAddress { prefix, handle } => {
                Ok(format!("FacetAddress::new(\"{}\", \"{}\")", prefix, handle))
            }

            TypedExprKind::Binary { left, op, right } => {
                let mut l = self.generate_expression(left)?;
                let mut r = self.generate_expression(right)?;

                // FIX F: For division, ensure both sides are f64
                if matches!(op, BinaryOp::Div) {
                    if left.ty != Type::Float {
                        l = format!("({} as f64)", l);
                    }
                    if right.ty != Type::Float {
                        r = format!("({} as f64)", r);
                    }
                } else {
                    // Standard mixed arithmetic casting for non-division
                    if left.ty == Type::Float && right.ty == Type::Int {
                        r = format!("({} as f64)", r);
                    } else if left.ty == Type::Int && right.ty == Type::Float {
                        l = format!("({} as f64)", l);
                    }
                }

                if needs_parens(op, left) {
                    l = format!("({})", l);
                }
                if needs_parens(op, right) {
                    r = format!("({})", r);
                }

                // Fix i64 vs usize comparisons (e.g., i >= s.len())
                if matches!(
                    op,
                    BinaryOp::Eq
                        | BinaryOp::NotEq
                        | BinaryOp::Lt
                        | BinaryOp::Gt
                        | BinaryOp::LtEq
                        | BinaryOp::GtEq
                ) {
                    if left.ty == Type::Int && r.contains(".len()") {
                        r = format!("({} as i64)", r);
                    } else if right.ty == Type::Int && l.contains(".len()") {
                        l = format!("({} as i64)", l);
                    }
                    // Fix Hash vs Vec<u8> comparisons (e.g., bc.previous_hash == Bytes::zeros(32))
                    if left.ty == Type::Hash && !matches!(right.ty, Type::Hash) {
                        r = format!("gns_runtime::Hash({})", r);
                    } else if right.ty == Type::Hash && !matches!(left.ty, Type::Hash) {
                        l = format!("gns_runtime::Hash({})", l);
                    }
                }

                let op_str = self.binary_op_to_rust(op);
                Ok(format!("{} {} {}", l, op_str, r))
            }

            TypedExprKind::Unary { op, operand } => {
                let inner = self.generate_expression(operand)?;
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                Ok(format!("{}{}", op_str, inner))
            }

            TypedExprKind::Member { object, member } => {
                let obj = self.generate_expression(object)?;
                let mapped = self.map_member(member);

                match member.as_str() {
                    "length" | "count" | "digest" => Ok(format!("{}.{}()", obj, mapped)),
                    "primary" => Ok(format!("{}::{}()", obj, mapped)),
                    "zeros" | "fill" => Ok(format!("{}::{}", obj, mapped)),
                    _ => {
                        if obj == "Keychain" || obj == "Bytes" || obj == "Sensors" {
                            Ok(format!("{}::{}", obj, mapped))
                        } else if mapped == "stellar_address" {
                            Ok(format!("{}.stellar_address()", obj))
                        } else {
                            Ok(format!("{}.{}", obj, mapped))
                        }
                    }
                }
            }

            TypedExprKind::Call { callee, args } => {
                let callee_code = self.generate_expression(callee)?;
                let args_code: Result<Vec<_>, _> =
                    args.iter().map(|a| self.generate_expression(a)).collect();
                let args_vec = args_code?;
                let args_str = args_vec.join(", ");

                let call = match callee_code.as_str() {
                    "print" => format!("println!(\"{{:?}}\", {})", args_str),
                    "breadcrumb" => {
                        let cell = &args_vec[0];
                        let context = &args_vec[1];
                        let prev_arg = &args_vec[2];
                        let prev_code = match &args[2].ty {
                            Type::Nil => "None::<&gns_runtime::Breadcrumb>".to_string(),
                            Type::Breadcrumb => format!("Some(&{})", prev_arg),
                            Type::Optional(inner) if matches!(**inner, Type::Breadcrumb) => {
                                format!("{}.as_ref()", prev_arg)
                            }
                            _ => format!("Some(&{})", prev_arg),
                        };
                        format!(
                            "gns_runtime::breadcrumb().cell({}).context({}).previous({})",
                            cell, context, prev_code
                        )
                    }
                    "computeTIT" => {
                        format!("TIT::from_public_key({}.as_bytes())", args_vec[0])
                    }
                    "log2" => {
                        let arg = if args[0].ty == Type::Int {
                            format!("({} as f64)", args_vec[0])
                        } else {
                            args_vec[0].clone()
                        };
                        format!("({}).log2()", arg)
                    }
                    "tanh" => {
                        let arg = if args[0].ty == Type::Int {
                            format!("({} as f64)", args_vec[0])
                        } else {
                            args_vec[0].clone()
                        };
                        format!("({}).tanh()", arg)
                    }
                    "exp" => {
                        let arg = if args[0].ty == Type::Int {
                            format!("({} as f64)", args_vec[0])
                        } else {
                            args_vec[0].clone()
                        };
                        format!("({}).exp()", arg)
                    }
                    "abs" => {
                        format!("({}).abs()", args_vec[0])
                    }
                    "sha256" => match &args[0].ty {
                        Type::PublicKey | Type::Signature | Type::Hash => {
                            format!("gns_runtime::sha256({}.as_bytes())", args_vec[0])
                        }
                        Type::Breadcrumb => {
                            format!("gns_runtime::sha256(&{}.to_bytes())", args_vec[0])
                        }
                        _ => format!("gns_runtime::sha256(&{})", args_vec[0]),
                    },
                    // FIX E: Keep original camelCase function names, add .await?
                    // (computeTrustScore and trustDecayAfterDays fall through to default handler
                    //  which applies fn_signatures argument type casting)
                    // FIX J: Handle Keychain::facet specially
                    "Keychain::facet" => {
                        let arg = strip_to_string(&args_vec[0]);
                        format!("Keychain::facet({})?", arg)
                    }
                    _ => {
                        // Cast arguments to match function parameter types
                        let mut cast_args = args_vec.clone();
                        if let Some(param_types) = self.fn_signatures.get(callee_code.as_str()) {
                            for (i, (arg_code, arg_expr)) in
                                cast_args.iter_mut().zip(args.iter()).enumerate()
                            {
                                if i < param_types.len()
                                    && param_types[i] == Type::Float
                                    && (arg_expr.ty == Type::Int
                                        || !matches!(arg_expr.ty, Type::Float))
                                    && !arg_code.contains("as f64")
                                    && !arg_code.contains(".000")
                                {
                                    *arg_code = format!("{} as f64", arg_code);
                                }
                            }
                        }
                        let cast_args_str = cast_args.join(", ");

                        let base_call = format!("{}({})", callee_code, cast_args_str);
                        if callee_code.ends_with("Sensors::current") {
                            "Sensors::current()".to_string()
                        } else if callee_code.ends_with("Location::current") {
                            "Location::current()?".to_string()
                        } else if callee_code.contains("Keychain::facet") {
                            // FIX J: Also catch Keychain::facet when not exact match
                            let stripped_args = strip_to_string(&cast_args_str);
                            format!("Keychain::facet({})?", stripped_args)
                        } else {
                            format!("{}.await?", base_call)
                        }
                    }
                };

                Ok(call)
            }

            TypedExprKind::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.generate_expression(object)?;

                let should_cast = matches!(object.ty, Type::Any | Type::Unknown)
                    || matches!(&object.ty, Type::Named(ref n) if n == "Any" || n == "any");

                let obj = if should_cast {
                    if method == "chars"
                        || method == "len"
                        || method == "startsWith"
                        || method == "slice"
                    {
                        format!("({}.downcast_ref::<String>().unwrap())", obj)
                    } else {
                        obj
                    }
                } else {
                    obj
                };

                let method_name = self.map_method(method);

                let args_code: Result<Vec<_>, _> =
                    args.iter().map(|a| self.generate_expression(a)).collect();
                let args_vec = args_code?;
                let args_str = args_vec.join(", ");

                match method.as_str() {
                    "length" => Ok(format!("{}.len()", obj)),
                    "stellarAddress" => Ok(format!("{}.stellar_address()", obj)),
                    "toHex" => Ok(format!("hex::encode(&{})", obj)),
                    "charAt" => Ok(format!(
                        "{}.chars().nth({} as usize).unwrap().to_string()",
                        obj, args_str
                    )),
                    "isDigit" => Ok(format!("{}.chars().all(|c| c.is_ascii_digit())", obj)),
                    "isLowercase" => Ok(format!("{}.chars().all(|c| c.is_lowercase())", obj)),
                    // FIX B: Strip .to_string() for starts_with (expects &str)
                    "startsWith" => {
                        let arg = strip_to_string(&args_str);
                        Ok(format!("{}.starts_with({})", obj, arg))
                    }
                    "slice" => {
                        let use_to_string = matches!(object.ty, Type::String)
                            || matches!(object.ty, Type::Any | Type::Unknown);
                        let conversion = if use_to_string {
                            ".to_string()"
                        } else {
                            ".to_vec()"
                        };

                        let args_split: Vec<&str> = args_str.split(", ").collect();
                        if args_split.len() == 1 {
                            Ok(format!("{}[{}..]{}", obj, args_split[0], conversion))
                        } else if args_split.len() == 2 {
                            Ok(format!(
                                "{}[{}..{}]{}",
                                obj, args_split[0], args_split[1], conversion
                            ))
                        } else {
                            Ok(format!("{}.slice({})", obj, args_str))
                        }
                    }
                    // FIX B: Strip .to_string() for facet (expects &str)
                    "facet" => {
                        let arg = strip_to_string(&args_str);
                        Ok(format!("{}::facet({})?", obj, arg))
                    }
                    // FIX C: h3/to_h3 → add `as u8` cast and `?`
                    "h3" | "to_h3" => {
                        let res_arg = format!("{} as u8", args_vec[0]);
                        Ok(format!("{}.to_h3({})?", obj, res_arg))
                    }
                    // FIX D: sign/signed → add & reference and ? for builder
                    "sign" | "signed" => {
                        let arg_raw = &args_vec[0];
                        let arg_prepped = if matches!(args[0].ty, Type::String) {
                            format!("{}.as_bytes()", arg_raw)
                        } else {
                            format!("&{}", arg_raw)
                        };

                        if matches!(object.ty, Type::Identity) {
                            // Identity.sign(data) → returns Signature directly
                            Ok(format!("{}.{}({})", obj, method_name, arg_prepped))
                        } else {
                            // Builder.sign(identity) → returns Result<Breadcrumb>
                            Ok(format!("{}.{}({})?", obj, method_name, arg_prepped))
                        }
                    }
                    "verify" => {
                        let data_raw = &args_vec[0];
                        let key_raw = &args_vec[1];
                        let data = if matches!(args[0].ty, Type::String) {
                            format!("{}.as_bytes()", data_raw)
                        } else if matches!(args[0].ty, Type::Breadcrumb) {
                            format!("&{}.to_bytes()", data_raw)
                        } else {
                            data_raw.clone()
                        };
                        Ok(format!("{}.{}({}, &{})", obj, method_name, data, key_raw))
                    }
                    "toFixed" => Ok(format!("format!(\"{{:.{}}}\", {})", args_vec[0], obj)),
                    "tit" => Ok(format!("{}.tit()", obj)),
                    _ => {
                        if obj == "Bytes" || obj == "Keychain" || obj == "Sensors" {
                            if args_str.is_empty() {
                                Ok(format!("{}::{}()", obj, method_name))
                            } else {
                                Ok(format!("{}::{}({})", obj, method_name, args_str))
                            }
                        } else if method == "fromSeed" || method == "primary" {
                            let fixed_args = strip_to_string(&args_str);
                            Ok(format!("{}::{}({})?", obj, method_name, fixed_args))
                        } else {
                            Ok(format!("{}.{}({})", obj, method_name, args_str))
                        }
                    }
                }
            }

            TypedExprKind::UnitValue { value, unit } => {
                let val = self.generate_expression(value)?;
                match unit.as_str() {
                    "milliseconds" => Ok(format!("std::time::Duration::from_millis({})", val)),
                    "seconds" | "second" | "sec" => {
                        Ok(format!("std::time::Duration::from_secs({})", val))
                    }
                    "minutes" | "minute" | "min" => {
                        Ok(format!("std::time::Duration::from_secs(({}) * 60)", val))
                    }
                    "hours" | "hour" | "hr" => {
                        Ok(format!("std::time::Duration::from_secs(({}) * 3600)", val))
                    }
                    "days" | "day" => {
                        Ok(format!("std::time::Duration::from_secs(({}) * 86400)", val))
                    }
                    "meters" | "meter" | "m" => Ok(format!("Distance::from_meters({})", val)),
                    "kilometers" | "kilometer" | "km" => {
                        Ok(format!("Distance::from_meters(({}) * 1000.0)", val))
                    }
                    _ => Ok(format!("{}::from({})", unit, val)),
                }
            }

            TypedExprKind::Array(elements) => {
                let elems: Result<Vec<_>, _> = elements
                    .iter()
                    .map(|e| self.generate_expression(e))
                    .collect();
                Ok(format!("vec![{}]", elems?.join(", ")))
            }

            TypedExprKind::OptionalMember { object, member } => {
                let obj = self.generate_expression(object)?;
                let member_rust = self.map_member(member);
                Ok(format!(
                    "{}.as_ref().map(|__opt| __opt.{}())",
                    obj, member_rust
                ))
            }

            TypedExprKind::OptionalMethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.generate_expression(object)?;
                let method_rust = self.map_method(method);
                let args_code: Result<Vec<_>, _> =
                    args.iter().map(|a| self.generate_expression(a)).collect();
                let args_str = args_code?.join(", ");

                if args_str.is_empty() {
                    Ok(format!(
                        "{}.as_ref().map(|__opt| __opt.{}())",
                        obj, method_rust
                    ))
                } else {
                    Ok(format!(
                        "{}.as_ref().map(|__opt| __opt.{}({}))",
                        obj, method_rust, args_str
                    ))
                }
            }

            TypedExprKind::NilCoalescing { primary, fallback } => {
                let primary_code = self.generate_expression(primary)?;
                let fallback_code = self.generate_expression(fallback)?;

                let is_simple_fallback = matches!(
                    &fallback.kind,
                    TypedExprKind::Literal(_) | TypedExprKind::Identifier(_)
                );

                if is_simple_fallback {
                    Ok(format!("{}.unwrap_or({})", primary_code, fallback_code))
                } else {
                    Ok(format!(
                        "{}.unwrap_or_else(|| {})",
                        primary_code, fallback_code
                    ))
                }
            }

            TypedExprKind::ObjectLiteral { fields, type_hint } => {
                self.generate_object_literal(fields, type_hint)
            }

            TypedExprKind::InterpolatedString { parts } => self.generate_interpolated_string(parts),

            TypedExprKind::Search {
                target,
                filters,
                ranking,
            } => self.generate_search_expression(target, filters, ranking),
        }
    }

    fn generate_search_expression(
        &self,
        target: &ulissy_types::TypedSearchTarget,
        filters: &[ulissy_types::TypedSearchFilter],
        ranking: &Option<ulissy_types::TypedSearchRanking>,
    ) -> Result<String, CodeGenError> {
        let mut code = String::from("gns_search::query()");

        // --- Target ---
        match target {
            ulissy_types::TypedSearchTarget::Nearby { radius } => {
                if let Some(r) = radius {
                    let radius_code = self.generate_expression(r)?;
                    code.push_str(&format!("\n        .nearby({})", radius_code));
                } else {
                    code.push_str("\n        .nearby_default()");
                }
            }
            ulissy_types::TypedSearchTarget::Within { center, radius } => {
                let center_code = self.generate_expression(center)?;
                let radius_code = self.generate_expression(radius)?;
                code.push_str(&format!(
                    "\n        .within({}, {})",
                    center_code, radius_code
                ));
            }
            ulissy_types::TypedSearchTarget::Identity { handle } => {
                let handle_code = self.generate_expression(handle)?;
                code.push_str(&format!("\n        .identity({})", handle_code));
            }
            ulissy_types::TypedSearchTarget::Text { query } => {
                let query_code = self.generate_expression(query)?;
                code.push_str(&format!("\n        .text({})", query_code));
            }
        }

        // --- Filters ---
        for filter in filters {
            match filter {
                ulissy_types::TypedSearchFilter::TrustThreshold { op, value } => {
                    let val = self.generate_expression(value)?;
                    match op {
                        ast::ComparisonOp::Greater | ast::ComparisonOp::GreaterEqual => {
                            code.push_str(&format!("\n        .trust_min({})", val));
                        }
                        ast::ComparisonOp::Less | ast::ComparisonOp::LessEqual => {
                            code.push_str(&format!("\n        .trust_max({})", val));
                        }
                        ast::ComparisonOp::Equal => {
                            code.push_str(&format!("\n        .trust_exact({})", val));
                        }
                        _ => {
                            code.push_str(&format!("\n        .trust_min({})", val));
                        }
                    }
                }
                ulissy_types::TypedSearchFilter::FacetMatch { facet_name } => {
                    let name = self.generate_expression(facet_name)?;
                    code.push_str(&format!("\n        .facet({})", name));
                }
                ulissy_types::TypedSearchFilter::ActiveWithin { duration } => {
                    let dur = self.generate_expression(duration)?;
                    code.push_str(&format!("\n        .active_within({})", dur));
                }
                ulissy_types::TypedSearchFilter::OrgMatch { org_name } => {
                    let org = self.generate_expression(org_name)?;
                    code.push_str(&format!("\n        .org({})", org));
                }
                ulissy_types::TypedSearchFilter::FieldCompare { field, op, value } => {
                    let val = self.generate_expression(value)?;
                    let op_str = match op {
                        ast::ComparisonOp::Greater => "Gt",
                        ast::ComparisonOp::GreaterEqual => "Ge",
                        ast::ComparisonOp::Less => "Lt",
                        ast::ComparisonOp::LessEqual => "Le",
                        ast::ComparisonOp::Equal => "Eq",
                        ast::ComparisonOp::NotEqual => "Ne",
                        ast::ComparisonOp::Contains => "Contains",
                    };
                    code.push_str(&format!(
                        "\n        .filter(\"{}\", gns_search::Op::{}, {})",
                        field, op_str, val
                    ));
                }
            }
        }

        // --- Ranking ---
        if let Some(rank) = ranking {
            let (key, order) = match rank {
                ulissy_types::TypedSearchRanking::Trust { order } => ("Trust", order),
                ulissy_types::TypedSearchRanking::Distance { order } => ("Distance", order),
                ulissy_types::TypedSearchRanking::Recency { order } => ("Recency", order),
                ulissy_types::TypedSearchRanking::Relevance { order } => ("Relevance", order),
            };
            let order_str = match order {
                ast::SortOrder::Ascending => "Asc",
                ast::SortOrder::Descending => "Desc",
            };
            code.push_str(&format!(
                "\n        .rank_by(gns_search::RankingKey::{}, gns_search::Order::{})",
                key, order_str
            ));
        }

        // --- Execute ---
        code.push_str("\n        .execute().await?");

        Ok(code)
    }

    fn generate_object_literal(
        &self,
        fields: &[TypedObjectField],
        type_hint: &Option<String>,
    ) -> Result<String, CodeGenError> {
        if let Some(type_name) = type_hint {
            let mut field_inits = Vec::new();

            for field in fields {
                let value_code = self.generate_expression(&field.value)?;
                let field_name = to_snake_case(&field.name);
                field_inits.push(format!("{}: {}", field_name, value_code));
            }

            Ok(format!("{} {{ {} }}", type_name, field_inits.join(", ")))
        } else {
            let mut inserts = Vec::new();

            for field in fields {
                let value_code = self.generate_expression(&field.value)?;
                inserts.push(format!(
                    "(\"{}\".to_string(), Box::new({}) as Box<dyn std::any::Any>)",
                    field.name, value_code
                ));
            }

            Ok(format!(
                "std::collections::HashMap::from([{}])",
                inserts.join(", ")
            ))
        }
    }

    fn generate_interpolated_string(
        &self,
        parts: &[TypedInterpolatedPart],
    ) -> Result<String, CodeGenError> {
        let mut format_string = String::new();
        let mut args = Vec::new();

        for part in parts {
            match part {
                TypedInterpolatedPart::Literal(s) => {
                    let escaped = s.replace('{', "{{").replace('}', "}}");
                    format_string.push_str(&escaped);
                }
                TypedInterpolatedPart::Expression(expr) => {
                    format_string.push_str("{}");
                    let expr_code = self.generate_expression(expr)?;
                    args.push(expr_code);
                }
            }
        }

        if args.is_empty() {
            Ok(format!("\"{}\".to_string()", format_string))
        } else {
            Ok(format!(
                "format!(\"{}\", {})",
                format_string,
                args.join(", ")
            ))
        }
    }

    fn generate_literal(&self, lit: &Literal) -> Result<String, CodeGenError> {
        match lit {
            Literal::Int(n) => Ok(n.to_string()),
            Literal::Float(f) => Ok(format!("{:.6}", f)),
            Literal::String(s) => Ok(format!("\"{}\".to_string()", s.replace('"', "\\\""))),
            Literal::Bool(b) => Ok(b.to_string()),
            Literal::Nil => Ok("None".to_string()),
        }
    }

    // ========================================================================
    // TYPE MAPPING
    // ========================================================================

    fn type_to_rust(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "i64".to_string(),
            Type::Float => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Nil => "Option<()>".to_string(),
            Type::Identity => "Identity".to_string(),
            Type::PublicKey => "PublicKey".to_string(),
            Type::PrivateKey => "PrivateKey".to_string(),
            Type::Signature => "Signature".to_string(),
            Type::Handle => "Handle".to_string(),
            Type::H3Cell => "H3Cell".to_string(),
            Type::Coordinates => "Coordinates".to_string(),
            Type::Distance => "Distance".to_string(),
            Type::Moment => "Moment".to_string(),
            Type::Duration => "Duration".to_string(),
            Type::Hash => "Hash".to_string(),
            Type::BatteryLevel => "BatteryLevel".to_string(),
            Type::Breadcrumb => "Breadcrumb".to_string(),
            Type::PresenceProof => "PresenceProof".to_string(),
            Type::Trajectory => "Trajectory".to_string(),
            Type::FacetAddress => "FacetAddress".to_string(),
            Type::Array(inner) => format!("Vec<{}>", self.type_to_rust(inner)),
            Type::Optional(inner) => format!("Option<{}>", self.type_to_rust(inner)),
            Type::Unit => "()".to_string(),
            // FIX A: Any/Unknown still maps here for function params;
            // LetDecl/VarDecl override to "auto" for local variables
            Type::Any | Type::Unknown => "Box<dyn std::any::Any>".to_string(),
            Type::Named(n) => {
                if n == "Bytes" {
                    "Vec<u8>".to_string()
                } else {
                    n.clone()
                }
            }
            _ => "Box<dyn std::any::Any>".to_string(),
        }
    }

    // ========================================================================
    // IDENTIFIER/METHOD MAPPING
    // ========================================================================

    fn map_identifier(&self, name: &str) -> String {
        match name {
            "Keychain" => "Keychain".to_string(),
            "here" => "Location::current()?".to_string(),
            "now" => "Moment::now()".to_string(),
            "battery" => "Battery::level()?".to_string(),
            "sensors" => "Sensors::current()".to_string(), // No ? - returns SensorContext directly
            "Visibility" => "TrustLevel".to_string(),
            // Config access: use static ref instead of static mut
            "config" => "CONFIG".to_string(),
            _ => name.to_string(),
        }
    }

    fn map_member(&self, member: &str) -> String {
        match member {
            "publicKey" => "public_key".to_string(),
            "privateKey" => "private_key".to_string(),
            "stellarAddress" => "stellar_address".to_string(),
            "previousHash" => "previous_hash".to_string(),
            "startIndex" => "start_index".to_string(),
            "endIndex" => "end_index".to_string(),
            "startTime" => "start_time".to_string(),
            "endTime" => "end_time".to_string(),
            "merkleRoot" => "merkle_root".to_string(),
            "signalStrength" => "signal_strength".to_string(),
            "proximityType" => "proximity_type".to_string(),
            "baseScore" => "base_score".to_string(),
            "peerMultiplier" => "peer_multiplier".to_string(),
            "anchorMultiplier" => "anchor_multiplier".to_string(),
            "decayFactor" => "decay_factor".to_string(),
            "protocolVersion" => "protocol_version".to_string(),
            "parisiK" => "parisi_k".to_string(),
            "tau0Human" => "tau0_human".to_string(),
            "tau0Drone" => "tau0_drone".to_string(),
            "minBreadcrumbs" => "min_breadcrumbs".to_string(),
            "h3Resolution" => "h3_resolution".to_string(),
            "primary" => "primary".to_string(),
            "trajectory" => "trajectory".to_string(),
            "length" => "len".to_string(),
            "count" => "len".to_string(),
            "last" => "last_hash".to_string(),
            "digest" => "digest".to_string(),
            _ => to_snake_case(member),
        }
    }

    fn map_method(&self, method: &str) -> String {
        match method {
            "length" => "len".to_string(),
            "startsWith" => "starts_with".to_string(),
            "endsWith" => "ends_with".to_string(),
            "charAt" => "chars().nth".to_string(),
            "toUpperCase" => "to_uppercase".to_string(),
            "toLowerCase" => "to_lowercase".to_string(),
            "contains" => "contains".to_string(),
            "split" => "split".to_string(),
            "trim" => "trim".to_string(),
            "slice" => "get".to_string(),
            "push" => "push".to_string(),
            "pop" => "pop".to_string(),
            "first" => "first".to_string(),
            "last" => "last".to_string(),
            "isEmpty" => "is_empty".to_string(),
            "count" => "len".to_string(),
            "abs" => "abs".to_string(),
            "floor" => "floor".to_string(),
            "ceil" => "ceil".to_string(),
            "round" => "round".to_string(),
            "sqrt" => "sqrt".to_string(),
            "pow" => "powf".to_string(),
            "log" => "ln".to_string(),
            "log2" => "log2".to_string(),
            "log10" => "log10".to_string(),
            "exp" => "exp".to_string(),
            "sin" => "sin".to_string(),
            "cos" => "cos".to_string(),
            "tan" => "tan".to_string(),
            "tanh" => "tanh".to_string(),
            "toString" => "to_string".to_string(),
            "toInt" => "try_into().unwrap".to_string(),
            "toFloat" => "into".to_string(),
            "toHex" => "to_hex".to_string(),
            "toBytes" => "as_bytes".to_string(),
            "digest" => "digest".to_string(),
            "verify" => "verify".to_string(),
            "sign" | "signed" => "sign".to_string(),
            "stellarAddress" => "stellar_address".to_string(),
            "h3" => "to_h3".to_string(),
            "append" => "append".to_string(),
            "post" => "post".to_string(),
            "set" => "set".to_string(),
            "request" => "request".to_string(),
            _ => method.to_string(),
        }
    }

    fn binary_op_to_rust(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Range => "..",
            BinaryOp::RangeExclusive => "..",
            BinaryOp::Within => "/* within */",
            BinaryOp::Near => "/* near */",
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// FIX B: Strip .to_string() from a string expression
/// Used when passing string literals to functions expecting &str
fn strip_to_string(s: &str) -> String {
    s.replace(".to_string()", "")
}

/// Detect if a function body contains a call to itself (recursive)
fn body_contains_call(body: &[TypedStatement], fn_name: &str) -> bool {
    for stmt in body {
        if stmt_contains_call(stmt, fn_name) {
            return true;
        }
    }
    false
}

fn stmt_contains_call(stmt: &TypedStatement, fn_name: &str) -> bool {
    match &stmt.kind {
        TypedStatementKind::ExpressionStatement(expr) => expr_contains_call(expr, fn_name),
        TypedStatementKind::ReturnStatement(Some(expr)) => expr_contains_call(expr, fn_name),
        TypedStatementKind::LetDecl { init, .. } => expr_contains_call(init, fn_name),
        TypedStatementKind::VarDecl {
            init: Some(init), ..
        } => expr_contains_call(init, fn_name),
        TypedStatementKind::IfStatement {
            condition,
            then_block,
            else_block,
        } => {
            expr_contains_call(condition, fn_name)
                || body_contains_call(then_block, fn_name)
                || else_block
                    .as_ref()
                    .is_some_and(|b| body_contains_call(b, fn_name))
        }
        TypedStatementKind::ForStatement { body, .. } => body_contains_call(body, fn_name),
        _ => false,
    }
}

fn expr_contains_call(expr: &TypedExpr, fn_name: &str) -> bool {
    match &expr.kind {
        TypedExprKind::Call { callee, args } => {
            if let TypedExprKind::Identifier(name) = &callee.kind {
                if name == fn_name {
                    return true;
                }
            }
            expr_contains_call(callee, fn_name)
                || args.iter().any(|a| expr_contains_call(a, fn_name))
        }
        TypedExprKind::MethodCall { object, args, .. } => {
            expr_contains_call(object, fn_name)
                || args.iter().any(|a| expr_contains_call(a, fn_name))
        }
        TypedExprKind::Binary { left, right, .. } => {
            expr_contains_call(left, fn_name) || expr_contains_call(right, fn_name)
        }
        TypedExprKind::Unary { operand, .. } => expr_contains_call(operand, fn_name),
        _ => false,
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c);
        } else {
            result.push(c.to_uppercase().next().unwrap());
        }
    }
    result
}

fn precedence(op: &BinaryOp) -> u8 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::And => 2,
        BinaryOp::Eq | BinaryOp::NotEq => 3,
        BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => 4,
        BinaryOp::Add | BinaryOp::Sub => 5,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 6,
        _ => 0,
    }
}

fn needs_parens(parent_op: &BinaryOp, child: &TypedExpr) -> bool {
    match &child.kind {
        TypedExprKind::Binary { op: child_op, .. } => precedence(child_op) < precedence(parent_op),
        _ => false,
    }
}
