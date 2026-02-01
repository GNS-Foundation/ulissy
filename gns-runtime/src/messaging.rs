// gns-runtime/src/messaging.rs
// GNS Messaging Types
// Version 0.1.0

use std::collections::HashMap;
use crate::handle::{Handle, FacetAddress, Visibility};
use crate::identity::{Identity, PublicKey};

// ============================================================================
// MESSAGE
// ============================================================================

/// A Message is a structured payload that can be sent between identities
#[derive(Debug, Clone, Default)]
pub struct Message {
    /// Message fields as key-value pairs
    fields: HashMap<String, MessageValue>,
}

/// A value that can be stored in a message
#[derive(Debug, Clone)]
pub enum MessageValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Null,
}

impl Message {
    /// Create a new empty message
    pub fn new() -> Self {
        Message {
            fields: HashMap::new(),
        }
    }
    
    /// Set a field value
    pub fn set(&mut self, key: &str, value: impl Into<MessageValue>) {
        self.fields.insert(key.to_string(), value.into());
    }
    
    /// Get a field value
    pub fn get(&self, key: &str) -> Option<&MessageValue> {
        self.fields.get(key)
    }
    
    /// Check if message has a field
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }
    
    /// Remove a field
    pub fn remove(&mut self, key: &str) -> Option<MessageValue> {
        self.fields.remove(key)
    }
    
    /// Get all field names
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }
    
    /// Serialize message to JSON
    pub fn to_json(&self) -> String {
        // Simple JSON serialization
        let mut parts = Vec::new();
        for (k, v) in &self.fields {
            let value_str = match v {
                MessageValue::String(s) => format!("\"{}\"", s),
                MessageValue::Int(n) => n.to_string(),
                MessageValue::Float(f) => f.to_string(),
                MessageValue::Bool(b) => b.to_string(),
                MessageValue::Bytes(b) => format!("\"{}\"", hex::encode(b)),
                MessageValue::Null => "null".to_string(),
            };
            parts.push(format!("\"{}\": {}", k, value_str));
        }
        format!("{{{}}}", parts.join(", "))
    }
}

impl From<String> for MessageValue {
    fn from(s: String) -> Self {
        MessageValue::String(s)
    }
}

impl From<&str> for MessageValue {
    fn from(s: &str) -> Self {
        MessageValue::String(s.to_string())
    }
}

impl From<i64> for MessageValue {
    fn from(n: i64) -> Self {
        MessageValue::Int(n)
    }
}

impl From<f64> for MessageValue {
    fn from(f: f64) -> Self {
        MessageValue::Float(f)
    }
}

impl From<bool> for MessageValue {
    fn from(b: bool) -> Self {
        MessageValue::Bool(b)
    }
}

impl From<Vec<u8>> for MessageValue {
    fn from(b: Vec<u8>) -> Self {
        MessageValue::Bytes(b)
    }
}

// ============================================================================
// ENVELOPE
// ============================================================================

/// An Envelope is an encrypted message wrapper
#[derive(Debug, Clone)]
pub struct Envelope {
    /// The encrypted payload
    pub ciphertext: Vec<u8>,
    /// The sender's ephemeral public key
    pub ephemeral_key: PublicKey,
    /// The recipient's public key (for routing)
    pub recipient: PublicKey,
    /// Message visibility
    pub visibility: Visibility,
    /// Creation timestamp
    pub timestamp: u64,
}

impl Envelope {
    /// Create a new envelope (placeholder - would do actual encryption)
    pub fn encrypt(
        message: &Message,
        _sender: &Identity,
        recipient: &PublicKey,
        visibility: Visibility,
    ) -> Self {
        Envelope {
            ciphertext: message.to_json().into_bytes(),
            ephemeral_key: PublicKey::random(),
            recipient: recipient.clone(),
            visibility,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    /// Decrypt an envelope (placeholder)
    pub fn decrypt(&self, _recipient: &Identity) -> Result<Message, crate::RuntimeError> {
        // Stub - real implementation would decrypt
        Ok(Message::new())
    }
}

// ============================================================================
// SEND FUNCTIONS
// ============================================================================

/// Send an encrypted message to a handle
pub fn send_encrypted(recipient: &Handle, message: Message) -> Result<(), crate::RuntimeError> {
    // Stub - would actually send via network
    tracing::info!("Sending encrypted message to {}: {:?}", recipient, message);
    Ok(())
}

/// Send an encrypted message to a facet address
pub fn send_to_facet(facet: &FacetAddress, message: Message) -> Result<(), crate::RuntimeError> {
    tracing::info!("Sending to facet {}: {:?}", facet, message);
    Ok(())
}
