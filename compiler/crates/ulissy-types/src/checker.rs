// ulissy-types/src/checker.rs
// ULissy Type Checker - Main Checking Logic
// Version 0.1.0

use crate::types::*;
use crate::{TypeContext, TypeError};
use ulissy_parser::ast;

// ============================================================================
// TYPE CHECKER
// ============================================================================

pub struct TypeChecker {
    context: TypeContext,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            context: TypeContext::new(),
            errors: Vec::new(),
        }
    }

    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<TypeError>> {
        // Pass 0: Register types and enums (forward declaration)
        for stmt in &program.statements {
            match stmt {
                ast::Statement::TypeDecl(decl) => {
                    // Register Type placeholder
                    self.context
                        .register_type(&decl.name, Type::Named(decl.name.clone()));
                }
                ast::Statement::EnumDecl(decl) => {
                    // Register Enum placeholder
                    self.context
                        .register_type(&decl.name, Type::Named(decl.name.clone()));
                    // Register Variants
                    for variant in &decl.variants {
                        self.context
                            .define(&variant.name, Type::Named(decl.name.clone()), true);
                    }
                }
                _ => {}
            }
        }

        // Pass 1: collect all function declarations for forward reference support
        for stmt in &program.statements {
            if let ast::Statement::FnDecl(fn_decl) = stmt {
                // Register function in scope before type checking bodies
                let return_type = if let Some(ref type_expr) = fn_decl.return_type {
                    self.resolve_type_expr(type_expr).unwrap_or(Type::Any)
                } else {
                    Type::Unit
                };

                let param_types: Vec<Type> = fn_decl
                    .params
                    .iter()
                    .map(|p| self.resolve_type_expr(&p.type_expr).unwrap_or(Type::Any))
                    .collect();

                self.context.define(
                    &fn_decl.name,
                    Type::Function {
                        params: param_types,
                        ret: Box::new(return_type),
                    },
                    false,
                );
            }
        }

        // Second pass: type check all statements
        let mut typed_statements = Vec::new();

        for stmt in &program.statements {
            match self.check_statement(stmt) {
                Ok(typed) => typed_statements.push(typed),
                Err(e) => self.errors.push(e),
            }
        }

        if self.errors.is_empty() {
            Ok(TypedProgram {
                statements: typed_statements,
            })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    // ========================================================================
    // STATEMENT CHECKING
    // ========================================================================

    fn check_statement(&mut self, stmt: &ast::Statement) -> Result<TypedStatement, TypeError> {
        match stmt {
            ast::Statement::IdentityDecl(decl) => self.check_identity_decl(decl),
            ast::Statement::LetDecl(decl) => self.check_let_decl(decl),
            ast::Statement::VarDecl(decl) => self.check_var_decl(decl),
            ast::Statement::EveryBlock(block) => self.check_every_block(block),
            ast::Statement::WhenBlock(block) => self.check_when_block(block),
            ast::Statement::SendStatement(send) => self.check_send_statement(send),
            ast::Statement::EnumDecl(decl) => self.check_enum_decl(decl),
            ast::Statement::ConfigBlock(config) => self.check_config_block(config),
            ast::Statement::ComputedPropertyDecl(computed) => {
                self.check_computed_property_decl(computed)
            }
            ast::Statement::TypeDecl(decl) => self.check_type_decl(decl),
            ast::Statement::ExpressionStatement(expr) => self.check_expression_statement(expr),
            ast::Statement::FnDecl(fn_decl) => self.check_fn_decl(fn_decl),
            ast::Statement::ReturnStatement(ret) => self.check_return_statement(ret),
            ast::Statement::IfStatement(if_stmt) => self.check_if_statement(if_stmt),
            ast::Statement::IfLetStatement(stmt) => self.check_if_let_statement(stmt),
            ast::Statement::ForStatement(for_stmt) => self.check_for_statement(for_stmt),
            ast::Statement::MatchStatement(match_stmt) => self.check_match_statement(match_stmt),
            ast::Statement::AfterBlock(after) => self.check_after_block(after),
            _ => {
                // TODO: Implement remaining statement types (Import)
                Ok(TypedStatement {
                    kind: TypedStatementKind::ExpressionStatement(TypedExpr {
                        kind: TypedExprKind::Literal(ast::Literal::Nil),
                        ty: Type::Unit,
                        span: ast::Span::default(),
                    }),
                    span: ast::Span::default(),
                })
            }
        }
    }

    fn check_identity_decl(
        &mut self,
        decl: &ast::IdentityDecl,
    ) -> Result<TypedStatement, TypeError> {
        // Check if already defined
        if self.context.is_defined_in_current_scope(&decl.name) {
            return Err(TypeError::new(
                &format!("'{}' is already defined in this scope", decl.name),
                decl.span,
            ));
        }

        // Check initializer - must resolve to Identity type
        let init = self.check_expression(&decl.initializer)?;

        if init.ty != Type::Identity && init.ty != Type::Any {
            return Err(TypeError::with_hint(
                &format!(
                    "Identity declaration requires Identity type, found {}",
                    init.ty
                ),
                "identity declarations must be initialized with Keychain.primary or similar",
                decl.span,
            ));
        }

        // Register in context
        self.context.define(&decl.name, Type::Identity, false);

        Ok(TypedStatement {
            kind: TypedStatementKind::IdentityDecl {
                name: decl.name.clone(),
                init,
            },
            span: decl.span,
        })
    }

    fn check_let_decl(&mut self, decl: &ast::LetDecl) -> Result<TypedStatement, TypeError> {
        if self.context.is_defined_in_current_scope(&decl.name) {
            return Err(TypeError::new(
                &format!("'{}' is already defined in this scope", decl.name),
                decl.span,
            ));
        }

        let init = self.check_expression(&decl.initializer)?;

        let declared_type = if let Some(type_expr) = &decl.type_annotation {
            Some(self.resolve_type_expr(type_expr)?)
        } else {
            None
        };

        let inferred_type = if let Some(ref declared) = declared_type {
            if !declared.is_assignable_from(&init.ty) {
                return Err(TypeError::new(
                    &format!("Cannot assign {} to {}", init.ty, declared),
                    decl.span,
                ));
            }
            declared.clone()
        } else {
            init.ty.clone()
        };

        self.context
            .define(&decl.name, inferred_type.clone(), false);

        Ok(TypedStatement {
            kind: TypedStatementKind::LetDecl {
                name: decl.name.clone(),
                declared_type,
                inferred_type,
                init,
            },
            span: decl.span,
        })
    }

    fn check_var_decl(&mut self, decl: &ast::VarDecl) -> Result<TypedStatement, TypeError> {
        if self.context.is_defined_in_current_scope(&decl.name) {
            return Err(TypeError::new(
                &format!("'{}' is already defined in this scope", decl.name),
                decl.span,
            ));
        }

        let init = if let Some(expr) = &decl.initializer {
            Some(self.check_expression(expr)?)
        } else {
            None
        };

        let declared_type = if let Some(type_expr) = &decl.type_annotation {
            Some(self.resolve_type_expr(type_expr)?)
        } else {
            None
        };

        let inferred_type = match (&declared_type, &init) {
            (Some(declared), Some(init_expr)) => {
                if !declared.is_assignable_from(&init_expr.ty) {
                    return Err(TypeError::new(
                        &format!("Cannot assign {} to {}", init_expr.ty, declared),
                        decl.span,
                    ));
                }
                declared.clone()
            }
            (Some(declared), None) => declared.clone(),
            (None, Some(init_expr)) => init_expr.ty.clone(),
            (None, None) => {
                return Err(TypeError::with_hint(
                    "Cannot infer type for variable without initializer or type annotation",
                    "add a type annotation: var x: Int",
                    decl.span,
                ));
            }
        };

        self.context.define(&decl.name, inferred_type.clone(), true);

        Ok(TypedStatement {
            kind: TypedStatementKind::VarDecl {
                name: decl.name.clone(),
                declared_type,
                inferred_type,
                init,
            },
            span: decl.span,
        })
    }

    fn check_every_block(&mut self, block: &ast::EveryBlock) -> Result<TypedStatement, TypeError> {
        // Interval must be Duration
        let interval = self.check_expression(&block.interval)?;
        if interval.ty != Type::Duration && interval.ty != Type::Any {
            return Err(TypeError::with_hint(
                &format!("'every' requires Duration, found {}", interval.ty),
                "use a duration like: every 10.minutes",
                block.span,
            ));
        }

        // Condition must be Bool
        let condition = if let Some(cond) = &block.condition {
            let typed = self.check_expression(cond)?;
            if typed.ty != Type::Bool && typed.ty != Type::Any {
                return Err(TypeError::new(
                    &format!("'when' condition must be Bool, found {}", typed.ty),
                    block.span,
                ));
            }
            Some(typed)
        } else {
            None
        };

        // Check body in new scope
        self.context.push_scope();
        let mut body = Vec::new();
        for stmt in &block.body.statements {
            body.push(self.check_statement(stmt)?);
        }
        self.context.pop_scope();

        Ok(TypedStatement {
            kind: TypedStatementKind::EveryBlock {
                interval,
                condition,
                body,
            },
            span: block.span,
        })
    }

    fn check_when_block(&mut self, block: &ast::WhenBlock) -> Result<TypedStatement, TypeError> {
        let condition = self.check_expression(&block.condition)?;

        if condition.ty != Type::Bool && condition.ty != Type::Any {
            return Err(TypeError::new(
                &format!("'when' condition must be Bool, found {}", condition.ty),
                block.span,
            ));
        }

        self.context.push_scope();
        let mut body = Vec::new();
        for stmt in &block.body.statements {
            body.push(self.check_statement(stmt)?);
        }
        self.context.pop_scope();

        Ok(TypedStatement {
            kind: TypedStatementKind::WhenBlock { condition, body },
            span: block.span,
        })
    }

    fn check_send_statement(
        &mut self,
        send: &ast::SendStatement,
    ) -> Result<TypedStatement, TypeError> {
        let recipient = self.check_expression(&send.recipient)?;

        // Recipient should be Handle or FacetAddress
        if recipient.ty != Type::Handle
            && recipient.ty != Type::FacetAddress
            && recipient.ty != Type::Any
        {
            return Err(TypeError::with_hint(
                &format!("'send to' requires Handle, found {}", recipient.ty),
                "use a handle like: send to @alice",
                send.span,
            ));
        }

        let mut fields = Vec::new();
        for (name, expr) in &send.fields {
            let typed = self.check_expression(expr)?;
            fields.push((name.clone(), typed));
        }

        Ok(TypedStatement {
            kind: TypedStatementKind::SendStatement { recipient, fields },
            span: send.span,
        })
    }

    /// Type check an enum declaration
    fn check_enum_decl(&mut self, decl: &ast::EnumDecl) -> Result<TypedStatement, TypeError> {
        let mut typed_variants = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        // Check each variant
        for variant in &decl.variants {
            // Ensure unique variant names
            if !seen_names.insert(&variant.name) {
                return Err(TypeError::new(
                    &format!("Duplicate enum variant '{}'", variant.name),
                    decl.span,
                ));
            }

            // Resolve associated types
            let associated_types = if let Some(type_exprs) = &variant.associated_types {
                let mut resolved = Vec::new();
                for type_expr in type_exprs {
                    resolved
                        .push(self.resolve_type_expr_with_params(type_expr, &decl.type_params)?);
                }
                Some(resolved)
            } else {
                None
            };

            typed_variants.push(TypedEnumVariant {
                name: variant.name.clone(),
                associated_types,
            });
        }

        // Create the enum type
        let enum_type = Type::Enum {
            name: decl.name.clone(),
            variants: typed_variants
                .iter()
                .map(|v| EnumVariantType {
                    name: v.name.clone(),
                    associated_types: v.associated_types.clone(),
                })
                .collect(),
        };

        // Register enum type in context
        self.context.register_type(&decl.name, enum_type);

        // Register variant constructors
        for variant in &typed_variants {
            let constructor_name = format!("{}.{}", decl.name, variant.name);
            let param_types = variant.associated_types.clone().unwrap_or_default();
            let return_type = Type::Named(decl.name.clone());

            self.context.define(
                &constructor_name,
                Type::Function {
                    params: param_types,
                    ret: Box::new(return_type),
                },
                false,
            );
        }

        Ok(TypedStatement {
            kind: TypedStatementKind::EnumDecl {
                name: decl.name.clone(),
                type_params: decl.type_params.clone(),
                variants: typed_variants,
            },
            span: decl.span,
        })
    }

    /// Resolve a type expression with type parameter support
    fn resolve_type_expr_with_params(
        &self,
        type_expr: &ast::TypeExpr,
        type_params: &[String],
    ) -> Result<Type, TypeError> {
        match type_expr {
            ast::TypeExpr::Simple(name) => {
                // Check if it's a type parameter (e.g., T, E)
                if type_params.contains(name) {
                    Ok(Type::Named(name.clone())) // Generic type parameter
                } else {
                    self.context.resolve_type(name).ok_or_else(|| {
                        TypeError::new(&format!("Unknown type: {}", name), ast::Span::default())
                    })
                }
            }
            ast::TypeExpr::Optional(inner) => {
                let inner_type = self.resolve_type_expr_with_params(inner, type_params)?;
                Ok(Type::Optional(Box::new(inner_type)))
            }
            ast::TypeExpr::Generic { name, params } => {
                let resolved_params: Result<Vec<_>, _> = params
                    .iter()
                    .map(|p| self.resolve_type_expr_with_params(p, type_params))
                    .collect();

                // For now, return a named type
                // Future: proper generic type instantiation
                Ok(Type::Named(format!(
                    "{}<{}>",
                    name,
                    resolved_params?
                        .iter()
                        .map(|t| format!("{:?}", t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )))
            }
            _ => Ok(Type::Any),
        }
    }

    /// Type check a config block
    fn check_config_block(
        &mut self,
        config: &ast::ConfigBlock,
    ) -> Result<TypedStatement, TypeError> {
        let mut typed_fields = Vec::new();

        for field in &config.fields {
            let typed_value = self.check_expression(&field.value)?;

            // Config values should be compile-time constants
            // For now, we allow any expression
            typed_fields.push(TypedConfigField {
                name: field.name.clone(),
                value: typed_value.clone(),
            });

            // Register in context as config.fieldName
            let config_field_name = format!("config.{}", field.name);
            self.context
                .define(&config_field_name, typed_value.ty, false);
        }

        Ok(TypedStatement {
            kind: TypedStatementKind::ConfigBlock {
                fields: typed_fields,
            },
            span: config.span,
        })
    }

    /// Type check a computed property declaration
    fn check_computed_property_decl(
        &mut self,
        computed: &ast::ComputedPropertyDecl,
    ) -> Result<TypedStatement, TypeError> {
        let expected_type = self.resolve_type_expr(&computed.type_annotation)?;

        let typed_body = match &computed.body {
            ast::ComputedBody::Expression(expr) => {
                let typed_expr = self.check_expression(expr)?;
                // TODO: Add type compatibility check
                TypedComputedBody::Expression(typed_expr)
            }
            ast::ComputedBody::ObjectFields(fields) => {
                let mut typed_fields = Vec::new();
                for field in fields {
                    let (typed_value, field_type) = if let Some(value) = &field.value {
                        let typed = self.check_expression(value)?;
                        let ty = typed.ty.clone();
                        (typed, ty)
                    } else {
                        // Shorthand: { x } means { x: x }
                        if let Some(sym) = self.context.lookup(&field.name) {
                            let ty = sym.ty.clone();
                            let typed = TypedExpr {
                                kind: TypedExprKind::Identifier(field.name.clone()),
                                ty: ty.clone(),
                                span: field.span,
                            };
                            (typed, ty)
                        } else {
                            return Err(TypeError::new(
                                &format!(
                                    "Unknown variable '{}' in computed property shorthand",
                                    field.name
                                ),
                                field.span,
                            ));
                        }
                    };

                    typed_fields.push(TypedObjectField {
                        name: field.name.clone(),
                        value: typed_value,
                        field_type,
                    });
                }
                TypedComputedBody::ObjectFields(typed_fields)
            }
        };

        // Register the computed property in scope
        self.context
            .define(&computed.name, expected_type.clone(), false);

        Ok(TypedStatement {
            kind: TypedStatementKind::ComputedPropertyDecl {
                name: computed.name.clone(),
                inferred_type: expected_type,
                body: typed_body,
            },
            span: computed.span,
        })
    }

    fn check_type_decl(&mut self, decl: &ast::TypeDecl) -> Result<TypedStatement, TypeError> {
        let mut fields = Vec::new();
        for field in &decl.fields {
            let ty = self.resolve_type_expr(&field.type_expr)?;
            println!(
                "DEBUG: TypeDecl {} field {} resolved to {:?}",
                decl.name, field.name, ty
            );
            fields.push((field.name.clone(), ty));
        }

        // Register the new type in the context
        self.context.register_type(
            &decl.name,
            Type::Object {
                fields: fields.clone(),
            },
        );

        Ok(TypedStatement {
            kind: TypedStatementKind::TypeDecl {
                name: decl.name.clone(),
                fields,
            },
            span: decl.span,
        })
    }

    fn check_expression_statement(
        &mut self,
        expr: &ast::Expression,
    ) -> Result<TypedStatement, TypeError> {
        let span = self.get_expr_span(expr);
        let typed = self.check_expression(expr)?;
        Ok(TypedStatement {
            kind: TypedStatementKind::ExpressionStatement(typed),
            span,
        })
    }

    fn check_fn_decl(&mut self, fn_decl: &ast::FnDecl) -> Result<TypedStatement, TypeError> {
        // Enter a new scope for the function
        self.context.push_scope();

        // Process parameters
        let mut typed_params = Vec::new();
        for param in &fn_decl.params {
            let param_type = self
                .resolve_type_expr(&param.type_expr)
                .unwrap_or(Type::Any);
            self.context.define(&param.name, param_type.clone(), false);
            typed_params.push(TypedParam {
                name: param.name.clone(),
                param_type,
            });
        }

        // Resolve return type
        let return_type = if let Some(ref type_expr) = fn_decl.return_type {
            self.resolve_type_expr(type_expr).unwrap_or(Type::Any)
        } else {
            Type::Unit
        };

        // Type check the function body
        let mut typed_body = Vec::new();
        for stmt in &fn_decl.body.statements {
            match self.check_statement(stmt) {
                Ok(typed) => typed_body.push(typed),
                Err(e) => self.errors.push(e),
            }
        }

        // Exit function scope
        self.context.pop_scope();

        Ok(TypedStatement {
            kind: TypedStatementKind::FnDecl {
                name: fn_decl.name.clone(),
                params: typed_params,
                return_type,
                body: typed_body,
            },
            span: fn_decl.span,
        })
    }

    fn check_return_statement(
        &mut self,
        ret: &ast::ReturnStatement,
    ) -> Result<TypedStatement, TypeError> {
        let typed_expr = if let Some(expr) = &ret.value {
            Some(self.check_expression(expr)?)
        } else {
            None
        };

        Ok(TypedStatement {
            kind: TypedStatementKind::ReturnStatement(typed_expr),
            span: ret.span,
        })
    }

    fn check_if_statement(&mut self, stmt: &ast::IfStatement) -> Result<TypedStatement, TypeError> {
        let condition = self.check_expression(&stmt.condition)?;
        if condition.ty != Type::Bool && condition.ty != Type::Any {
            return Err(TypeError::new(
                &format!("If condition must be Bool, found {}", condition.ty),
                stmt.span,
            ));
        }

        let mut then_block = Vec::new();
        self.context.push_scope();
        for s in &stmt.then_branch.statements {
            match self.check_statement(s) {
                Ok(t) => then_block.push(t),
                Err(e) => self.errors.push(e),
            }
        }
        self.context.pop_scope();

        let else_block = if let Some(branch) = &stmt.else_branch {
            // Handle Else or ElseIf
            let stmts = match &**branch {
                ast::ElseBranch::Else(block) => {
                    self.context.push_scope();
                    let mut stmts = Vec::new();
                    for s in &block.statements {
                        match self.check_statement(s) {
                            Ok(t) => stmts.push(t),
                            Err(e) => self.errors.push(e),
                        }
                    }
                    self.context.pop_scope();
                    stmts
                }
                ast::ElseBranch::ElseIf(nested_if) => {
                    // Recursively check
                    vec![self.check_if_statement(nested_if)?]
                }
                ast::ElseBranch::ElseIfLet(nested_if_let) => {
                    vec![self.check_if_let_statement(nested_if_let)?]
                }
            };
            Some(stmts)
        } else {
            None
        };

        Ok(TypedStatement {
            kind: TypedStatementKind::IfStatement {
                condition,
                then_block,
                else_block,
            },
            span: stmt.span,
        })
    }

    fn check_if_let_statement(
        &mut self,
        stmt: &ast::IfLetStatement,
    ) -> Result<TypedStatement, TypeError> {
        let typed_value = self.check_expression(&stmt.value)?;

        // Value must be Optional<T>
        let inner_type = match &typed_value.ty {
            Type::Optional(inner) => inner.as_ref().clone(),
            _ => {
                return Err(TypeError::new(
                    &format!("if let requires an Optional type, found {}", typed_value.ty),
                    stmt.span,
                ));
            }
        };

        // Type-check then branch with binding in scope as the UNWRAPPED type
        self.context.push_scope();
        self.context
            .define(&stmt.binding, inner_type.clone(), false);

        let mut then_block = Vec::new();
        for s in &stmt.then_branch.statements {
            match self.check_statement(s) {
                Ok(t) => then_block.push(t),
                Err(e) => self.errors.push(e),
            }
        }
        self.context.pop_scope();

        // Type-check else branch (binding is NOT in scope here)
        let else_block = if let Some(branch) = &stmt.else_branch {
            let stmts = match &**branch {
                ast::ElseBranch::Else(block) => {
                    self.context.push_scope();
                    let mut stmts = Vec::new();
                    for s in &block.statements {
                        match self.check_statement(s) {
                            Ok(t) => stmts.push(t),
                            Err(e) => self.errors.push(e),
                        }
                    }
                    self.context.pop_scope();
                    stmts
                }
                ast::ElseBranch::ElseIf(nested_if) => {
                    vec![self.check_if_statement(nested_if)?]
                }
                ast::ElseBranch::ElseIfLet(nested_if_let) => {
                    vec![self.check_if_let_statement(nested_if_let)?]
                }
            };
            Some(stmts)
        } else {
            None
        };

        Ok(TypedStatement {
            kind: TypedStatementKind::IfLetStatement {
                binding: stmt.binding.clone(),
                binding_type: inner_type,
                value: typed_value,
                then_block,
                else_block,
            },
            span: stmt.span,
        })
    }

    fn check_for_statement(
        &mut self,
        stmt: &ast::ForStatement,
    ) -> Result<TypedStatement, TypeError> {
        let collection = self.check_expression(&stmt.iterable)?;

        // Determine item type
        let item_type = match &collection.ty {
            Type::Array(inner) => *inner.clone(),
            Type::SearchResultSet => Type::SearchResult,
            Type::Any => Type::Any,
            _ => {
                return Err(TypeError::new(
                    &format!("Cannot iterate over {}", collection.ty),
                    stmt.span,
                ))
            }
        };

        let mut body = Vec::new();
        self.context.push_scope();
        self.context.define(&stmt.variable, item_type, false);

        for s in &stmt.body.statements {
            match self.check_statement(s) {
                Ok(t) => body.push(t),
                Err(e) => self.errors.push(e),
            }
        }
        self.context.pop_scope();

        Ok(TypedStatement {
            kind: TypedStatementKind::ForStatement {
                item_name: stmt.variable.clone(),
                collection,
                body,
            },
            span: stmt.span,
        })
    }

    fn check_match_statement(
        &mut self,
        stmt: &ast::MatchStatement,
    ) -> Result<TypedStatement, TypeError> {
        let expr = self.check_expression(&stmt.subject)?;

        let mut cases = Vec::new();
        for case in &stmt.cases {
            // New scope for each case (variables bound in pattern would go here)
            self.context.push_scope();

            // TODO: Bind pattern variables

            let guard = if let Some(g) = &case.guard {
                Some(self.check_expression(g)?)
            } else {
                None
            };

            let mut body = Vec::new();
            for s in &case.body.statements {
                match self.check_statement(s) {
                    Ok(t) => body.push(t),
                    Err(e) => self.errors.push(e),
                }
            }

            cases.push(TypedMatchCase {
                pattern: case.pattern.clone(),
                guard,
                body,
            });

            self.context.pop_scope();
        }

        Ok(TypedStatement {
            kind: TypedStatementKind::MatchStatement { expr, cases },
            span: stmt.span,
        })
    }

    fn check_after_block(&mut self, block: &ast::AfterBlock) -> Result<TypedStatement, TypeError> {
        let delay = self.check_expression(&block.delay)?;

        let mut body = Vec::new();
        self.context.push_scope();
        for s in &block.body.statements {
            match self.check_statement(s) {
                Ok(t) => body.push(t),
                Err(e) => self.errors.push(e),
            }
        }
        self.context.pop_scope();

        Ok(TypedStatement {
            kind: TypedStatementKind::AfterBlock { delay, body },
            span: block.span,
        })
    }

    // ========================================================================
    // EXPRESSION CHECKING
    // ========================================================================

    fn check_expression(&mut self, expr: &ast::Expression) -> Result<TypedExpr, TypeError> {
        let span = self.get_expr_span(expr);

        match expr {
            ast::Expression::Search(search) => self.check_search_expression(search),
            ast::Expression::Literal(lit) => {
                let ty = match lit {
                    ast::Literal::Int(_) => Type::Int,
                    ast::Literal::Float(_) => Type::Float,
                    ast::Literal::String(_) => Type::String,
                    ast::Literal::Bool(_) => Type::Bool,
                    ast::Literal::Nil => Type::Nil,
                };
                Ok(TypedExpr {
                    kind: TypedExprKind::Literal(lit.clone()),
                    ty,
                    span,
                })
            }

            ast::Expression::Identifier(name) => {
                // Handle enum shorthand (.public, .private)
                if name.starts_with('.') {
                    return Ok(TypedExpr {
                        kind: TypedExprKind::Identifier(name.clone()),
                        ty: Type::Any, // Enum variants need context to type
                        span,
                    });
                }

                if let Some(symbol) = self.context.lookup(name) {
                    Ok(TypedExpr {
                        kind: TypedExprKind::Identifier(name.clone()),
                        ty: symbol.ty.clone(),
                        span,
                    })
                } else {
                    Err(TypeError::with_hint(
                        &format!("Undefined variable: '{}'", name),
                        "variables must be declared before use",
                        span,
                    ))
                }
            }

            ast::Expression::Handle(h) => Ok(TypedExpr {
                kind: TypedExprKind::Handle(h.clone()),
                ty: Type::Handle,
                span,
            }),

            ast::Expression::FacetAddress { prefix, handle } => Ok(TypedExpr {
                kind: TypedExprKind::FacetAddress {
                    prefix: prefix.clone(),
                    handle: handle.clone(),
                },
                ty: Type::FacetAddress,
                span,
            }),

            ast::Expression::FacetPath {
                prefix,
                handle,
                path,
            } => Ok(TypedExpr {
                kind: TypedExprKind::FacetAddress {
                    prefix: prefix.clone(),
                    handle: format!("{}/{}", handle, path),
                },
                ty: Type::FacetAddress,
                span,
            }),

            ast::Expression::Binary(binary) => {
                let left = self.check_expression(&binary.left)?;
                let right = self.check_expression(&binary.right)?;

                let ty = left
                    .ty
                    .binary_result(&binary.operator, &right.ty)
                    .ok_or_else(|| {
                        TypeError::new(
                            &format!(
                                "Cannot apply {} to {} and {}",
                                binary.operator, left.ty, right.ty
                            ),
                            span,
                        )
                    })?;

                Ok(TypedExpr {
                    kind: TypedExprKind::Binary {
                        left: Box::new(left),
                        op: binary.operator,
                        right: Box::new(right),
                    },
                    ty,
                    span,
                })
            }

            ast::Expression::Unary(unary) => {
                let operand = self.check_expression(&unary.operand)?;

                let ty = match unary.operator {
                    ast::UnaryOp::Neg => {
                        if operand.ty == Type::Any {
                            Type::Any
                        } else if operand.ty.is_numeric() {
                            operand.ty.clone()
                        } else {
                            return Err(TypeError::new(
                                &format!("Cannot negate {}", operand.ty),
                                span,
                            ));
                        }
                    }
                    ast::UnaryOp::Not => {
                        if operand.ty == Type::Any {
                            Type::Bool
                        } else if operand.ty == Type::Bool {
                            Type::Bool
                        } else {
                            return Err(TypeError::new(
                                &format!("Cannot apply ! to {}", operand.ty),
                                span,
                            ));
                        }
                    }
                };

                Ok(TypedExpr {
                    kind: TypedExprKind::Unary {
                        op: unary.operator,
                        operand: Box::new(operand),
                    },
                    ty,
                    span,
                })
            }

            ast::Expression::Member(member) => {
                let object = self.check_expression(&member.object)?;

                let ty = self
                    .lookup_member_type(&object.ty, &member.member)
                    .ok_or_else(|| {
                        TypeError::with_hint(
                            &format!("'{}' has no member '{}'", object.ty, member.member),
                            &format!("Available members: {}", self.list_members(&object.ty)),
                            span,
                        )
                    })?;

                Ok(TypedExpr {
                    kind: TypedExprKind::Member {
                        object: Box::new(object),
                        member: member.member.clone(),
                    },
                    ty,
                    span,
                })
            }

            ast::Expression::Call(call) => {
                let callee = self.check_expression(&call.callee)?;

                let ret_type = match &callee.ty {
                    Type::Function { ret, .. } => (**ret).clone(),
                    _ => Type::Any, // Be lenient for now
                };

                let mut args = Vec::new();
                for arg in &call.arguments {
                    args.push(self.check_expression(&arg.value)?);
                }

                Ok(TypedExpr {
                    kind: TypedExprKind::Call {
                        callee: Box::new(callee),
                        args,
                    },
                    ty: ret_type,
                    span,
                })
            }

            ast::Expression::MethodCall(method) => {
                let object = self.check_expression(&method.object)?;

                let method_type = self.context.get_member_type(&object.ty, &method.method);
                let ret_type = match method_type {
                    Some(Type::Function { ret, .. }) => (*ret).clone(),
                    _ => Type::Any,
                };

                let mut args = Vec::new();
                for arg in &method.arguments {
                    args.push(self.check_expression(&arg.value)?);
                }

                Ok(TypedExpr {
                    kind: TypedExprKind::MethodCall {
                        object: Box::new(object),
                        method: method.method.clone(),
                        args,
                    },
                    ty: ret_type,
                    span,
                })
            }

            ast::Expression::UnitValue(unit) => {
                let value = self.check_expression(&unit.value)?;

                let ty = self
                    .context
                    .get_member_type(&value.ty, &unit.unit)
                    .unwrap_or(Type::Any);

                Ok(TypedExpr {
                    kind: TypedExprKind::UnitValue {
                        value: Box::new(value),
                        unit: unit.unit.clone(),
                    },
                    ty,
                    span,
                })
            }

            ast::Expression::Array(elements) => {
                let mut typed_elements = Vec::new();
                let mut element_type = Type::Unknown;

                for elem in elements {
                    let typed = self.check_expression(elem)?;
                    if element_type == Type::Unknown {
                        element_type = typed.ty.clone();
                    }
                    typed_elements.push(typed);
                }

                Ok(TypedExpr {
                    kind: TypedExprKind::Array(typed_elements),
                    ty: Type::Array(Box::new(element_type)),
                    span,
                })
            }

            ast::Expression::Grouped(inner) => self.check_expression(inner),

            ast::Expression::OptionalMember(om) => {
                self.check_optional_member(&om.object, &om.member, om.span)
            }

            ast::Expression::OptionalMethodCall(omc) => {
                self.check_optional_method_call(&omc.object, &omc.method, &omc.arguments, omc.span)
            }

            ast::Expression::NilCoalescing(nc) => {
                self.check_nil_coalescing(&nc.primary, &nc.fallback, nc.span)
            }

            ast::Expression::ObjectLiteral(obj) => self.check_object_literal(obj),

            ast::Expression::InterpolatedString(interp) => self.check_interpolated_string(interp),

            _ => {
                // Fallback for unimplemented expression types
                Ok(TypedExpr {
                    kind: TypedExprKind::Literal(ast::Literal::Nil),
                    ty: Type::Any,
                    span,
                })
            }
        }
    }

    // ========================================================================
    // INTERPOLATED STRINGS
    // ========================================================================

    /// Type check an interpolated string: "Hello, \(name)!"
    fn check_interpolated_string(
        &mut self,
        interp: &ast::InterpolatedStringExpr,
    ) -> Result<TypedExpr, TypeError> {
        let mut typed_parts = Vec::new();

        for part in &interp.parts {
            match part {
                ast::InterpolatedPart::Literal(s) => {
                    typed_parts.push(TypedInterpolatedPart::Literal(s.clone()));
                }
                ast::InterpolatedPart::Expression(expr) => {
                    let typed_expr = self.check_expression(expr)?;
                    typed_parts.push(TypedInterpolatedPart::Expression(typed_expr));
                }
            }
        }

        Ok(TypedExpr {
            kind: TypedExprKind::InterpolatedString { parts: typed_parts },
            ty: Type::String,
            span: interp.span,
        })
    }

    // ========================================================================
    // OBJECT LITERALS
    // ========================================================================

    /// Type check an object literal: { field1: value1, field2: value2 }
    fn check_object_literal(
        &mut self,
        obj: &ast::ObjectLiteralExpr,
    ) -> Result<TypedExpr, TypeError> {
        let mut typed_fields = Vec::new();
        let mut field_types = Vec::new();

        for field in &obj.fields {
            let (typed_value, field_type) = if let Some(value) = &field.value {
                // Explicit value: { x: 10 }
                let typed = self.check_expression(value)?;
                let ty = typed.ty.clone();
                (typed, ty)
            } else {
                // Shorthand: { x } means { x: x }
                // Look up identifier in scope
                let sym = self.context.lookup(&field.name).ok_or_else(|| {
                    TypeError::with_hint(
                        &format!("Unknown variable '{}' in object shorthand", field.name),
                        &format!("Did you mean {{ {}: <value> }}?", field.name),
                        field.span,
                    )
                })?;
                let ty = sym.ty.clone();

                let typed = TypedExpr {
                    kind: TypedExprKind::Identifier(field.name.clone()),
                    ty: ty.clone(),
                    span: field.span,
                };
                (typed, ty)
            };

            typed_fields.push(TypedObjectField {
                name: field.name.clone(),
                value: typed_value,
                field_type: field_type.clone(),
            });

            field_types.push((field.name.clone(), field_type));
        }

        // Construct the object type
        let object_type = if let Some(type_name) = &obj.type_hint {
            Type::Named(type_name.clone())
        } else {
            Type::Object {
                fields: field_types,
            }
        };

        Ok(TypedExpr {
            kind: TypedExprKind::ObjectLiteral {
                fields: typed_fields,
                type_hint: obj.type_hint.clone(),
            },
            ty: object_type,
            span: obj.span,
        })
    }

    // ========================================================================
    // OPTIONAL CHAINING & NIL COALESCING
    // ========================================================================

    /// Type check an optional member access: obj?.member
    ///
    /// Rules:
    /// - obj can be T or Optional<T>
    /// - Result is always Optional<MemberType>
    fn check_optional_member(
        &mut self,
        object: &ast::Expression,
        member: &str,
        span: ast::Span,
    ) -> Result<TypedExpr, TypeError> {
        let typed_object = self.check_expression(object)?;

        // Unwrap if optional, otherwise use directly
        let inner_type = match &typed_object.ty {
            Type::Optional(inner) => inner.as_ref().clone(),
            other => other.clone(),
        };

        // Look up the member on the inner type
        let member_type = self
            .lookup_member_type(&inner_type, member)
            .ok_or_else(|| {
                TypeError::with_hint(
                    &format!("Type '{}' has no member '{}'", inner_type, member),
                    &format!("Available members: {}", self.list_members(&inner_type)),
                    span,
                )
            })?;

        // Result is always Optional (the ?. propagates None)
        let result_type = Type::Optional(Box::new(member_type));

        Ok(TypedExpr {
            kind: TypedExprKind::OptionalMember {
                object: Box::new(typed_object),
                member: member.to_string(),
            },
            ty: result_type,
            span,
        })
    }

    /// Type check an optional method call: obj?.method(args)
    fn check_optional_method_call(
        &mut self,
        object: &ast::Expression,
        method: &str,
        arguments: &[ast::Argument],
        span: ast::Span,
    ) -> Result<TypedExpr, TypeError> {
        let typed_object = self.check_expression(object)?;

        // Unwrap if optional
        let inner_type = match &typed_object.ty {
            Type::Optional(inner) => inner.as_ref().clone(),
            other => other.clone(),
        };

        // Look up method signature
        let return_type = self
            .lookup_method_return_type(&inner_type, method)
            .unwrap_or(Type::Any);

        // Type check arguments
        let mut typed_args = Vec::new();
        for arg in arguments {
            typed_args.push(self.check_expression(&arg.value)?);
        }

        // Result is Optional<ReturnType>
        let result_type = Type::Optional(Box::new(return_type));

        Ok(TypedExpr {
            kind: TypedExprKind::OptionalMethodCall {
                object: Box::new(typed_object),
                method: method.to_string(),
                args: typed_args,
            },
            ty: result_type,
            span,
        })
    }

    /// Type-check search expression with privacy enforcement
    fn check_search_expression(
        &mut self,
        search: &ast::SearchExpr,
    ) -> Result<TypedExpr, TypeError> {
        // --- Check search target ---
        let typed_target = match &search.target {
            ast::SearchTarget::Nearby { radius } => {
                let typed_radius = if let Some(r) = radius {
                    let tr = self.check_expression(r)?;
                    if tr.ty != Type::Distance && tr.ty != Type::Int && tr.ty != Type::Float {
                        return Err(TypeError::new(
                            &format!(
                                "Nearby radius must be Distance, Int or Float, found {}",
                                tr.ty
                            ),
                            search.span,
                        ));
                    }
                    Some(Box::new(tr))
                } else {
                    None
                };
                TypedSearchTarget::Nearby {
                    radius: typed_radius,
                }
            }

            ast::SearchTarget::Within { center, radius } => {
                let typed_center = self.check_expression(center)?;
                let typed_radius = self.check_expression(radius)?;

                // Privacy: center CANNOT be Breadcrumb, Trajectory, or PrivateKey
                self.reject_private_in_search(
                    &typed_center.ty,
                    search.span,
                    "search within() center",
                )?;

                // Validate center type
                match &typed_center.ty {
                    Type::H3Cell | Type::Coordinates | Type::Handle | Type::String => {}
                    _ => return Err(TypeError::new(
                        &format!("search within() center must be H3Cell, Coordinates, Handle, or String, found {}", typed_center.ty),
                        search.span
                    )),
                }

                if typed_radius.ty != Type::Distance
                    && typed_radius.ty != Type::Int
                    && typed_radius.ty != Type::Float
                {
                    return Err(TypeError::new(
                        &format!(
                            "search within() radius must be Distance, Int or Float, found {}",
                            typed_radius.ty
                        ),
                        search.span,
                    ));
                }

                TypedSearchTarget::Within {
                    center: Box::new(typed_center),
                    radius: Box::new(typed_radius),
                }
            }

            ast::SearchTarget::Identity { handle } => {
                let typed_handle = self.check_expression(handle)?;
                match &typed_handle.ty {
                    Type::Handle | Type::FacetAddress | Type::String => {}
                    _ => return Err(TypeError::new(
                        &format!("search identity target must be Handle, FacetAddress, or String, found {}", typed_handle.ty),
                        search.span
                    )),
                }
                TypedSearchTarget::Identity {
                    handle: Box::new(typed_handle),
                }
            }

            ast::SearchTarget::Text { query } => {
                let typed_query = self.check_expression(query)?;
                if typed_query.ty != Type::String {
                    return Err(TypeError::new(
                        &format!("search text query must be String, found {}", typed_query.ty),
                        search.span,
                    ));
                }
                TypedSearchTarget::Text {
                    query: Box::new(typed_query),
                }
            }
        };

        // --- Check filters ---
        let mut typed_filters = Vec::new();
        for filter in &search.filters {
            let typed_filter = match filter {
                ast::SearchFilter::TrustThreshold { op, value } => {
                    let typed_value = self.check_expression(value)?;
                    self.reject_private_in_search(&typed_value.ty, search.span, "trust filter")?;
                    match &typed_value.ty {
                        Type::Float | Type::Int | Type::TrustScore => {}
                        _ => {
                            return Err(TypeError::new(
                                &format!(
                                    "Trust threshold must be numeric, found {}",
                                    typed_value.ty
                                ),
                                search.span,
                            ))
                        }
                    }
                    TypedSearchFilter::TrustThreshold {
                        op: *op,
                        value: Box::new(typed_value),
                    }
                }

                ast::SearchFilter::FacetMatch { facet_name } => {
                    let typed_name = self.check_expression(facet_name)?;
                    if typed_name.ty != Type::String {
                        return Err(TypeError::new(
                            &format!("Facet filter must be String, found {}", typed_name.ty),
                            search.span,
                        ));
                    }
                    TypedSearchFilter::FacetMatch {
                        facet_name: Box::new(typed_name),
                    }
                }

                ast::SearchFilter::ActiveWithin { duration } => {
                    let typed_duration = self.check_expression(duration)?;
                    if typed_duration.ty != Type::Duration
                        && typed_duration.ty != Type::Int
                        && typed_duration.ty != Type::Float
                    {
                        return Err(TypeError::new(
                            &format!(
                                "Active within filter must be Duration, found {}",
                                typed_duration.ty
                            ),
                            search.span,
                        ));
                    }
                    TypedSearchFilter::ActiveWithin {
                        duration: Box::new(typed_duration),
                    }
                }

                ast::SearchFilter::OrgMatch { org_name } => {
                    let typed_name = self.check_expression(org_name)?;
                    if typed_name.ty != Type::String {
                        return Err(TypeError::new(
                            &format!("Org filter must be String, found {}", typed_name.ty),
                            search.span,
                        ));
                    }
                    TypedSearchFilter::OrgMatch {
                        org_name: Box::new(typed_name),
                    }
                }

                ast::SearchFilter::FieldCompare { field, op, value } => {
                    // Backward-compatible generic filter
                    match field.as_str() {
                        "trust" | "distance" | "age" | "handlers" | "credentials" => {}
                        _ => return Err(TypeError::new(
                            &format!("Unknown search field '{}'. Use trust, distance, age, or typed filters (facet ==, active within, org ==)", field),
                            search.span
                        )),
                    }
                    let typed_value = self.check_expression(value)?;
                    self.reject_private_in_search(
                        &typed_value.ty,
                        search.span,
                        &format!("search filter '{}'", field),
                    )?;
                    TypedSearchFilter::FieldCompare {
                        field: field.clone(),
                        op: *op,
                        value: Box::new(typed_value),
                    }
                }
            };
            typed_filters.push(typed_filter);
        }

        // --- Check ranking ---
        let ranking = search.ranking.clone().map(|r| {
            // Convert AST ranking to typed ranking
            match &r {
                ast::SearchRanking::Trust { order } => TypedSearchRanking::Trust { order: *order },
                ast::SearchRanking::Distance { order } => {
                    TypedSearchRanking::Distance { order: *order }
                }
                ast::SearchRanking::Recency { order } => {
                    TypedSearchRanking::Recency { order: *order }
                }
                ast::SearchRanking::Relevance { order } => {
                    TypedSearchRanking::Relevance { order: *order }
                }
            }
        });

        // --- Determine result type ---
        // Identity lookups return Optional<SearchResult>, everything else returns SearchResultSet
        let result_type = match &search.target {
            ast::SearchTarget::Identity { .. } => Type::Optional(Box::new(Type::SearchResult)),
            _ => Type::SearchResultSet,
        };

        Ok(TypedExpr {
            kind: TypedExprKind::Search {
                target: typed_target,
                filters: typed_filters,
                ranking,
            },
            ty: result_type,
            span: search.span,
        })
    }

    /// Privacy gate: reject private spatial types in any search context
    fn reject_private_in_search(
        &self,
        ty: &Type,
        span: ast::Span,
        context: &str,
    ) -> Result<(), TypeError> {
        match ty {
            Type::Breadcrumb => Err(TypeError::new(
                &format!("PRIVACY VIOLATION: Cannot use Breadcrumb in {}. Breadcrumbs contain raw GPS — use H3Cell or Distance instead.", context),
                span
            )),
            Type::Trajectory => Err(TypeError::new(
                &format!("PRIVACY VIOLATION: Cannot use Trajectory in {}. Search operates on proofs, not paths.", context),
                span
            )),
            Type::PrivateKey => Err(TypeError::new(
                &format!("PRIVACY VIOLATION: Cannot use PrivateKey in {}.", context),
                span
            )),
            Type::Identity => Err(TypeError::new(
                &format!("PRIVACY VIOLATION: Cannot use Identity in {}. Use Handle for public lookups.", context),
                span
            )),
            Type::Array(inner) if matches!(**inner, Type::Breadcrumb | Type::Trajectory) => {
                Err(TypeError::new(
                    &format!("PRIVACY VIOLATION: Cannot use Array of private type in {}.", context),
                    span
                ))
            }
            _ => Ok(()),
        }
    }

    /// Type check nil coalescing: expr ?? default
    ///
    /// Rules:
    /// - primary must be Optional<T>
    /// - fallback must be compatible with T
    /// - Result is T (unwrapped)
    fn check_nil_coalescing(
        &mut self,
        primary: &ast::Expression,
        fallback: &ast::Expression,
        span: ast::Span,
    ) -> Result<TypedExpr, TypeError> {
        let typed_primary = self.check_expression(primary)?;
        let typed_fallback = self.check_expression(fallback)?;

        // Primary must be Optional<T>
        let inner_type = match &typed_primary.ty {
            Type::Optional(inner) => inner.as_ref().clone(),
            Type::Any | Type::Unknown => {
                // Infer from fallback
                typed_fallback.ty.clone()
            }
            other => {
                // Warning: ?? on non-optional is always the left side
                return Err(TypeError::with_hint(
                    &format!("Left side of '??' should be optional, found '{}'", other),
                    "The '??' operator is for providing default values for optional types. \
                     Either make the left side optional or remove the '??' operator.",
                    span,
                ));
            }
        };

        // Check fallback is compatible
        if !inner_type.is_assignable_from(&typed_fallback.ty) {
            return Err(TypeError::with_hint(
                &format!(
                    "Type mismatch in '??': expected '{}' but fallback is '{}'",
                    inner_type, typed_fallback.ty
                ),
                "The fallback value must match the optional's inner type",
                span,
            ));
        }

        // Result is the unwrapped type
        Ok(TypedExpr {
            kind: TypedExprKind::NilCoalescing {
                primary: Box::new(typed_primary),
                fallback: Box::new(typed_fallback),
            },
            ty: inner_type,
            span,
        })
    }

    /// Look up member type on a type
    fn lookup_member_type(&self, ty: &Type, member: &str) -> Option<Type> {
        match ty {
            Type::Trajectory => match member {
                "count" => Some(Type::Int),
                "last" => Some(Type::Optional(Box::new(Type::Breadcrumb))),
                "first" => Some(Type::Optional(Box::new(Type::Breadcrumb))),
                "pending" => Some(Type::Int),
                "uniqueCells" => Some(Type::Int),
                _ => None,
            },
            Type::Breadcrumb => match member {
                "hash" => Some(Type::Hash),
                "timestamp" => Some(Type::Moment),
                "h3Index" | "cell" => Some(Type::H3Cell),
                "signature" => Some(Type::Signature),
                "published" => Some(Type::Bool),
                "index" => Some(Type::Int),
                "previousHash" | "previous" => Some(Type::Hash),
                "owner" => Some(Type::Hash),
                "context" => Some(Type::Hash),
                _ => None,
            },
            Type::Identity => match member {
                "publicKey" => Some(Type::PublicKey),
                "trajectory" => Some(Type::Trajectory),
                "hasHandle" => Some(Type::Bool),
                "stellarAddress" => Some(Type::String),
                "handle" => Some(Type::Optional(Box::new(Type::Handle))),
                "trustScore" => Some(Type::Float),
                _ => None,
            },
            Type::SearchResultSet => match member {
                "count" => Some(Type::Int),
                _ => None,
            },
            Type::SearchResult => match member {
                "handle" => Some(Type::Handle),
                "distance" => Some(Type::Optional(Box::new(Type::Distance))),
                "trust" | "trust_score" => Some(Type::TrustScore),
                "age" | "last_active" => Some(Type::Moment),
                "tit" => Some(Type::Hash),
                "facets" => Some(Type::Array(Box::new(Type::FacetAddress))),
                "proof" => Some(Type::PresenceProof),
                "rank" => Some(Type::Float),
                _ => None,
            },
            Type::PresenceProof => match member {
                "epoch_count" => Some(Type::Int),
                "first_seen" => Some(Type::Moment),
                "spatial_diversity" => Some(Type::Float),
                "trajectory_continuity" => Some(Type::Float),
                "verification_level" => Some(Type::Int),
                _ => None,
            },
            _ => self.context.get_member_type(ty, member),
        }
    }

    fn list_members(&self, ty: &Type) -> String {
        match ty {
            Type::Trajectory => "count, last, first, pending, uniqueCells".to_string(),
            Type::Breadcrumb => "hash, timestamp, h3Index, cell, signature, published, index, previousHash, owner, context".to_string(),
            Type::Identity => "publicKey, trajectory, hasHandle, stellarAddress, handle, trustScore".to_string(),
            Type::SearchResultSet => "count".to_string(),
            Type::SearchResult => "handle, distance, trust, age, tit".to_string(),
            _ => "(unknown)".to_string(),
        }
    }

    fn lookup_method_return_type(&self, ty: &Type, method: &str) -> Option<Type> {
        match ty {
            Type::Breadcrumb => match method {
                "signed" => Some(Type::Breadcrumb),
                _ => None,
            },
            Type::Trajectory => match method {
                "append" => Some(Type::Unit),
                "bundleEpoch" => Some(Type::Named("Epoch".into())),
                _ => None,
            },
            _ => None,
        }
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    fn resolve_type_expr(&self, type_expr: &ast::TypeExpr) -> Result<Type, TypeError> {
        match type_expr {
            ast::TypeExpr::Simple(name) => {
                let resolved = self.context.resolve_type(name).ok_or_else(|| {
                    TypeError::new(&format!("Unknown type: {}", name), ast::Span::default())
                })?;

                // For helper types (Object/Enum), use Named reference to keep the name in generator
                match resolved {
                    Type::Object { .. } | Type::Enum { .. } => Ok(Type::Named(name.clone())),
                    _ => Ok(resolved),
                }
            }
            ast::TypeExpr::Optional(inner) => {
                let inner_type = self.resolve_type_expr(inner)?;
                Ok(Type::Optional(Box::new(inner_type)))
            }
            ast::TypeExpr::Generic { name, params } => {
                let mut resolved_params = Vec::new();
                for p in params {
                    resolved_params.push(self.resolve_type_expr(p)?);
                }

                match name.as_str() {
                    "Array" => Ok(Type::Array(Box::new(resolved_params.remove(0)))),
                    "Envelope" => Ok(Type::Envelope(Box::new(resolved_params.remove(0)))),
                    _ => Ok(Type::Named(name.clone())),
                }
            }
            _ => Ok(Type::Any),
        }
    }

    fn get_expr_span(&self, expr: &ast::Expression) -> ast::Span {
        match expr {
            ast::Expression::Binary(b) => b.span,
            ast::Expression::Unary(u) => u.span,
            ast::Expression::Member(m) => m.span,
            ast::Expression::Call(c) => c.span,
            ast::Expression::MethodCall(m) => m.span,
            ast::Expression::UnitValue(u) => u.span,
            ast::Expression::OptionalMember(om) => om.span,
            ast::Expression::OptionalMethodCall(omc) => omc.span,
            ast::Expression::NilCoalescing(nc) => nc.span,
            ast::Expression::Search(s) => s.span,
            _ => ast::Span::default(),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
