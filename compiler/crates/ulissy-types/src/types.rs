// ulissy-types/src/types.rs
// ULissy Type System - Type Definitions
// Version 0.1.0

use std::fmt;
use ulissy_parser::ast;

// ============================================================================
// ULISSY TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // === PRIMITIVES ===
    Int,
    Float,
    Bool,
    String,
    Nil,
    
    // === IDENTITY TYPES (GNS-specific) ===
    Identity,
    PublicKey,
    PrivateKey,
    Signature,
    Handle,
    
    // === SPATIAL TYPES (GNS-specific) ===
    H3Cell,
    Resolution,
    Distance,
    Coordinates,
    
    // === TEMPORAL TYPES ===
    Moment,
    Duration,
    
    // === CRYPTO TYPES ===
    Hash,
    SharedSecret,
    Ciphertext,
    
    // === ENERGY TYPES ===
    BatteryLevel,
    PowerMode,
    
    // === COMPOUND TYPES ===
    Array(Box<Type>),
    Optional(Box<Type>),
    Function { params: Vec<Type>, ret: Box<Type> },
    Tuple(Vec<Type>),
    
    // === GNS PROTOCOL TYPES ===
    Breadcrumb,
    Trajectory,
    Envelope(Box<Type>),
    GnsRecord,
    
    // === FACET TYPES ===
    FacetAddress,
    
    // === SPECIAL ===
    Unit,           // void/nothing
    Any,            // for flexibility during development
    Unknown,        // not yet inferred
    Error,          // type error placeholder
    
    // === USER-DEFINED ===
    Named(String),
    
    /// An enum type with its variants
    Enum {
        name: String,
        variants: Vec<EnumVariantType>,
    },
    
    /// Anonymous object/record type: { x: Int, y: Int }
    Object {
        fields: Vec<(String, Type)>,
    },
}

/// Represents a typed enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantType {
    /// Variant name (e.g., "gps", "ok")
    pub name: String,
    
    /// Associated types, if any
    /// None for simple variants
    /// Some([Type::Int]) for single-value variants
    pub associated_types: Option<Vec<Type>>,
}

