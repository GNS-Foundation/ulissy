// gns-runtime/src/handle.rs
// GNS Handle and Facet Address Types
// Version 0.1.0

use std::fmt;

// ============================================================================
// HANDLE
// ============================================================================

/// A GNS Handle represents a human-readable address in the network
/// Format: @username (e.g., @alice, @bob)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle {
    /// The handle username (without @)
    username: String,
}

impl Handle {
    /// Create a new handle from a username
    pub fn new(username: &str) -> Self {
        // Strip @ if present
        let clean = username.trim_start_matches('@');
        Handle {
            username: clean.to_string(),
        }
    }
    
    /// Parse a handle from a string (e.g., "@alice")
    pub fn from_str(s: &str) -> Result<Self, crate::RuntimeError> {
        let clean = s.trim_start_matches('@');
        if clean.is_empty() {
            return Err(crate::RuntimeError::new("Invalid handle: empty username"));
        }
        if clean.len() > 32 {
            return Err(crate::RuntimeError::new("Invalid handle: too long"));
        }
        Ok(Handle::new(clean))
    }
    
    /// Get the username
    pub fn username(&self) -> &str {
        &self.username
    }
    
    /// Get the full handle string (with @)
    pub fn to_handle_string(&self) -> String {
        format!("@{}", self.username)
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.username)
    }
}

// ============================================================================
// FACET ADDRESS
// ============================================================================

/// A Facet Address represents a specific service/facet on a handle
/// Format: prefix@handle (e.g., pay@alice, dix@bob)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FacetAddress {
    /// The facet prefix (e.g., "pay", "dix", "home")
    prefix: String,
    /// The target handle
    handle: Handle,
    /// Optional path (e.g., "/lights" in home@alice/lights)
    path: Option<String>,
}

impl FacetAddress {
    /// Create a new facet address
    pub fn new(prefix: &str, handle: &str) -> Self {
        FacetAddress {
            prefix: prefix.to_string(),
            handle: Handle::new(handle),
            path: None,
        }
    }
    
    /// Create a facet address with a path
    pub fn with_path(prefix: &str, handle: &str, path: &str) -> Self {
        FacetAddress {
            prefix: prefix.to_string(),
            handle: Handle::new(handle),
            path: Some(path.to_string()),
        }
    }
    
    /// Get the facet prefix
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    
    /// Get the handle
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
    
    /// Get the optional path
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

impl fmt::Display for FacetAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}@{}/{}", self.prefix, self.handle.username(), path)
        } else {
            write!(f, "{}@{}", self.prefix, self.handle.username())
        }
    }
}

// ============================================================================
// VISIBILITY
// ============================================================================

/// Visibility level for messages and data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Public - visible to anyone
    Public,
    /// Private - only visible to the recipient
    Private,
    /// Contacts - visible to contacts only
    Contacts,
    /// Group - visible to a specific group
    Group,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Private
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Private => write!(f, "private"),
            Visibility::Contacts => write!(f, "contacts"),
            Visibility::Group => write!(f, "group"),
        }
    }
}
