// ulissy-types/src/context.rs
// ULissy Type Checker - Symbol Table and Context
// Version 0.1.0

use std::collections::HashMap;
use crate::types::{Type, Symbol};

// ============================================================================
// TYPE CONTEXT (Symbol Table + Scopes)
// ============================================================================

pub struct TypeContext {
    scopes: Vec<Scope>,
    /// Built-in types and their definitions
    type_definitions: HashMap<String, Type>,
}

struct Scope {
    symbols: HashMap<String, Symbol>,
}

impl TypeContext {
    pub fn new() -> Self {
        let mut ctx = TypeContext {
            scopes: vec![Scope { symbols: HashMap::new() }],
            type_definitions: HashMap::new(),
        };
        ctx.register_builtins();
        ctx
    }
    
    /// Register all built-in symbols and types
    fn register_builtins(&mut self) {
        // Built-in objects
        self.define("Keychain", Type::Named("Keychain".to_string()), false);
        self.define("here", Type::Coordinates, false);
        self.define("now", Type::Moment, false);
        self.define("battery", Type::BatteryLevel, false);
        self.define("sensors", Type::Named("Sensors".to_string()), false);
        
        // Built-in functions
        self.define("print", Type::Function { 
            params: vec![Type::Any], 
            ret: Box::new(Type::Unit) 
        }, false);
        
        self.define("breadcrumb", Type::Function {
            params: vec![Type::H3Cell, Type::Hash, Type::Hash],
            ret: Box::new(Type::Breadcrumb),
        }, false);
        
        self.define("distance", Type::Function {
            params: vec![Type::Coordinates, Type::Coordinates],
            ret: Box::new(Type::Distance),
        }, false);
        
        // Type definitions
        self.type_definitions.insert("Int".to_string(), Type::Int);
        self.type_definitions.insert("Float".to_string(), Type::Float);
        self.type_definitions.insert("Bool".to_string(), Type::Bool);
        self.type_definitions.insert("String".to_string(), Type::String);
        self.type_definitions.insert("Identity".to_string(), Type::Identity);
        self.type_definitions.insert("Handle".to_string(), Type::Handle);
        self.type_definitions.insert("PublicKey".to_string(), Type::PublicKey);
        self.type_definitions.insert("H3Cell".to_string(), Type::H3Cell);
        self.type_definitions.insert("Distance".to_string(), Type::Distance);
        self.type_definitions.insert("Duration".to_string(), Type::Duration);
        self.type_definitions.insert("Moment".to_string(), Type::Moment);
        self.type_definitions.insert("Hash".to_string(), Type::Hash);
        self.type_definitions.insert("Breadcrumb".to_string(), Type::Breadcrumb);
        self.type_definitions.insert("Trajectory".to_string(), Type::Trajectory);
        self.type_definitions.insert("Signature".to_string(), Type::Signature);
        self.type_definitions.insert("BatteryLevel".to_string(), Type::BatteryLevel);
    }
    
    /// Enter a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope { symbols: HashMap::new() });
    }
    
    /// Exit the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    
    /// Define a new symbol in the current scope
    pub fn define(&mut self, name: &str, ty: Type, mutable: bool) {
        let symbol = Symbol::new(name, ty, mutable);
        if let Some(scope) = self.scopes.last_mut() {
            scope.symbols.insert(name.to_string(), symbol);
        }
    }
    
    /// Look up a symbol by name (searches all scopes)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
        }
        None
    }
    
    /// Check if a symbol exists in the current scope only
    pub fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last()
            .map(|s| s.symbols.contains_key(name))
            .unwrap_or(false)
    }
    
    /// Resolve a type name to a Type
    pub fn resolve_type(&self, name: &str) -> Option<Type> {
        self.type_definitions.get(name).cloned()
    }
    
    /// Get the type of a member access (e.g., identity.trajectory)
    pub fn get_member_type(&self, base_type: &Type, member: &str) -> Option<Type> {
        match (base_type, member) {
            // Keychain members
            (Type::Named(n), "primary") if n == "Keychain" => Some(Type::Identity),
            (Type::Named(n), "facet") if n == "Keychain" => Some(Type::Function {
                params: vec![Type::String],
                ret: Box::new(Type::Identity),
            }),
            
            // Identity members
            (Type::Identity, "publicKey") => Some(Type::PublicKey),
            (Type::Identity, "handle") => Some(Type::Optional(Box::new(Type::Handle))),
            (Type::Identity, "trajectory") => Some(Type::Trajectory),
            (Type::Identity, "trustScore") => Some(Type::Float),
            
            // Trajectory members
            (Type::Trajectory, "count") => Some(Type::Int),
            (Type::Trajectory, "last") => Some(Type::Hash),
            (Type::Trajectory, "append") => Some(Type::Function {
                params: vec![Type::Breadcrumb],
                ret: Box::new(Type::Unit),
            }),
            
            // Breadcrumb members
            (Type::Breadcrumb, "cell") => Some(Type::H3Cell),
            (Type::Breadcrumb, "timestamp") => Some(Type::Moment),
            (Type::Breadcrumb, "signature") => Some(Type::Signature),
            (Type::Breadcrumb, "signed") => Some(Type::Function {
                params: vec![Type::Identity],
                ret: Box::new(Type::Breadcrumb),
            }),
            
            // Coordinates members
            (Type::Coordinates, "h3") => Some(Type::Function {
                params: vec![Type::Int],
                ret: Box::new(Type::H3Cell),
            }),
            
            // Sensors members
            (Type::Named(n), "digest") if n == "Sensors" => Some(Type::Hash),
            
            // Distance units
            (Type::Int, "meters") | (Type::Float, "meters") => Some(Type::Distance),
            (Type::Int, "kilometers") | (Type::Float, "kilometers") => Some(Type::Distance),
            
            // Duration units
            (Type::Int, "seconds") | (Type::Float, "seconds") => Some(Type::Duration),
            (Type::Int, "minutes") | (Type::Float, "minutes") => Some(Type::Duration),
            (Type::Int, "hours") | (Type::Float, "hours") => Some(Type::Duration),
            (Type::Int, "days") | (Type::Float, "days") => Some(Type::Duration),
            
            // Percentage
            (Type::Int, "percent") | (Type::Float, "percent") => Some(Type::BatteryLevel),
            
            // FacetAddress methods
            (Type::FacetAddress, "post") => Some(Type::Function {
                params: vec![Type::String],
                ret: Box::new(Type::Unit),
            }),
            (Type::FacetAddress, "send") => Some(Type::Function {
                params: vec![Type::Any],
                ret: Box::new(Type::Unit),
            }),
            (Type::FacetAddress, "request") => Some(Type::Function {
                params: vec![Type::Any],
                ret: Box::new(Type::Unit),
            }),
            (Type::FacetAddress, "set") => Some(Type::Function {
                params: vec![Type::Any],
                ret: Box::new(Type::Unit),
            }),
            
            _ => None,
        }
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
