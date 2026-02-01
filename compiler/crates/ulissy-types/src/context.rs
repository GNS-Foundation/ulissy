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
        // Register built-in types mappings
        self.register_type("Uint64", Type::Int);
        self.register_type("Uint32", Type::Int);
        self.register_type("Int8", Type::Int);
        self.register_type("Int", Type::Int); // Explicit alias
        self.register_type("Float", Type::Float);
        self.register_type("Bool", Type::Bool);
        self.register_type("String", Type::String);
        
        self.register_type("Bytes", Type::Named("Bytes".to_string()));
        self.register_type("Hash", Type::Hash);
        self.register_type("Signature", Type::Signature);
        self.register_type("PublicKey", Type::PublicKey);
        
        self.register_type("Moment", Type::Moment);
        self.register_type("Duration", Type::Duration);
        self.register_type("Distance", Type::Distance);
        self.register_type("H3Cell", Type::H3Cell);

        // Built-in objects
        self.define("Keychain", Type::Named("Keychain".to_string()), false);
        self.define("here", Type::Coordinates, false);
        self.define("now", Type::Moment, false);
        self.define("battery", Type::BatteryLevel, false);
        self.define("sensors", Type::Named("Sensors".to_string()), false);
        
        // Config object (accessible as 'config' in expressions)
        self.define("config", Type::Named("Config".to_string()), false);
        
        // Built-in print function
        self.define("print", Type::Function { 
            params: vec![Type::Any], 
            ret: Box::new(Type::Unit) 
        }, false);
        
        // Breadcrumb constructor
        self.define("breadcrumb", Type::Function {
            params: vec![Type::H3Cell, Type::Hash, Type::Hash],
            ret: Box::new(Type::Breadcrumb),
        }, false);
        
        // Distance function
        self.define("distance", Type::Function {
            params: vec![Type::Coordinates, Type::Coordinates],
            ret: Box::new(Type::Distance),
        }, false);
        
        // Math built-ins
        self.define("log2", Type::Function {
            params: vec![Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        self.define("exp", Type::Function {
            params: vec![Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        self.define("tanh", Type::Function {
            params: vec![Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        self.define("abs", Type::Function {
            params: vec![Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        self.define("sqrt", Type::Function {
            params: vec![Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        self.define("pow", Type::Function {
            params: vec![Type::Float, Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        self.define("min", Type::Function {
            params: vec![Type::Float, Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        self.define("max", Type::Function {
            params: vec![Type::Float, Type::Float],
            ret: Box::new(Type::Float),
        }, false);
        
        // Crypto built-ins
        self.define("sha256", Type::Function {
            params: vec![Type::Any],
            ret: Box::new(Type::Hash),
        }, false);
        
        // Bytes type constructor
        self.define("Bytes", Type::Named("Bytes".to_string()), false);
        
        // Stellar integration (stub)
        self.define("stellar", Type::Named("Stellar".to_string()), false);
        
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
        self.type_definitions.insert("Bytes".to_string(), Type::Named("Bytes".to_string()));
        self.type_definitions.insert("Config".to_string(), Type::Named("Config".to_string()));
        self.type_definitions.insert("Stellar".to_string(), Type::Named("Stellar".to_string()));
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
    
    /// Register a user-defined type (enum, struct, etc.)
    pub fn register_type(&mut self, name: &str, ty: Type) {
        self.type_definitions.insert(name.to_string(), ty);
    }
    
    /// Look up a user-defined type
    pub fn lookup_type(&self, name: &str) -> Option<Type> {
        self.type_definitions.get(name).cloned()
    }
    
    /// Look up a member of a user-defined type
    pub fn lookup_type_member(&self, type_name: &str, member: &str) -> Option<Type> {
        // For enums, check if member is a variant
        if let Some(Type::Enum { variants, .. }) = self.type_definitions.get(type_name) {
            for variant in variants {
                if variant.name == member {
                    // Return a function type for the constructor
                    let param_types = variant.associated_types.clone().unwrap_or_default();
                    if param_types.is_empty() {
                        return Some(Type::Named(type_name.to_string()));
                    } else {
                        return Some(Type::Function {
                            params: param_types,
                            ret: Box::new(Type::Named(type_name.to_string())),
                        });
                    }
                }
            }
        }
        None
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
            (Type::Breadcrumb, "previous") => Some(Type::Hash),
            (Type::Breadcrumb, "previousHash") => Some(Type::Hash), // Alias for previous
            (Type::Breadcrumb, "index") => Some(Type::H3Cell), // Maybe alias for cell? Or actual H3 index?
            (Type::Breadcrumb, "hash") => Some(Type::Hash),
            (Type::Breadcrumb, "signed") => Some(Type::Function {
                params: vec![Type::Identity],
                ret: Box::new(Type::Breadcrumb),
            }),
            
            // String members
            (Type::String, "length") => Some(Type::Int),
            
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
            
            // Config members - all config fields are accessible
            (Type::Named(n), _) if n == "Config" => Some(Type::Any),
            
            // PublicKey members
            (Type::PublicKey, "length") => Some(Type::Int),
            (Type::PublicKey, "data") => Some(Type::Named("Bytes".to_string())),
            
            // Identity additional members
            (Type::Identity, "stellarAddress") => Some(Type::String),
            
            // Stellar members
            (Type::Named(n), "length") if n == "Stellar" => Some(Type::Int),
            (Type::Named(n), _) if n == "Stellar" => Some(Type::Any),
            
            // Bytes members
            (Type::Named(n), "length") if n == "Bytes" => Some(Type::Int),
            
            (Type::Named(n), "data") if n == "Bytes" => Some(Type::Named("Bytes".to_string())),
            
            // Hash members
            (Type::Named(n), "data") if n == "Hash" => Some(Type::Named("Bytes".to_string())),
            (Type::Named(n), "length") if n == "Hash" => Some(Type::Int),
            (Type::Named(n), "toHex") if n == "Hash" => Some(Type::String),
            (Type::Hash, "data") => Some(Type::Named("Bytes".to_string())),
            (Type::Hash, "length") => Some(Type::Int),
            (Type::Hash, "toHex") => Some(Type::String),
            (Type::Hash, "slice") => Some(Type::Named("Bytes".to_string())),

            // String members (length is already handled above)
            (Type::String, "replace") => Some(Type::String), // returns String
            (Type::String, "isDigit") => Some(Type::Bool),
            (Type::String, "isLowercase") => Some(Type::Bool),
            (Type::String, "charAt") => Some(Type::String), // returns partial string or char? Treated as String for now.
            (Type::String, "slice") => Some(Type::String),

            // Generic Object field access (Fallback)
            (Type::Object { fields }, m) => {
                for (name, ty) in fields {
                    if name == m {
                        return Some(ty.clone());
                    }
                }
                None
            },
            
            // Named types - lookup definition (Fallback)
            (Type::Named(n), m) => {
                if let Some(resolved) = self.resolve_type(n) {
                    if let Type::Named(res_name) = &resolved {
                        if res_name == n { return None; }
                    }
                    self.get_member_type(&resolved, m)
                } else {
                    None
                }
            },
            
            // Any type - allow all member access
            (Type::Any, _) => Some(Type::Any),
            
            _ => None,
        }
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
