// ============================================================================
// gns-runtime/src/types.rs - Core Types
// Basic types used throughout the runtime
// ============================================================================

use std::time::{SystemTime, UNIX_EPOCH, Duration as StdDuration};
use ed25519_dalek::{Verifier, VerifyingKey, Signature as Ed25519Signature};

// ----------------------------------------------------------------------------
// HASH - 32-byte SHA-256 hash
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Hash(pub Vec<u8>);

impl Hash {
    pub fn zero() -> Self {
        Hash(vec![0u8; 32])
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.0[start..end]
    }
}

impl std::ops::Index<std::ops::Range<usize>> for Hash {
    type Output = [u8];
    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        &self.0[range]
    }
}

// ----------------------------------------------------------------------------
// SIGNATURE - 64-byte Ed25519 signature
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Signature(pub Vec<u8>);

impl Signature {
    pub fn empty() -> Self {
        Signature(vec![])
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn verify(&self, data: &[u8], public_key: &PublicKey) -> bool {
        if self.0.len() != 64 || public_key.0.len() != 32 {
            return false;
        }
        
        let sig_bytes: [u8; 64] = match self.0.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        
        let key_bytes: [u8; 32] = match public_key.0.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        
        let signature = Ed25519Signature::from_bytes(&sig_bytes);
        
        let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        
        verifying_key.verify(data, &signature).is_ok()
    }
}

// ----------------------------------------------------------------------------
// PUBLIC KEY - 32-byte Ed25519 public key
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey(pub Vec<u8>);

impl PublicKey {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

// ----------------------------------------------------------------------------
// H3 CELL - Hexagonal grid cell identifier
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct H3Cell(pub u64);

impl H3Cell {
    pub fn from_u64(value: u64) -> Self {
        H3Cell(value)
    }
    
    pub fn to_string(&self) -> String {
        format!("{:x}", self.0)
    }
}

// ----------------------------------------------------------------------------
// MOMENT - Point in time (milliseconds since epoch)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Moment(pub u64);

impl Moment {
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Moment(duration.as_millis() as u64)
    }
    
    pub fn from_millis(ms: u64) -> Self {
        Moment(ms)
    }
    
    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }
    
    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

impl std::cmp::PartialOrd for Moment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// ----------------------------------------------------------------------------
// DURATION - Time span with units
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Duration(pub u64); // milliseconds

impl Duration {
    pub fn from_millis(ms: u64) -> Self {
        Duration(ms)
    }
    
    pub fn from_seconds(secs: u64) -> Self {
        Duration(secs * 1000)
    }
    
    pub fn from_minutes(mins: u64) -> Self {
        Duration(mins * 60 * 1000)
    }
    
    pub fn from_hours(hours: u64) -> Self {
        Duration(hours * 60 * 60 * 1000)
    }
    
    pub fn as_millis(&self) -> u64 {
        self.0
    }
    
    pub fn as_seconds(&self) -> f64 {
        self.0 as f64 / 1000.0
    }
    
    pub fn as_std(&self) -> StdDuration {
        StdDuration::from_millis(self.0)
    }
}

// Duration extension traits for numeric types
pub trait DurationExt {
    fn milliseconds(self) -> Duration;
    fn seconds(self) -> Duration;
    fn minutes(self) -> Duration;
    fn hours(self) -> Duration;
}

impl DurationExt for u64 {
    fn milliseconds(self) -> Duration { Duration::from_millis(self) }
    fn seconds(self) -> Duration { Duration::from_seconds(self) }
    fn minutes(self) -> Duration { Duration::from_minutes(self) }
    fn hours(self) -> Duration { Duration::from_hours(self) }
}

impl DurationExt for i64 {
    fn milliseconds(self) -> Duration { Duration::from_millis(self as u64) }
    fn seconds(self) -> Duration { Duration::from_seconds(self as u64) }
    fn minutes(self) -> Duration { Duration::from_minutes(self as u64) }
    fn hours(self) -> Duration { Duration::from_hours(self as u64) }
}

// ----------------------------------------------------------------------------
// LOCATION - Geographic coordinates
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub timestamp: Moment,
}

impl Location {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            accuracy: 0.0,
            timestamp: Moment::now(),
        }
    }
    
    /// Get current location (mock for now)
    pub fn current() -> crate::RuntimeResult<Self> {
        // In production: use platform GPS APIs
        // For testing: return a mock location
        Ok(Self::new(41.9028, 12.4964)) // Rome
    }
    
    /// Convert to H3 cell at given resolution
    pub fn to_h3(&self, resolution: u8) -> crate::RuntimeResult<H3Cell> {
        // In production: use h3-rs crate
        // For testing: return a mock cell
        let mock_cell = 0x8a283082a677fff_u64 | (resolution as u64);
        Ok(H3Cell(mock_cell))
    }
}

