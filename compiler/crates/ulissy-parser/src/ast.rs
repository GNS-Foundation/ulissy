// ulissy-parser/src/ast.rs
// ULissy Abstract Syntax Tree Definitions
// Version 0.1.0

use std::fmt;

// ============================================================================
// PROGRAM ROOT
// ============================================================================

/// The root of an ULissy program
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

// ============================================================================
// STATEMENTS
// ============================================================================

#[derive(Debug, Clone)]
pub enum Statement {
    /// identity me = Keychain.primary
    IdentityDecl(IdentityDecl),
    
    /// let x = expr
    LetDecl(LetDecl),
    
    /// var x = expr
    VarDecl(VarDecl),
    
    /// const X = expr
    ConstDecl(ConstDecl),
    
    /// fn name(params) -> ReturnType { body }
    FnDecl(FnDecl),
    
    /// type Name { fields }
    TypeDecl(TypeDecl),
    
    /// enum Name { variant1, variant2, ... }
    EnumDecl(EnumDecl),
    
    /// every 10.minutes { ... }
    EveryBlock(EveryBlock),
    
    /// when condition { ... }
    WhenBlock(WhenBlock),
    
    /// after 5.seconds { ... }
    AfterBlock(AfterBlock),
    
    /// send to @handle { ... }
    SendStatement(SendStatement),
    
    /// if condition { ... } else { ... }
    IfStatement(IfStatement),
    
    /// match expr { cases }
    MatchStatement(MatchStatement),
    
    /// return expr
    ReturnStatement(ReturnStatement),
    
    /// Expression as statement (e.g., function call)
    ExpressionStatement(Expression),
    
    /// import module.path
    ImportStatement(ImportStatement),
    
    /// config { field: value, ... }
    ConfigBlock(ConfigBlock),
    
    /// computed name: Type = expr  OR  computed name: Type { fields }
    ComputedPropertyDecl(ComputedPropertyDecl),
    
    /// for item in collection { body }
    ForStatement(ForStatement),
}

// ============================================================================
// DECLARATIONS
// ============================================================================

/// identity me = Keychain.primary
#[derive(Debug, Clone)]
pub struct IdentityDecl {
    pub name: String,
    pub initializer: Expression,
    pub span: Span,
}

/// let x = expr  OR  let x: Type = expr
#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub type_annotation: Option<TypeExpr>,
    pub initializer: Expression,
    pub span: Span,
}

/// var x = expr
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub type_annotation: Option<TypeExpr>,
    pub initializer: Option<Expression>,
    pub span: Span,
}

/// const X = expr
#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub type_annotation: Option<TypeExpr>,
    pub initializer: Expression,
    pub span: Span,
}

/// fn name(params) -> ReturnType { body }
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeExpr>,
    pub constraints: Vec<WhereClause>,
    pub body: Block,
    pub is_async: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_expr: TypeExpr,
    pub default_value: Option<Expression>,
}

/// type Name { fields }
#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub invariants: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub type_expr: TypeExpr,
    pub is_computed: bool,
    pub default_value: Option<Expression>,
}

/// Enum declaration: enum Name { variant1, variant2, ... }
///
/// Example:
/// ```ulissy
/// enum LocationSource {
///     gps,
///     wifi,
///     cell,
///     ip,
///     manual
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EnumDecl {
    /// The enum name (e.g., "LocationSource")
    pub name: String,
    
    /// Generic type parameters (e.g., ["T", "E"] for Result<T, E>)
    pub type_params: Vec<String>,
    
    /// The enum variants
    pub variants: Vec<EnumVariant>,
    
    /// Source location
    pub span: Span,
}

