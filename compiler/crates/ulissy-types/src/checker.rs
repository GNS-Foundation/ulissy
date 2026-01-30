// ulissy-types/src/checker.rs
// ULissy Type Checker - Main Checking Logic
// Version 0.1.0

use crate::{TypeError, TypeContext};
use crate::types::*;
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
    
    pub fn check_program(&mut self, program: &ast::Program) -> Result<TypedProgram, Vec<TypeError>> {
        let mut typed_statements = Vec::new();
        
        for stmt in &program.statements {
            match self.check_statement(stmt) {
                Ok(typed) => typed_statements.push(typed),
                Err(e) => self.errors.push(e),
            }
        }
        
        if self.errors.is_empty() {
            Ok(TypedProgram { statements: typed_statements })
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
            ast::Statement::ComputedPropertyDecl(computed) => self.check_computed_property_decl(computed),
            ast::Statement::ExpressionStatement(expr) => self.check_expression_statement(expr),
            _ => {
                // TODO: Implement remaining statement types
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
    
    fn check_identity_decl(&mut self, decl: &ast::IdentityDecl) -> Result<TypedStatement, TypeError> {
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
                &format!("Identity declaration requires Identity type, found {}", init.ty),
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
        
        self.context.define(&decl.name, inferred_type.clone(), false);
        
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
            kind: TypedStatementKind::EveryBlock { interval, condition, body },
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
    
    fn check_send_statement(&mut self, send: &ast::SendStatement) -> Result<TypedStatement, TypeError> {
        let recipient = self.check_expression(&send.recipient)?;
        
        // Recipient should be Handle or FacetAddress
        if recipient.ty != Type::Handle && recipient.ty != Type::FacetAddress && recipient.ty != Type::Any {
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
                    resolved.push(self.resolve_type_expr_with_params(type_expr, &decl.type_params)?);
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
            variants: typed_variants.iter().map(|v| EnumVariantType {
                name: v.name.clone(),
                associated_types: v.associated_types.clone(),
            }).collect(),
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
                    self.context.resolve_type(name)
                        .ok_or_else(|| TypeError::new(
                            &format!("Unknown type: {}", name),
                            ast::Span::default(),
                        ))
                }
            }
            ast::TypeExpr::Optional(inner) => {
                let inner_type = self.resolve_type_expr_with_params(inner, type_params)?;
                Ok(Type::Optional(Box::new(inner_type)))
            }
            ast::TypeExpr::Generic { name, params } => {
                let resolved_params: Result<Vec<_>, _> = params.iter()
                    .map(|p| self.resolve_type_expr_with_params(p, type_params))
                    .collect();
                
                // For now, return a named type
                // Future: proper generic type instantiation
                Ok(Type::Named(format!("{}<{}>", name, 
                    resolved_params?.iter()
                        .map(|t| format!("{:?}", t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )))
            }
            _ => Ok(Type::Any),
        }
    }
    
    /// Type check a config block
    fn check_config_block(&mut self, config: &ast::ConfigBlock) -> Result<TypedStatement, TypeError> {
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
            self.context.define(&config_field_name, typed_value.ty, false);
        }
        
        Ok(TypedStatement {
            kind: TypedStatementKind::ConfigBlock { fields: typed_fields },
            span: config.span,
        })
    }
    
    /// Type check a computed property declaration
    fn check_computed_property_decl(&mut self, computed: &ast::ComputedPropertyDecl) -> Result<TypedStatement, TypeError> {
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
                                &format!("Unknown variable '{}' in computed property shorthand", field.name),
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
        self.context.define(&computed.name, expected_type.clone(), false);
        
        Ok(TypedStatement {
            kind: TypedStatementKind::ComputedPropertyDecl {
                name: computed.name.clone(),
                inferred_type: expected_type,
                body: typed_body,
            },
            span: computed.span,
        })
    }
    
    fn check_expression_statement(&mut self, expr: &ast::Expression) -> Result<TypedStatement, TypeError> {
        let span = self.get_expr_span(expr);
        let typed = self.check_expression(expr)?;
        Ok(TypedStatement {
            kind: TypedStatementKind::ExpressionStatement(typed),
            span,
        })
    }
    
    // ========================================================================
    // EXPRESSION CHECKING
    // ========================================================================
    
    fn check_expression(&mut self, expr: &ast::Expression) -> Result<TypedExpr, TypeError> {
        let span = self.get_expr_span(expr);
        
        match expr {
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
            
            ast::Expression::Handle(h) => {
                Ok(TypedExpr {
                    kind: TypedExprKind::Handle(h.clone()),
                    ty: Type::Handle,
                    span,
                })
            }
            
            ast::Expression::FacetAddress { prefix, handle } => {
                Ok(TypedExpr {
                    kind: TypedExprKind::FacetAddress {
                        prefix: prefix.clone(),
                        handle: handle.clone(),
                    },
                    ty: Type::FacetAddress,
                    span,
                })
            }
            
            ast::Expression::FacetPath { prefix, handle, path } => {
                Ok(TypedExpr {
                    kind: TypedExprKind::FacetAddress {
                        prefix: prefix.clone(),
                        handle: format!("{}/{}", handle, path),
                    },
                    ty: Type::FacetAddress,
                    span,
                })
            }
            
            ast::Expression::Binary(binary) => {
                let left = self.check_expression(&binary.left)?;
                let right = self.check_expression(&binary.right)?;
                
                let ty = left.ty.binary_result(&binary.operator, &right.ty)
                    .ok_or_else(|| TypeError::new(
                        &format!("Cannot apply {} to {} and {}", 
                            binary.operator, left.ty, right.ty),
                        span,
                    ))?;
                
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
                        if operand.ty.is_numeric() {
                            operand.ty.clone()
                        } else {
                            return Err(TypeError::new(
                                &format!("Cannot negate {}", operand.ty),
                                span,
                            ));
                        }
                    }
                    ast::UnaryOp::Not => {
                        if operand.ty == Type::Bool {
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
                
                let ty = self.context.get_member_type(&object.ty, &member.member)
                    .ok_or_else(|| TypeError::new(
                        &format!("'{}' has no member '{}'", object.ty, member.member),
                        span,
                    ))?;
                
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
                
                let ty = self.context.get_member_type(&value.ty, &unit.unit)
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
            
            ast::Expression::Grouped(inner) => {
                self.check_expression(inner)
            }
            
            ast::Expression::OptionalMember(om) => {
                self.check_optional_member(&om.object, &om.member, om.span)
            }
            
            ast::Expression::OptionalMethodCall(omc) => {
                self.check_optional_method_call(&omc.object, &omc.method, &omc.arguments, omc.span)
            }
            
            ast::Expression::NilCoalescing(nc) => {
                self.check_nil_coalescing(&nc.primary, &nc.fallback, nc.span)
            }
            
            ast::Expression::ObjectLiteral(obj) => {
                self.check_object_literal(obj)
            }
            
            ast::Expression::InterpolatedString(interp) => {
                self.check_interpolated_string(interp)
            }
            
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
                let sym = self.context.lookup(&field.name)
                    .ok_or_else(|| TypeError::with_hint(
                        &format!("Unknown variable '{}' in object shorthand", field.name),
                        &format!("Did you mean {{ {}: <value> }}?", field.name),
                        field.span,
                    ))?;
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
            Type::Object { fields: field_types }
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
        let member_type = self.lookup_member_type(&inner_type, member)
            .ok_or_else(|| TypeError::with_hint(
                &format!("Type '{}' has no member '{}'", inner_type, member),
                &format!("Available members: {}", self.list_members(&inner_type)),
                span,
            ))?;
        
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
        let return_type = self.lookup_method_return_type(&inner_type, method)
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
                    &format!(
                        "Left side of '??' should be optional, found '{}'",
                        other
                    ),
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
                "h3Index" => Some(Type::H3Cell),
                "signature" => Some(Type::Signature),
                "published" => Some(Type::Bool),
                _ => None,
            },
            Type::Identity => match member {
                "publicKey" => Some(Type::PublicKey),
                "trajectory" => Some(Type::Trajectory),
                "hasHandle" => Some(Type::Bool),
                _ => None,
            },
            _ => self.context.get_member_type(ty, member),
        }
    }
    
    fn list_members(&self, ty: &Type) -> String {
        match ty {
            Type::Trajectory => "count, last, first, pending, uniqueCells".to_string(),
            Type::Breadcrumb => "hash, timestamp, h3Index, signature, published".to_string(),
            Type::Identity => "publicKey, trajectory, hasHandle".to_string(),
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
                self.context.resolve_type(name)
                    .ok_or_else(|| TypeError::new(
                        &format!("Unknown type: {}", name),
                        ast::Span::default(),
                    ))
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
            _ => ast::Span::default(),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
