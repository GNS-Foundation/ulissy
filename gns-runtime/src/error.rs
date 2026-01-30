//! Error types for GNS Runtime

use thiserror::Error;

/// Runtime error type
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Location unavailable: {0}")]
    LocationUnavailable(String),
    
    #[error("GPS not available")]
    GpsUnavailable,
    
    #[error("Battery status unavailable")]
    BatteryUnavailable,
    
    #[error("Sensor error: {0}")]
    SensorError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Scheduling error: {0}")]
    SchedulingError(String),
    
    #[error("Task cancelled")]
    TaskCancelled,
    
    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),
    
    #[error("H3 conversion error: {0}")]
    H3Error(String),
    
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl RuntimeError {
    /// Create a location unavailable error
    pub fn location(msg: impl Into<String>) -> Self {
        RuntimeError::LocationUnavailable(msg.into())
    }
    
    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        RuntimeError::NetworkError(msg.into())
    }
    
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        RuntimeError::Internal(msg.into())
    }
}

/// GNS Error (alias for compatibility with generated code)
pub type GnsError = RuntimeError;