/// A single enum variant
///
/// Simple: `gps`
/// With value: `ok(T)` or `point(x: Int, y: Int)`
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// Variant name (e.g., "gps", "ok")
    pub name: String,
    
    /// Associated types, if any
    /// None = simple variant like `gps`
    /// Some([]) = empty tuple like `none()`
    /// Some([Type]) = single value like `ok(T)`
    pub associated_types: Option<Vec<TypeExpr>>,
    
    /// Named fields for record-style variants (future)
    /// e.g., `point(x: Int, y: Int)`
    pub named_fields: Option<Vec<(String, TypeExpr)>>,
}

impl fmt::Display for EnumDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enum {} {{ ", self.name)?;
        for (i, variant) in self.variants.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}", variant.name)?;
            if let Some(types) = &variant.associated_types {
                write!(f, "(")?;
                for (j, ty) in types.iter().enumerate() {
                    if j > 0 { write!(f, ", ")?; }
                    write!(f, "{:?}", ty)?;
                }
                write!(f, ")")?;
            }
        }
        write!(f, " }}")
    }
}

// ============================================================================
// TEMPORAL BLOCKS (ULissy-specific)
// ============================================================================

/// every 10.minutes when condition { ... }
#[derive(Debug, Clone)]
pub struct EveryBlock {
    pub interval: Expression,
    pub condition: Option<Expression>,
    pub body: Block,
    pub span: Span,
}

/// when condition { ... }
#[derive(Debug, Clone)]
pub struct WhenBlock {
    pub condition: Expression,
    pub body: Block,
    pub span: Span,
}

/// after 5.seconds { ... }
#[derive(Debug, Clone)]
pub struct AfterBlock {
    pub delay: Expression,
    pub body: Block,
    pub span: Span,
}

// ============================================================================
// MESSAGING (ULissy-specific)
// ============================================================================

/// send to @handle { message: "..." }
#[derive(Debug, Clone)]
pub struct SendStatement {
    pub recipient: Expression,
    pub fields: Vec<(String, Expression)>,
    pub span: Span,
}

// ============================================================================
// CONTROL FLOW
// ============================================================================

/// if condition { ... } else { ... }
#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Block,
    pub else_branch: Option<Box<ElseBranch>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ElseBranch {
    ElseIf(IfStatement),
    Else(Block),
}

/// match expr { case pattern: body, ... }
#[derive(Debug, Clone)]
pub struct MatchStatement {
    pub subject: Expression,
    pub cases: Vec<MatchCase>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),
    Literal(Literal),
    Wildcard,
    EnumVariant { name: String, bindings: Vec<String> },
    Tuple(Vec<Pattern>),
}

/// return expr
#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub span: Span,
}

/// import ulissy.spatial
#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub span: Span,
}

/// for item in collection { body }
#[derive(Debug, Clone)]
pub struct ForStatement {
    /// Loop variable name
    pub variable: String,
    /// Collection/iterable expression
    pub iterable: Expression,
    /// Loop body
    pub body: Block,
    /// Source location
    pub span: Span,
}

// ============================================================================
// CONFIG BLOCK (Module-level configuration)
// ============================================================================

/// Configuration block - module-level settings
/// 
/// Example:
/// ```ulissy
/// config {
///     resolution: 7,
///     interval: 10.minutes,
///     minBreadcrumbsPerEpoch: 100
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConfigBlock {
    pub fields: Vec<ConfigField>,
    pub span: Span,
}

/// A single field in a config block
#[derive(Debug, Clone)]
pub struct ConfigField {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

// ============================================================================
// COMPUTED PROPERTY DECLARATION (Standalone reactive values)
// ============================================================================

/// Standalone computed property - module-level reactive value
/// 
/// Example:
/// ```ulissy
/// computed status: CollectionStatus {
///     isActive: collection.running,
///     totalCount: me.trajectory.count
/// }
/// 
/// computed total: Int = items.count
/// ```
#[derive(Debug, Clone)]
pub struct ComputedPropertyDecl {
    pub name: String,
    pub type_annotation: TypeExpr,
    pub body: ComputedBody,
    pub span: Span,
}

/// Body of a computed property declaration
#[derive(Debug, Clone)]
pub enum ComputedBody {
    /// Single expression: computed x: Int = a + b
    Expression(Expression),
    /// Object literal body: computed status: Status { field1: expr1, field2: expr2 }
    ObjectFields(Vec<ObjectField>),
}

// ============================================================================
// EXPRESSIONS
// ============================================================================

#[derive(Debug, Clone)]
pub enum Expression {
    /// Literal values: 42, 3.14, "hello", true
    Literal(Literal),
    
