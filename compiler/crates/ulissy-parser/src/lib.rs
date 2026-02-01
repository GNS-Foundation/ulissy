// ulissy-parser/src/lib.rs
// ULissy Parser - Converts tokens to AST
// Version 0.1.0

pub mod ast;

use ast::*;
use ulissy_lexer::{Token, TokenKind, StringPart};
use std::fmt;

// ============================================================================
// PARSER ERROR
// ============================================================================

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub fn new(message: &str, line: usize, column: usize) -> Self {
        ParseError { message: message.to_string(), line, column }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// PARSER
// ============================================================================

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    /// Parse the entire program
    pub fn parse(mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        Ok(Program { statements })
    }

    // ========================================================================
    // STATEMENT PARSING
    // ========================================================================

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Identity) => self.parse_identity_decl(),
            Some(TokenKind::Let) => self.parse_let_decl(),
            Some(TokenKind::Var) => self.parse_var_decl(),
            Some(TokenKind::Const) => self.parse_const_decl(),
            Some(TokenKind::Fn) => self.parse_fn_decl(),
            Some(TokenKind::Type) => self.parse_type_decl(),
            Some(TokenKind::Enum) => self.parse_enum_decl(),
            Some(TokenKind::Every) => self.parse_every_block(),
            Some(TokenKind::When) => self.parse_when_block(),
            Some(TokenKind::After) => self.parse_after_block(),
            Some(TokenKind::Send) => self.parse_send_statement(),
            Some(TokenKind::If) => self.parse_if_statement(),
            Some(TokenKind::Match) => self.parse_match_statement(),
            Some(TokenKind::Return) => self.parse_return_statement(),
            Some(TokenKind::Import) => self.parse_import_statement(),
            Some(TokenKind::Config) => self.parse_config_block(),
            Some(TokenKind::Computed) => self.parse_computed_property_decl(),
            Some(TokenKind::For) => self.parse_for_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    /// identity me = Keychain.primary
    fn parse_identity_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Identity)?;
        
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Equal)?;
        let initializer = self.parse_expression()?;
        
        Ok(Statement::IdentityDecl(IdentityDecl {
            name,
            initializer,
            span: start,
        }))
    }

    /// let x = expr  OR  let x: Type = expr
    fn parse_let_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Let)?;
        
        let name = self.expect_identifier()?;
        
        let type_annotation = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Equal)?;
        let initializer = self.parse_expression()?;
        
        Ok(Statement::LetDecl(LetDecl {
            name,
            type_annotation,
            initializer,
            span: start,
        }))
    }

    /// var x = expr
    fn parse_var_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Var)?;
        
        let name = self.expect_identifier()?;
        
        let type_annotation = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        
        let initializer = if self.check(TokenKind::Equal) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(Statement::VarDecl(VarDecl {
            name,
            type_annotation,
            initializer,
            span: start,
        }))
    }

    /// const X = expr
    fn parse_const_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Const)?;
        
        let name = self.expect_identifier()?;
        
        let type_annotation = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Equal)?;
        let initializer = self.parse_expression()?;
        
        Ok(Statement::ConstDecl(ConstDecl {
            name,
            type_annotation,
            initializer,
            span: start,
        }))
    }

    /// fn name(params) -> ReturnType { body }
    fn parse_fn_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        
        let is_async = if self.check(TokenKind::Async) {
            self.advance();
            true
        } else {
            false
        };
        
        self.expect(TokenKind::Fn)?;
        let name = self.expect_identifier()?;
        
        self.expect(TokenKind::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RightParen)?;
        
        let return_type = if self.check(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        
        let constraints = if self.check(TokenKind::Where) {
            self.parse_where_clauses()?
        } else {
            Vec::new()
        };
        
        let body = self.parse_block()?;
        
        Ok(Statement::FnDecl(FnDecl {
            name,
            params,
            return_type,
            constraints,
            body,
            is_async,
            span: start,
        }))
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        
        if !self.check(TokenKind::RightParen) {
            loop {
                let name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let type_expr = self.parse_type_expr()?;
                
                let default_value = if self.check(TokenKind::Equal) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                params.push(Parameter { name, type_expr, default_value });
                
                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        Ok(params)
    }

    fn parse_where_clauses(&mut self) -> Result<Vec<WhereClause>, ParseError> {
        let mut clauses = Vec::new();
        self.expect(TokenKind::Where)?;
        
        loop {
            let expression = self.parse_expression()?;
            clauses.push(WhereClause { expression });
            
            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(clauses)
    }

    /// type Name { fields }
    fn parse_type_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Type)?;
        
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftBrace)?;
        
        let mut fields = Vec::new();
        let mut invariants = Vec::new();
        
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(TokenKind::Invariant) {
                self.advance();
                invariants.push(self.parse_expression()?);
            } else {
                let is_computed = if self.check(TokenKind::Computed) {
                    self.advance();
                    true
                } else {
                    false
                };
                
                let field_name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let type_expr = self.parse_type_expr()?;
                
                let default_value = if self.check(TokenKind::Equal) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                fields.push(FieldDecl {
                    name: field_name,
                    type_expr,
                    is_computed,
                    default_value,
                });
            }
            
            // Optional comma/newline between fields
            self.check(TokenKind::Comma).then(|| self.advance());
        }
        
        self.expect(TokenKind::RightBrace)?;
        
        Ok(Statement::TypeDecl(TypeDecl {
            name,
            fields,
            invariants,
            span: start,
        }))
    }

    /// enum Name { variant1, variant2, ... }
    ///
    /// Examples:
    /// ```ulissy
    /// enum LocationSource { gps, wifi, cell, ip, manual }
    /// enum Option<T> { some(T), none }
    /// enum Result<T, E> { ok(T), error(E) }
    /// ```
    fn parse_enum_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Enum)?;
        
        // Parse enum name
        let name = self.expect_identifier()?;
        
        // Parse optional type parameters: <T, E>
        let type_params = if self.check(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };
        
        // Expect opening brace
        self.expect(TokenKind::LeftBrace)?;
        
        // Parse variants
        let mut variants = Vec::new();
        
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let variant = self.parse_enum_variant()?;
            variants.push(variant);
            
            // Comma is optional between variants (allows trailing comma)
            if self.check(TokenKind::Comma) {
                self.advance();
            }
        }
        
        // Expect closing brace
        self.expect(TokenKind::RightBrace)?;
        
        // Validate: enum must have at least one variant
        if variants.is_empty() {
            return Err(self.error("Enum must have at least one variant"));
        }
        
        Ok(Statement::EnumDecl(EnumDecl {
            name,
            type_params,
            variants,
            span: start,
        }))
    }

    /// Parse a single enum variant: name or name(Type, Type)
    fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParseError> {
        let name = self.expect_identifier()?;
        
        // Check for associated types: variant(Type, Type)
        let associated_types = if self.check(TokenKind::LeftParen) {
            self.advance();
            
            // Empty parens: none()
            if self.check(TokenKind::RightParen) {
                self.advance();
                Some(Vec::new())
            } else {
                // Parse type list
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type_expr()?);
                    
                    if !self.check(TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                
                self.expect(TokenKind::RightParen)?;
                Some(types)
            }
        } else {
            None
        };
        
        Ok(EnumVariant {
            name,
            associated_types,
            named_fields: None, // Future: record-style variants
        })
    }

    /// Parse type parameters: <T, E, ...>
    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(TokenKind::Less)?;
        
        let mut params = Vec::new();
        loop {
            params.push(self.expect_identifier()?);
            
            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenKind::Greater)?;
        
        Ok(params)
    }

    /// every 10.minutes when condition { ... }
    fn parse_every_block(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Every)?;
        
        let interval = self.parse_expression()?;
        
        let condition = if self.check(TokenKind::When) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(Statement::EveryBlock(EveryBlock {
            interval,
            condition,
            body,
            span: start,
        }))
    }

    /// when condition { ... }
    fn parse_when_block(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::When)?;
        
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        
        Ok(Statement::WhenBlock(WhenBlock {
            condition,
            body,
            span: start,
        }))
    }

    /// after 5.seconds { ... }
    fn parse_after_block(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::After)?;
        
        let delay = self.parse_expression()?;
        let body = self.parse_block()?;
        
        Ok(Statement::AfterBlock(AfterBlock {
            delay,
            body,
            span: start,
        }))
    }

    /// send to @handle { message: "..." }
    fn parse_send_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Send)?;
        self.expect(TokenKind::To)?;
        
        let recipient = self.parse_expression()?;
        
        self.expect(TokenKind::LeftBrace)?;
        let mut fields = Vec::new();
        
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let field_name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expression()?;
            fields.push((field_name, value));
            
            // Optional comma
            self.check(TokenKind::Comma).then(|| self.advance());
        }
        
        self.expect(TokenKind::RightBrace)?;
        
        Ok(Statement::SendStatement(SendStatement {
            recipient,
            fields,
            span: start,
        }))
    }

    /// if condition { ... } else { ... }
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::If)?;
        
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        
        let else_branch = if self.check(TokenKind::Else) {
            self.advance();
            if self.check(TokenKind::If) {
                Some(Box::new(ElseBranch::ElseIf(
                    match self.parse_if_statement()? {
                        Statement::IfStatement(if_stmt) => if_stmt,
                        _ => unreachable!(),
                    }
                )))
            } else {
                Some(Box::new(ElseBranch::Else(self.parse_block()?)))
            }
        } else {
            None
        };
        
        Ok(Statement::IfStatement(IfStatement {
            condition,
            then_branch,
            else_branch,
            span: start,
        }))
    }

    /// match expr { case pattern: body }
    fn parse_match_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Match)?;
        
        let subject = self.parse_expression()?;
        self.expect(TokenKind::LeftBrace)?;
        
        let mut cases = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Handle both 'case pattern:' and 'default:'
            let pattern = if self.check(TokenKind::Default) {
                self.advance(); // consume 'default'
                Pattern::Wildcard
            } else {
                self.expect(TokenKind::Case)?;
                self.parse_pattern()?
            };
            
            let guard = if self.check(TokenKind::Where) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            
            cases.push(MatchCase { pattern, guard, body });
        }
        
        self.expect(TokenKind::RightBrace)?;
        
        Ok(Statement::MatchStatement(MatchStatement {
            subject,
            cases,
            span: start,
        }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                
                if self.check(TokenKind::LeftParen) {
                    self.advance();
                    let mut bindings = Vec::new();
                    while !self.check(TokenKind::RightParen) {
                        bindings.push(self.expect_identifier()?);
                        if !self.check(TokenKind::Comma) { break; }
                        self.advance();
                    }
                    self.expect(TokenKind::RightParen)?;
                    Ok(Pattern::EnumVariant { name, bindings })
                } else {
                    Ok(Pattern::Identifier(name))
                }
            }
            Some(TokenKind::IntLiteral(n)) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(n)))
            }
            Some(TokenKind::StringLiteral(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            _ => Err(self.error("Expected pattern")),
        }
    }

    /// return expr
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Return)?;
        
        let value = if !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(Statement::ReturnStatement(ReturnStatement { value, span: start }))
    }

    /// for item in collection { body }
    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::For)?;
        
        let variable = self.expect_identifier()?;
        self.expect(TokenKind::In)?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        
        Ok(Statement::ForStatement(ForStatement {
            variable,
            iterable,
            body,
            span: start,
        }))
    }

    /// import ulissy.spatial
    fn parse_import_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Import)?;
        
        let mut path = vec![self.expect_identifier()?];
        while self.check(TokenKind::Dot) {
            self.advance();
            path.push(self.expect_identifier()?);
        }
        
        let alias = if self.check(TokenKind::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        Ok(Statement::ImportStatement(ImportStatement { path, alias, span: start }))
    }

    /// config { resolution: 7, interval: 10.minutes }
    /// 
    /// Config blocks define module-level constants that are:
    /// - Evaluated at compile time when possible
    /// - Accessible throughout the module as `config.fieldName`
    fn parse_config_block(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Config)?;
        self.expect(TokenKind::LeftBrace)?;
        
        let mut fields = Vec::new();
        
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let field_span = self.current_span();
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expression()?;
            
            fields.push(ConfigField {
                name,
                value,
                span: field_span,
            });
            
            // Optional comma between fields
            if self.check(TokenKind::Comma) {
                self.advance();
            }
        }
        
        self.expect(TokenKind::RightBrace)?;
        
        Ok(Statement::ConfigBlock(ConfigBlock {
            fields,
            span: start,
        }))
    }

    /// computed status: CollectionStatus { isActive: running, count: total }
    /// OR
    /// computed total: Int = items.count
    /// 
    /// Standalone computed properties are reactive values that:
    /// - Automatically update when dependencies change
    /// - Can be accessed like regular properties
    fn parse_computed_property_decl(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::Computed)?;
        
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_annotation = self.parse_type_expr()?;
        
        // Two forms:
        // 1. computed x: Int = expr
        // 2. computed x: Type { field: expr, ... }
        
        let body = if self.check(TokenKind::Equal) {
            // Form 1: Single expression
            self.advance();
            let expr = self.parse_expression()?;
            ComputedBody::Expression(expr)
        } else if self.check(TokenKind::LeftBrace) {
            // Form 2: Object literal body
            self.advance();
            
            let mut fields = Vec::new();
            
            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                let field = self.parse_object_field()?;
                fields.push(field);
                
                if self.check(TokenKind::Comma) {
                    self.advance();
                }
            }
            
            self.expect(TokenKind::RightBrace)?;
            ComputedBody::ObjectFields(fields)
        } else {
            return Err(self.error("Expected '=' or '{' after computed property type"));
        };
        
        Ok(Statement::ComputedPropertyDecl(ComputedPropertyDecl {
            name,
            type_annotation,
            body,
            span: start,
        }))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression()?;
        Ok(Statement::ExpressionStatement(expr))
    }

    // ========================================================================
    // EXPRESSION PARSING (Pratt Parser / Precedence Climbing)
    // ========================================================================

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        // Assignment has lowest precedence
        self.parse_assignment_expr()
    }
    
    /// Parse assignment expression: target = value
    /// Assignment is right-associative: a = b = c becomes a = (b = c)
    fn parse_assignment_expr(&mut self) -> Result<Expression, ParseError> {
        let span = self.current_span();
        let left = self.parse_nil_coalescing_expr()?;
        
        if self.check(TokenKind::Equal) {
            self.advance(); // consume =
            let value = self.parse_assignment_expr()?; // right-associative
            return Ok(Expression::Assignment(Box::new(AssignmentExpr {
                target: left,
                value,
                span,
            })));
        }
        
        Ok(left)
    }

    /// Parse nil coalescing expression: expr ?? default
    /// 
    /// Precedence: Lower than || (or), so `a || b ?? c` means `(a || b) ?? c`
    /// 
    /// Example: `me.trajectory.last?.hash ?? "genesis"`
    fn parse_nil_coalescing_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_or_expr()?;
        
        while self.check(TokenKind::DoubleQuestion) {
            let span = self.current_span();
            self.advance(); // consume ??
            
            // Right side: parse at same precedence for right-associativity
            // a ?? b ?? c  =>  a ?? (b ?? c)
            let right = self.parse_nil_coalescing_expr()?;
            
            left = Expression::NilCoalescing(Box::new(NilCoalescingExpr {
                primary: left,
                fallback: right,
                span,
            }));
        }
        
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_and_expr()?;
        
        while self.check(TokenKind::Or) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::Or,
                right,
                span,
            }));
        }
        
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality_expr()?;
        
        while self.check(TokenKind::And) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_equality_expr()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::And,
                right,
                span,
            }));
        }
        
        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison_expr()?;
        
        while let Some(op) = self.match_equality_op() {
            let span = self.current_span();
            self.advance();
            let right = self.parse_comparison_expr()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left, operator: op, right, span,
            }));
        }
        
        Ok(left)
    }

    fn match_equality_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::EqualEqual) => Some(BinaryOp::Eq),
            Some(TokenKind::NotEqual) => Some(BinaryOp::NotEq),
            _ => None,
        }
    }

    fn parse_comparison_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive_expr()?;
        
        while let Some(op) = self.match_comparison_op() {
            let span = self.current_span();
            self.advance();
            let right = self.parse_additive_expr()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left, operator: op, right, span,
            }));
        }
        
        Ok(left)
    }

    fn match_comparison_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Less) => Some(BinaryOp::Lt),
            Some(TokenKind::Greater) => Some(BinaryOp::Gt),
            Some(TokenKind::LessEqual) => Some(BinaryOp::LtEq),
            Some(TokenKind::GreaterEqual) => Some(BinaryOp::GtEq),
            _ => None,
        }
    }

    fn parse_additive_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative_expr()?;
        
        while let Some(op) = self.match_additive_op() {
            let span = self.current_span();
            self.advance();
            let right = self.parse_multiplicative_expr()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left, operator: op, right, span,
            }));
        }
        
        Ok(left)
    }

    fn match_additive_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Plus) => Some(BinaryOp::Add),
            Some(TokenKind::Minus) => Some(BinaryOp::Sub),
            _ => None,
        }
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary_expr()?;
        
        while let Some(op) = self.match_multiplicative_op() {
            let span = self.current_span();
            self.advance();
            let right = self.parse_unary_expr()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left, operator: op, right, span,
            }));
        }
        
        Ok(left)
    }

    fn match_multiplicative_op(&self) -> Option<BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Star) => Some(BinaryOp::Mul),
            Some(TokenKind::Slash) => Some(BinaryOp::Div),
            Some(TokenKind::Percent) => Some(BinaryOp::Mod),
            _ => None,
        }
    }

    fn parse_unary_expr(&mut self) -> Result<Expression, ParseError> {
        if let Some(op) = self.match_unary_op() {
            let span = self.current_span();
            self.advance();
            let operand = self.parse_unary_expr()?;
            return Ok(Expression::Unary(Box::new(UnaryExpr {
                operator: op, operand, span,
            })));
        }
        
        self.parse_postfix_expr()
    }

    fn match_unary_op(&self) -> Option<UnaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Minus) => Some(UnaryOp::Neg),
            Some(TokenKind::Not) => Some(UnaryOp::Not),
            _ => None,
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary_expr()?;
        
        loop {
            // Regular member access (.)
            if self.check(TokenKind::Dot) {
                let span = self.current_span();
                self.advance();
                let member = self.expect_identifier()?;
                
                // Check for method call: obj.method(args)
                if self.check(TokenKind::LeftParen) {
                    self.advance();
                    let arguments = self.parse_arguments()?;
                    self.expect(TokenKind::RightParen)?;
                    expr = Expression::MethodCall(Box::new(MethodCallExpr {
                        object: expr,
                        method: member,
                        arguments,
                        span,
                    }));
                } else {
                    // Check for unit suffix: 10.minutes
                    if let Expression::Literal(Literal::Int(_)) | Expression::Literal(Literal::Float(_)) = &expr {
                        expr = Expression::UnitValue(Box::new(UnitValueExpr {
                            value: expr,
                            unit: member,
                            span,
                        }));
                    } else {
                        expr = Expression::Member(Box::new(MemberExpr {
                            object: expr,
                            member,
                            span,
                        }));
                    }
                }
            }
            // Optional chaining (?.)
            else if self.check(TokenKind::QuestionDot) {
                let span = self.current_span();
                self.advance(); // consume ?.
                let member = self.expect_identifier()?;
                
                // Check for optional method call: obj?.method(args)
                if self.check(TokenKind::LeftParen) {
                    self.advance();
                    let arguments = self.parse_arguments()?;
                    self.expect(TokenKind::RightParen)?;
                    expr = Expression::OptionalMethodCall(Box::new(OptionalMethodCallExpr {
                        object: expr,
                        method: member,
                        arguments,
                        span,
                    }));
                } else {
                    // Optional member access: obj?.member
                    expr = Expression::OptionalMember(Box::new(OptionalMemberExpr {
                        object: expr,
                        member,
                        span,
                    }));
                }
            }
            // Function call
            else if self.check(TokenKind::LeftParen) {
                let span = self.current_span();
                self.advance();
                let arguments = self.parse_arguments()?;
                self.expect(TokenKind::RightParen)?;
                expr = Expression::Call(Box::new(CallExpr {
                    callee: expr,
                    arguments,
                    span,
                }));
            }
            // Index access
            else if self.check(TokenKind::LeftBracket) {
                let span = self.current_span();
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenKind::RightBracket)?;
                expr = Expression::Index(Box::new(IndexExpr {
                    object: expr,
                    index,
                    span,
                }));
            }
            else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Argument>, ParseError> {
        let mut args = Vec::new();
        
        if !self.check(TokenKind::RightParen) {
            loop {
                // Check for labeled argument: label: value
                let (label, value) = if let Some(TokenKind::Identifier(name)) = self.peek_kind() {
                    let name = name.clone();
                    if self.peek_next_kind() == Some(&TokenKind::Colon) {
                        self.advance(); // consume identifier
                        self.advance(); // consume colon
                        (Some(name), self.parse_expression()?)
                    } else {
                        (None, self.parse_expression()?)
                    }
                } else {
                    (None, self.parse_expression()?)
                };
                
                args.push(Argument { label, value });
                
                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        Ok(args)
    }

    fn parse_primary_expr(&mut self) -> Result<Expression, ParseError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::IntLiteral(n)) => {
                self.advance();
                Ok(Expression::Literal(Literal::Int(n)))
            }
            Some(TokenKind::FloatLiteral(n)) => {
                self.advance();
                Ok(Expression::Literal(Literal::Float(n)))
            }
            Some(TokenKind::StringLiteral(s)) => {
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            Some(TokenKind::InterpolatedString(parts)) => {
                self.parse_interpolated_string(parts)
            }
            Some(TokenKind::True) => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true)))
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false)))
            }
            Some(TokenKind::Nil) => {
                self.advance();
                Ok(Expression::Literal(Literal::Nil))
            }
            // Handle 'self' keyword as identifier (for use in type invariants, methods, etc.)
            Some(TokenKind::SelfLower) => {
                self.advance();
                Ok(Expression::Identifier("self".to_string()))
            }
            // Handle 'Self' keyword as identifier (for type references)
            Some(TokenKind::SelfUpper) => {
                self.advance();
                Ok(Expression::Identifier("Self".to_string()))
            }
            // Handle 'config' keyword as identifier (for config.field access)
            Some(TokenKind::Config) => {
                self.advance();
                Ok(Expression::Identifier("config".to_string()))
            }
            Some(TokenKind::Identifier(name)) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Some(TokenKind::Handle(h)) => {
                self.advance();
                Ok(Expression::Handle(h))
            }
            Some(TokenKind::FacetAddress(prefix, handle)) => {
                self.advance();
                Ok(Expression::FacetAddress { prefix, handle })
            }
            Some(TokenKind::FacetPath(prefix, handle, path)) => {
                self.advance();
                Ok(Expression::FacetPath { prefix, handle, path })
            }
            Some(TokenKind::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(Expression::Grouped(Box::new(expr)))
            }
            Some(TokenKind::LeftBracket) => {
                self.advance();
                let mut elements = Vec::new();
                while !self.check(TokenKind::RightBracket) {
                    elements.push(self.parse_expression()?);
                    if !self.check(TokenKind::Comma) { break; }
                    self.advance();
                }
                self.expect(TokenKind::RightBracket)?;
                Ok(Expression::Array(elements))
            }
            // Object literal: { x: 10, y: 20 }
            Some(TokenKind::LeftBrace) => {
                self.parse_object_literal()
            }
            // Handle .enumVariant syntax (Swift-style shorthand)
            Some(TokenKind::Dot) => {
                self.advance();
                let variant = self.expect_identifier_or_keyword()?;
                Ok(Expression::Identifier(format!(".{}", variant)))
            }
            // Handle if expression: if cond { then } else { else }
            Some(TokenKind::If) => {
                self.parse_if_expression()
            }
            _ => Err(self.error("Expected expression")),
        }
    }
    
    /// Parse an if expression (as opposed to if statement)
    /// Used when if appears in expression context: let x = if cond { a } else { b }
    fn parse_if_expression(&mut self) -> Result<Expression, ParseError> {
        let span = self.current_span();
        self.expect(TokenKind::If)?;
        
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        
        // If expressions must have an else branch to be complete expressions
        self.expect(TokenKind::Else)?;
        
        let else_expr = if self.check(TokenKind::If) {
            // else if ... - recursively parse as another if expression
            self.parse_if_expression()?
        } else {
            // else { ... } - parse the final block
            let else_block = self.parse_block()?;
            // Convert block to expression
            Expression::Block(else_block)
        };
        
        // Convert then_branch block to expression
        let then_expr = Expression::Block(then_branch);
        
        Ok(Expression::Conditional(Box::new(ConditionalExpr {
            condition,
            then_expr,
            else_expr,
            span,
        })))
    }
    
    /// Parse object literal: { field1: value1, field2: value2, ... }
    fn parse_object_literal(&mut self) -> Result<Expression, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::LeftBrace)?;
        
        let mut fields = Vec::new();
        
        // Empty object: {}
        if self.check(TokenKind::RightBrace) {
            self.advance();
            return Ok(Expression::ObjectLiteral(Box::new(ObjectLiteralExpr {
                fields,
                type_hint: None,
                span: start,
            })));
        }
        
        // Parse fields
        loop {
            let field = self.parse_object_field()?;
            fields.push(field);
            
            // Check for comma or end
            if self.check(TokenKind::Comma) {
                self.advance();
                // Allow trailing comma: { x: 1, y: 2, }
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            } else {
                break;
            }
        }
        
        self.expect(TokenKind::RightBrace)?;
        
        // Optional type hint: { x: 10 } as Point
        let type_hint = if self.check(TokenKind::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        Ok(Expression::ObjectLiteral(Box::new(ObjectLiteralExpr {
            fields,
            type_hint,
            span: start,
        })))
    }
    
    /// Parse a single object field: name: value OR name (shorthand)
    fn parse_object_field(&mut self) -> Result<ObjectField, ParseError> {
        let span = self.current_span();
        let name = self.expect_identifier()?;
        
        // Check for value: name: expression
        let value = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            // Shorthand: { x } means { x: x }
            None
        };
        
        Ok(ObjectField { name, value, span })
    }
    
    /// Parse an interpolated string from lexer output
    fn parse_interpolated_string(
        &mut self,
        lexer_parts: Vec<StringPart>,
    ) -> Result<Expression, ParseError> {
        let span = self.current_span();
        self.advance(); // consume the InterpolatedString token
        
        let mut parsed_parts = Vec::new();
        
        for part in lexer_parts {
            match part {
                StringPart::Literal(s) => {
                    parsed_parts.push(InterpolatedPart::Literal(s));
                }
                StringPart::Interpolation(expr_str) => {
                    // Parse the expression string using a sub-lexer/parser
                    let expr = self.parse_embedded_expression(&expr_str)?;
                    parsed_parts.push(InterpolatedPart::Expression(expr));
                }
            }
        }
        
        Ok(Expression::InterpolatedString(Box::new(InterpolatedStringExpr {
            parts: parsed_parts,
            span,
        })))
    }
    
    /// Parse an expression from a string (used for interpolation)
    fn parse_embedded_expression(&mut self, source: &str) -> Result<Expression, ParseError> {
        // Tokenize the embedded expression
        let tokens = ulissy_lexer::tokenize(source)
            .map_err(|e| ParseError::new(
                &format!("Error in interpolation: {}", e.message),
                e.line,
                e.column,
            ))?;
        
        // Create a sub-parser for the expression
        let mut sub_parser = Parser::new(tokens);
        sub_parser.parse_expression()
    }
    
    /// Accept identifier OR keyword as identifier (for .public, .private, etc.)
    fn expect_identifier_or_keyword(&mut self) -> Result<String, ParseError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Identifier(name)) => {
                self.advance();
                Ok(name)
            }
            Some(TokenKind::Public) => {
                self.advance();
                Ok("public".to_string())
            }
            Some(TokenKind::Private) => {
                self.advance();
                Ok("private".to_string())
            }
            Some(TokenKind::Internal) => {
                self.advance();
                Ok("internal".to_string())
            }
            _ => Err(self.error("Expected identifier")),
        }
    }

    // ========================================================================
    // TYPE PARSING
    // ========================================================================

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        let base = self.parse_base_type()?;
        
        // Check for optional: Type?
        if self.check(TokenKind::Question) {
            self.advance();
            return Ok(TypeExpr::Optional(Box::new(base)));
        }
        
        Ok(base)
    }

    fn parse_base_type(&mut self) -> Result<TypeExpr, ParseError> {
        let name = self.expect_identifier()?;
        
        // Check for generic: Type<Param>
        if self.check(TokenKind::Less) {
            self.advance();
            let mut params = vec![self.parse_type_expr()?];
            while self.check(TokenKind::Comma) {
                self.advance();
                params.push(self.parse_type_expr()?);
            }
            self.expect(TokenKind::Greater)?;
            return Ok(TypeExpr::Generic { name, params });
        }
        
        Ok(TypeExpr::Simple(name))
    }

    // ========================================================================
    // BLOCK PARSING
    // ========================================================================

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.current_span();
        self.expect(TokenKind::LeftBrace)?;
        
        let mut statements = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::RightBrace)?;
        
        Ok(Block { statements, span: start })
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::EOF) | None)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    fn peek_next_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.current + 1).map(|t| &t.kind)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(&kind)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        if self.check(kind.clone()) {
            Ok(self.advance().unwrap())
        } else {
            Err(self.error(&format!("Expected {:?}", kind)))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek_kind().cloned() {
            Some(TokenKind::Identifier(name)) => {
                self.advance();
                Ok(name)
            }
            _ => Err(self.error("Expected identifier")),
        }
    }

    fn current_span(&self) -> Span {
        self.peek()
            .map(|t| Span::from_position(t.line, t.column))
            .unwrap_or_default()
    }

    fn error(&self, message: &str) -> ParseError {
        let (line, column) = self.peek()
            .map(|t| (t.line, t.column))
            .unwrap_or((0, 0));
        ParseError::new(message, line, column)
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Parse ULissy source code into an AST
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = ulissy_lexer::tokenize(source)
        .map_err(|e| ParseError::new(&e.message, e.line, e.column))?;
    Parser::new(tokens).parse()
}