impl Type {
    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::BatteryLevel)
    }
    
    /// Check if this type is a GNS identity type
    pub fn is_identity(&self) -> bool {
        matches!(self, Type::Identity | Type::PublicKey | Type::PrivateKey | Type::Handle)
    }
    
    /// Check if this type is spatial
    pub fn is_spatial(&self) -> bool {
        matches!(self, Type::H3Cell | Type::Coordinates | Type::Distance)
    }
    
    /// Check if types are compatible for assignment
    pub fn is_assignable_from(&self, other: &Type) -> bool {
        if self == other { return true; }
        if *self == Type::Any || *other == Type::Any { return true; }
        
        match (self, other) {
            // Int can be promoted to Float
            (Type::Float, Type::Int) => true,
            // Optional can accept non-optional
            (Type::Optional(inner), other) => inner.is_assignable_from(other),
            // Array covariance
            (Type::Array(a), Type::Array(b)) => a.is_assignable_from(b),
            _ => false,
        }
    }
    
    /// Get the result type of a binary operation
    pub fn binary_result(&self, op: &ast::BinaryOp, other: &Type) -> Option<Type> {
        use ast::BinaryOp::*;
        
        match op {
            // Arithmetic: requires numeric, returns numeric
            Add | Sub | Mul | Div | Mod => {
                if self.is_numeric() && other.is_numeric() {
                    if *self == Type::Float || *other == Type::Float {
                        Some(Type::Float)
                    } else {
                        Some(Type::Int)
                    }
                } else if *self == Type::String && *op == Add {
                    Some(Type::String) // String concatenation
                } else if *self == Type::Duration && *other == Type::Duration {
                    Some(Type::Duration)
                } else if *self == Type::Distance && *other == Type::Distance {
                    Some(Type::Distance)
                } else {
                    None
                }
            }
            
            // Comparison: requires compatible types, returns Bool
            Eq | NotEq => Some(Type::Bool),
            Lt | Gt | LtEq | GtEq => {
                if self.is_numeric() && other.is_numeric() {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
            
            // Logical: requires Bool, returns Bool
            And | Or => {
                if *self == Type::Bool && *other == Type::Bool {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
            
            // Range
            Range | RangeExclusive => Some(Type::Array(Box::new(self.clone()))),
            
            // Spatial
            Within | Near => {
                if self.is_spatial() {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
            Type::Nil => write!(f, "Nil"),
            Type::Identity => write!(f, "Identity"),
            Type::PublicKey => write!(f, "PublicKey"),
            Type::PrivateKey => write!(f, "PrivateKey"),
            Type::Signature => write!(f, "Signature"),
            Type::Handle => write!(f, "Handle"),
            Type::H3Cell => write!(f, "H3Cell"),
            Type::Resolution => write!(f, "Resolution"),
            Type::Distance => write!(f, "Distance"),
            Type::Coordinates => write!(f, "Coordinates"),
            Type::Moment => write!(f, "Moment"),
            Type::Duration => write!(f, "Duration"),
            Type::Hash => write!(f, "Hash"),
            Type::SharedSecret => write!(f, "SharedSecret"),
            Type::Ciphertext => write!(f, "Ciphertext"),
            Type::BatteryLevel => write!(f, "BatteryLevel"),
            Type::PowerMode => write!(f, "PowerMode"),
            Type::Array(inner) => write!(f, "[{}]", inner),
            Type::Optional(inner) => write!(f, "{}?", inner),
            Type::Function { params, ret } => {
                let params_str: Vec<_> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "({}) -> {}", params_str.join(", "), ret)
            }
            Type::Tuple(types) => {
                let types_str: Vec<_> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", types_str.join(", "))
            }
            Type::Breadcrumb => write!(f, "Breadcrumb"),
            Type::Trajectory => write!(f, "Trajectory"),
            Type::Envelope(inner) => write!(f, "Envelope<{}>", inner),
            Type::GnsRecord => write!(f, "GnsRecord"),
            Type::FacetAddress => write!(f, "FacetAddress"),
            Type::Unit => write!(f, "()"),
            Type::Any => write!(f, "Any"),
            Type::Unknown => write!(f, "?"),
            Type::Error => write!(f, "<error>"),
            Type::Named(name) => write!(f, "{}", name),
            Type::Enum { name, .. } => write!(f, "enum {}", name),
            Type::Object { fields } => {
                write!(f, "{{ ")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, " }}")
            }
        }
    }
}

// ============================================================================
// TYPED AST
// ============================================================================

/// A fully type-checked program
#[derive(Debug)]
pub struct TypedProgram {
    pub statements: Vec<TypedStatement>,
}

#[derive(Debug)]
pub struct TypedStatement {
    pub kind: TypedStatementKind,
    pub span: ast::Span,
}

#[derive(Debug)]
pub enum TypedStatementKind {
    IdentityDecl {
        name: String,
        init: TypedExpr,
    },
    LetDecl {
        name: String,
        declared_type: Option<Type>,
        inferred_type: Type,
        init: TypedExpr,
    },
    VarDecl {
        name: String,
        declared_type: Option<Type>,
        inferred_type: Type,
        init: Option<TypedExpr>,
    },
    EveryBlock {
        interval: TypedExpr,
        condition: Option<TypedExpr>,
        body: Vec<TypedStatement>,
    },
    WhenBlock {
        condition: TypedExpr,
        body: Vec<TypedStatement>,
    },
    SendStatement {
        recipient: TypedExpr,
        fields: Vec<(String, TypedExpr)>,
    },
    ExpressionStatement(TypedExpr),
    
    EnumDecl {
        name: String,
        type_params: Vec<String>,
        variants: Vec<TypedEnumVariant>,
    },
    
    ConfigBlock {
        fields: Vec<TypedConfigField>,
    },
    
    ComputedPropertyDecl {
        name: String,
        inferred_type: Type,
        body: TypedComputedBody,
    },
}

/// A typed enum variant
#[derive(Debug, Clone)]
pub struct TypedEnumVariant {
    pub name: String,
    pub associated_types: Option<Vec<Type>>,
}

/// A typed config field
#[derive(Debug, Clone)]
pub struct TypedConfigField {
    pub name: String,
    pub value: TypedExpr,
}

/// Body of a typed computed property
#[derive(Debug, Clone)]
pub enum TypedComputedBody {
    /// Single expression: computed x: Int = a + b
    Expression(TypedExpr),
    /// Object literal body: computed status: Status { field1: expr1, field2: expr2 }
    ObjectFields(Vec<TypedObjectField>),
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Type,
    pub span: ast::Span,
}

#[derive(Debug, Clone)]
pub enum TypedExprKind {
    Literal(ast::Literal),
    Identifier(String),
    Handle(String),
    FacetAddress { prefix: String, handle: String },
    Binary {
        left: Box<TypedExpr>,
        op: ast::BinaryOp,
        right: Box<TypedExpr>,
    },
    Unary {
        op: ast::UnaryOp,
        operand: Box<TypedExpr>,
    },
    Member {
        object: Box<TypedExpr>,
        member: String,
    },
    Call {
        callee: Box<TypedExpr>,
        args: Vec<TypedExpr>,
    },
    MethodCall {
        object: Box<TypedExpr>,
        method: String,
        args: Vec<TypedExpr>,
    },
    UnitValue {
        value: Box<TypedExpr>,
        unit: String,
    },
    Array(Vec<TypedExpr>),
    
    /// Optional member access: obj?.member
    OptionalMember {
        object: Box<TypedExpr>,
        member: String,
    },
    
    /// Optional method call: obj?.method(args)
    OptionalMethodCall {
        object: Box<TypedExpr>,
        method: String,
        args: Vec<TypedExpr>,
    },
    
    /// Nil coalescing: expr ?? default
    NilCoalescing {
        primary: Box<TypedExpr>,
        fallback: Box<TypedExpr>,
    },
    
    /// Object literal: { x: 10, y: 20 }
    ObjectLiteral {
        fields: Vec<TypedObjectField>,
        type_hint: Option<String>,
    },
    
    /// Interpolated string: "Hello, \(name)!"
    InterpolatedString {
        parts: Vec<TypedInterpolatedPart>,
    },
}

/// A typed object field
#[derive(Debug, Clone)]
pub struct TypedObjectField {
    /// Field name
    pub name: String,
    
    /// Typed field value
    pub value: TypedExpr,
    
    /// Inferred type of the field
    pub field_type: Type,
}

/// A part of a typed interpolated string
#[derive(Debug, Clone)]
pub enum TypedInterpolatedPart {
    /// Literal text
    Literal(String),
    
    /// Typed expression to interpolate
    Expression(TypedExpr),
}

// ============================================================================
// SYMBOL TABLE ENTRY
// ============================================================================

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub initialized: bool,
}

impl Symbol {
    pub fn new(name: &str, ty: Type, mutable: bool) -> Self {
        Symbol {
            name: name.to_string(),
            ty,
            mutable,
            initialized: true,
        }
    }
}