    /// Variable reference: x, me, Keychain
    Identifier(String),
    
    /// @handle
    Handle(String),
    
    /// dix@alice, pay@merchant
    FacetAddress { prefix: String, handle: String },
    
    /// home@alice/lights
    FacetPath { prefix: String, handle: String, path: String },
    
    /// Binary operation: a + b, x == y
    Binary(Box<BinaryExpr>),
    
    /// Unary operation: !x, -y
    Unary(Box<UnaryExpr>),
    
    /// Member access: obj.field
    Member(Box<MemberExpr>),
    
    /// Function call: foo(a, b)
    Call(Box<CallExpr>),
    
    /// Method call: obj.method(a, b)
    MethodCall(Box<MethodCallExpr>),
    
    /// Index access: arr[0]
    Index(Box<IndexExpr>),
    
    /// Array literal: [1, 2, 3]
    Array(Vec<Expression>),
    
    /// Dictionary literal: ["a": 1, "b": 2]
    Dictionary(Vec<(Expression, Expression)>),
    
    /// Lambda: { |x| x + 1 }
    Lambda(Box<LambdaExpr>),
    
    /// Conditional: if cond { a } else { b }
    Conditional(Box<ConditionalExpr>),
    
    /// Optional member access: obj?.member
    /// Returns None if obj is None, otherwise returns obj.member
    OptionalMember(Box<OptionalMemberExpr>),
    
    /// Optional method call: obj?.method(args)
    /// Returns None if obj is None, otherwise calls method
    OptionalMethodCall(Box<OptionalMethodCallExpr>),
    
    /// Nil coalescing: expr ?? default
    /// Returns expr if not nil, otherwise returns default
    NilCoalescing(Box<NilCoalescingExpr>),
    
    /// Breadcrumb constructor (ULissy-specific)
    Breadcrumb(Box<BreadcrumbExpr>),
    
    /// Unit value with suffix: 10.minutes, 500.meters
    UnitValue(Box<UnitValueExpr>),
    
    /// Block expression
    Block(Block),
    
    /// Grouped expression: (expr)
    Grouped(Box<Expression>),
    
    /// Object literal: { field1: value1, field2: value2 }
    ObjectLiteral(Box<ObjectLiteralExpr>),
    
    /// Interpolated string: "Hello, \(name)!"
    InterpolatedString(Box<InterpolatedStringExpr>),
    