// Global "here" accessor
pub fn here() -> crate::RuntimeResult<Location> {
    Location::current()
}

// ----------------------------------------------------------------------------
// SENSORS - Device sensor context
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SensorContext {
    pub wifi_hash: Hash,
    pub cell_hash: Hash,
    pub motion_hash: Hash,
    pub timestamp: Moment,
}

impl SensorContext {
    pub fn current() -> Self {
        use sha2::{Sha256, Digest};
        
        // Mock sensor data for testing
        let mut wifi_hasher = Sha256::new();
        wifi_hasher.update(b"wifi-mock");
        wifi_hasher.update(&Moment::now().0.to_be_bytes());
        
        let mut cell_hasher = Sha256::new();
        cell_hasher.update(b"cell-mock");
        
        let mut motion_hasher = Sha256::new();
        motion_hasher.update(b"motion-mock");
        
        Self {
            wifi_hash: Hash(wifi_hasher.finalize().to_vec()),
            cell_hash: Hash(cell_hasher.finalize().to_vec()),
            motion_hash: Hash(motion_hasher.finalize().to_vec()),
            timestamp: Moment::now(),
        }
    }
    
    /// Combined digest of all sensor data
    pub fn digest(&self) -> Hash {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&self.wifi_hash.0);
        hasher.update(&self.cell_hash.0);
        hasher.update(&self.motion_hash.0);
        hasher.update(&self.timestamp.0.to_be_bytes());
        
        Hash(hasher.finalize().to_vec())
    }
}

pub struct Sensors;

impl Sensors {
    pub fn current() -> SensorContext {
        SensorContext::current()
    }
    
    pub fn digest() -> Hash {
        SensorContext::current().digest()
    }
}

// ----------------------------------------------------------------------------
// BYTES - Utility for creating byte arrays
// ----------------------------------------------------------------------------

pub struct Bytes;

impl Bytes {
    pub fn zeros(len: usize) -> Vec<u8> {
        vec![0u8; len]
    }
    
    pub fn fill(len: usize, value: u8) -> Vec<u8> {
        vec![value; len]
    }
    
    pub fn sequence(start: u8, len: usize) -> Vec<u8> {
        (0..len).map(|i| start.wrapping_add(i as u8)).collect()
    }
}

// ----------------------------------------------------------------------------
// ERRORS
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub enum RuntimeError {
    InvalidInput(String),
    Io(String),
    Crypto(String),
    Location(String),
    NotFound(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            RuntimeError::Io(s) => write!(f, "IO error: {}", s),
            RuntimeError::Crypto(s) => write!(f, "Crypto error: {}", s),
            RuntimeError::Location(s) => write!(f, "Location error: {}", s),
            RuntimeError::NotFound(s) => write!(f, "Not found: {}", s),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

// ----------------------------------------------------------------------------
// SHA256 Helper
// ----------------------------------------------------------------------------

pub fn sha256(data: &[u8]) -> Hash {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    Hash(hasher.finalize().to_vec())
}

// ----------------------------------------------------------------------------
// MISC TYPES (Added to support generated code)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Distance(pub f64); // meters

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percent(pub f64); // 0.0 to 1.0 (or 100.0)

// Alias for Location?
pub type Coordinates = Location;

pub struct Battery;
pub struct BatteryLevel(pub f64);

// Mock scheduling/async functions
pub async fn schedule_every<F: std::future::Future>(duration: Duration, f: impl Fn() -> F) {
    // Mock implementation
}

pub async fn watch_condition<F: std::future::Future<Output = bool>>(condition: impl Fn() -> F) {
    // Mock implementation
}

pub async fn delay(duration: Duration) {
    std::thread::sleep(std::time::Duration::from_millis(duration.0));
}

// Tokio Handle alias
pub type Handle = tokio::runtime::Handle;

#[derive(Debug, Clone)]
pub struct FacetAddress(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Encrypted,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub message: Message,
    pub signature: Signature,
}

pub async fn send_encrypted(recipient: FacetAddress, message: Message) -> RuntimeResult<()> {
    Ok(()) // Mock
}

// Alias or re-export
pub type GnsError = RuntimeError;
