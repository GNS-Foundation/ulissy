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
    
    /// Optional chaining: x?.y
    OptionalChain(Box<MemberExpr>),
    
    /// Nil coalescing: x ?? default
    NilCoalesce(Box<BinaryExpr>),
    
    /// Breadcrumb constructor (ULissy-specific)
    Breadcrumb(Box<BreadcrumbExpr>),
    
    /// Unit value with suffix: 10.minutes, 500.meters
    UnitValue(Box<UnitValueExpr>),
    
    /// Block expression
    Block(Block),
    
    /// Grouped expression: (expr)
    Grouped(Box<Expression>),
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
