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
            _ => ast::Span::default(),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