    /// Assignment expression: x = expr
    Assignment(Box<AssignmentExpr>),
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Expression,
    pub operator: BinaryOp,
    pub right: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Comparison
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    // Logical
    And, Or,
    // Range
    Range, RangeExclusive,
    // Spatial (ULissy-specific)
    Within, Near,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::NotEq => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::LtEq => write!(f, "<="),
            BinaryOp::GtEq => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
            BinaryOp::Range => write!(f, ".."),
            BinaryOp::RangeExclusive => write!(f, "..<"),
            BinaryOp::Within => write!(f, "within"),
            BinaryOp::Near => write!(f, "near"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,    // -x
    Not,    // !x
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub object: Expression,
    pub member: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Expression,
    pub arguments: Vec<Argument>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub label: Option<String>,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Expression,
    pub method: String,
    pub arguments: Vec<Argument>,
    pub span: Span,
}

/// Optional member access expression: obj?.member
/// 
/// Example: `me.trajectory.last?.hash`
/// 
/// Semantics: If the object is None/nil, the entire expression evaluates
/// to None. Otherwise, access the member as normal.
#[derive(Debug, Clone)]
pub struct OptionalMemberExpr {
    /// The object being accessed (may be optional)
    pub object: Expression,
    /// The member name to access
    pub member: String,
    /// Source location
    pub span: Span,
}

/// Optional method call expression: obj?.method(args)
/// 
/// Example: `me.trajectory.last?.signed(me)`
/// 
/// Semantics: If the object is None/nil, the entire expression evaluates
/// to None without calling the method. Otherwise, call the method.
#[derive(Debug, Clone)]
pub struct OptionalMethodCallExpr {
    /// The object on which to call the method (may be optional)
    pub object: Expression,
    /// The method name
    pub method: String,
    /// The arguments to pass
    pub arguments: Vec<Argument>,
    /// Source location
    pub span: Span,
}

/// Nil coalescing expression: expr ?? default
/// 
/// Example: `prevHash ?? "genesis"`
/// 
/// Semantics: If expr is None/nil, return default. Otherwise return expr.
/// This is equivalent to Rust's `.unwrap_or()` or Swift's `??`.
#[derive(Debug, Clone)]
pub struct NilCoalescingExpr {
    /// The primary expression (may be None)
    pub primary: Expression,
    /// The fallback value if primary is None
    pub fallback: Expression,
    /// Source location
    pub span: Span,
}

/// Object literal expression: { field1: value1, field2: value2, ... }
///
/// Example: `{ x: 10, y: 20, name: "point" }`
#[derive(Debug, Clone)]
pub struct ObjectLiteralExpr {
    /// The fields of the object
    pub fields: Vec<ObjectField>,
    
    /// Optional type hint: { x: 10, y: 20 } as Point
    pub type_hint: Option<String>,
    
    /// Source location
    pub span: Span,
}

/// A single field in an object literal
#[derive(Debug, Clone)]
pub struct ObjectField {
    /// Field name
    pub name: String,
    
    /// Field value
    /// None for shorthand: { x } means { x: x }
    pub value: Option<Expression>,
    
    /// Source location
    pub span: Span,
}

impl ObjectLiteralExpr {
    /// Create a new object literal
    pub fn new(fields: Vec<ObjectField>, span: Span) -> Self {
        ObjectLiteralExpr {
            fields,
            type_hint: None,
            span,
        }
    }
    
    /// Check if object has a field
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f.name == name)
    }
}

impl ObjectField {
    /// Create a field with explicit value
    pub fn new(name: String, value: Expression, span: Span) -> Self {
        ObjectField {
            name,
            value: Some(value),
            span,
        }
    }
    
    /// Create a shorthand field: { x } meaning { x: x }
    pub fn shorthand(name: String, span: Span) -> Self {
        ObjectField {
            name,
            value: None,
            span,
        }
    }
}

impl std::fmt::Display for ObjectLiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}", field.name)?;
            if let Some(value) = &field.value {
                write!(f, ": {:?}", value)?;
            }
        }
        write!(f, " }}")?;
        if let Some(hint) = &self.type_hint {
            write!(f, " as {}", hint)?;
        }
        Ok(())
    }
}

/// Interpolated string expression
///
/// Example: "Hello, \(name)! You have \(count) messages."
#[derive(Debug, Clone)]
pub struct InterpolatedStringExpr {
    /// The parts of the string (literals and expressions)
    pub parts: Vec<InterpolatedPart>,
    
    /// Source location
    pub span: Span,
}

/// A part of an interpolated string
#[derive(Debug, Clone)]
pub enum InterpolatedPart {
    /// Literal text: "Hello, "
    Literal(String),
    
    /// Expression to interpolate: \(name)
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub object: Expression,
    pub index: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub params: Vec<String>,
    pub body: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConditionalExpr {
    pub condition: Expression,
    pub then_expr: Expression,
    pub else_expr: Expression,
    pub span: Span,
}

/// breadcrumb(cell: here.h3(10), context: sensors.digest, previous: hash)
#[derive(Debug, Clone)]
pub struct BreadcrumbExpr {
    pub cell: Expression,
    pub context: Expression,
    pub previous: Expression,
    pub span: Span,
}

/// 10.minutes, 500.meters, 80.percent
#[derive(Debug, Clone)]
pub struct UnitValueExpr {
    pub value: Expression,
    pub unit: String,
    pub span: Span,
}

/// Assignment expression: target = value
#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    /// The target of the assignment (identifier or member access)
    pub target: Expression,
    /// The value being assigned
    pub value: Expression,
    /// Source location
    pub span: Span,
}

// ============================================================================
// LITERALS
// ============================================================================

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// Simple type: Int, String, Handle
    Simple(String),
    
    /// Generic type: Array<Int>, Envelope<Message>
    Generic { name: String, params: Vec<TypeExpr> },
    
    /// Optional type: Handle?
    Optional(Box<TypeExpr>),
    
    /// Function type: (Int, Int) -> Int
    Function { params: Vec<TypeExpr>, return_type: Box<TypeExpr> },
    
    /// Tuple type: (Int, String)
    Tuple(Vec<TypeExpr>),
    
    /// Union type: String | Int
    Union(Vec<TypeExpr>),
}

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub expression: Expression,
}

// ============================================================================
// BLOCK
// ============================================================================

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

// ============================================================================
// SOURCE LOCATION
// ============================================================================

#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Span {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Span { start_line, start_column, end_line, end_column }
    }
    
    pub fn from_position(line: usize, column: usize) -> Self {
        Span { start_line: line, start_column: column, end_line: line, end_column: column }
    }
    
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start_line: self.start_line,
            start_column: self.start_column,
            end_line: other.end_line,
            end_column: other.end_column,
        }
    }
}

// ============================================================================
// PRETTY PRINTING
// ============================================================================

impl Program {
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();
        output.push_str("Program\n");
        for (i, stmt) in self.statements.iter().enumerate() {
            let prefix = if i == self.statements.len() - 1 { "└── " } else { "├── " };
            let child_prefix = if i == self.statements.len() - 1 { "    " } else { "│   " };
            output.push_str(&stmt.pretty_print(prefix, child_prefix));
        }
        output
    }
}

impl Statement {
    fn pretty_print(&self, prefix: &str, child_prefix: &str) -> String {
        match self {
            Statement::IdentityDecl(decl) => {
                format!("{}IdentityDecl: {}\n{}└── init: {:?}\n", 
                    prefix, decl.name, child_prefix, decl.initializer)
            }
            Statement::LetDecl(decl) => {
                format!("{}LetDecl: {}\n{}└── init: {:?}\n",
                    prefix, decl.name, child_prefix, decl.initializer)
            }
            Statement::VarDecl(decl) => {
                format!("{}VarDecl: {}\n", prefix, decl.name)
            }
            Statement::EveryBlock(block) => {
                format!("{}EveryBlock\n{}├── interval: {:?}\n{}├── condition: {:?}\n{}└── body: {} statements\n",
                    prefix, child_prefix, block.interval, child_prefix, block.condition, child_prefix, block.body.statements.len())
            }
            Statement::WhenBlock(block) => {
                format!("{}WhenBlock\n{}├── condition: {:?}\n{}└── body: {} statements\n",
                    prefix, child_prefix, block.condition, child_prefix, block.body.statements.len())
            }
            Statement::SendStatement(send) => {
                format!("{}SendStatement\n{}├── to: {:?}\n{}└── fields: {} entries\n",
                    prefix, child_prefix, send.recipient, child_prefix, send.fields.len())
            }
            Statement::ExpressionStatement(expr) => {
                format!("{}ExpressionStmt: {:?}\n", prefix, expr)
            }
            _ => format!("{}{:?}\n", prefix, self),
        }
    }
}
