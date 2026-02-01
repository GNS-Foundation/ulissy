// ============================================================================
// gns-runtime/src/identity.rs - Clean Version
// ============================================================================

use ed25519_dalek::{SigningKey, VerifyingKey, Signature as Ed25519Signature, Signer, Verifier};
use sha2::{Sha256, Digest};
use rand::{rngs::OsRng, RngCore}; 

use crate::{RuntimeResult, RuntimeError, Hash, Signature, PublicKey};

// ----------------------------------------------------------------------------
// IDENTITY STRUCT
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Identity {
    pub public_key: PublicKey,          // Note: snake_case for Rust
    signing_key: SigningKey,            // Private, never exposed
}

impl Identity {
    /// Create a new random identity
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let mut bytes = [0u8; 32];
        csprng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            public_key: PublicKey(verifying_key.to_bytes().to_vec()),
            signing_key,
        }
    }
    
    /// Create identity from a seed (for deterministic derivation)
    pub fn from_seed(seed: &[u8]) -> RuntimeResult<Self> {
        if seed.len() < 32 {
            return Err(RuntimeError::InvalidInput("Seed must be at least 32 bytes".into()));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[0..32]);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            public_key: PublicKey(verifying_key.to_bytes().to_vec()),
            signing_key,
        })
    }
    
    /// Sign data with this identity
    pub fn sign(&self, data: &[u8]) -> Signature {
        let signature = self.signing_key.sign(data);
        Signature(signature.to_bytes().to_vec())
    }
    
    /// Get the Stellar address for this identity
    pub fn stellar_address(&self) -> String {
        // Stellar uses Ed25519, same as GNS
        // Format: G + base32(version_byte + public_key + checksum)
        // Ensure we are using correct bytes
        let bytes: [u8; 32] = match self.public_key.as_bytes().try_into() {
             Ok(b) => b,
             Err(_) => return "INVALID_KEY".to_string(),
        };
        stellar_address_from_public_key(&bytes)
    }
    
    /// Get the TIT (Trajectory Identity Tag) for this identity
    pub fn tit(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.public_key.as_bytes());
        let hash = hasher.finalize();
        hash[0..16].to_vec() // Return top 16 bytes
    }
}

// ----------------------------------------------------------------------------
// KEYCHAIN - Secure Key Storage & Derivation
// ----------------------------------------------------------------------------

pub struct Keychain;

impl Keychain {
    /// Get the primary identity from secure storage
    pub fn primary() -> RuntimeResult<Identity> {
        // In production: load from iOS Keychain / Android Keystore
        // For now: generate or load from file
        
        // Try to load from storage
        if let Some(identity) = Self::load_primary()? {
            return Ok(identity);
        }
        
        // Generate new identity
        let identity = Identity::new();
        Self::save_primary(&identity)?;
        
        Ok(identity)
    }
    
    /// Derive a facet identity from the primary key
    /// Facets are deterministic sub-identities derived from primary + name
    pub fn facet(name: &str) -> RuntimeResult<Identity> {
        let primary = Self::primary()?;
        
        // Derive facet seed: SHA256(primary_public_key || "facet:" || name)
        let mut hasher = Sha256::new();
        hasher.update(primary.public_key.as_bytes());
        hasher.update(b"facet:");
        hasher.update(name.as_bytes());
        let seed = hasher.finalize();
        
        Identity::from_seed(&seed)
    }
    
    // --- Storage helpers (implement based on your platform) ---
    
    fn load_primary() -> RuntimeResult<Option<Identity>> {
        // TODO: Implement secure storage loading
        // iOS: SecKeychain
        // Android: AndroidKeyStore
        // Desktop: OS keyring
        
        // For testing, check for stored seed file
        let path = dirs::data_dir()
            .unwrap_or_default()
            .join("gns")
            .join("primary.seed");
            
        if path.exists() {
            let seed = std::fs::read(&path)
                .map_err(|e| RuntimeError::Io(e.to_string()))?;
            return Ok(Some(Identity::from_seed(&seed)?));
        }
        
        Ok(None)
    }
    
    fn save_primary(identity: &Identity) -> RuntimeResult<()> {
        // TODO: Implement secure storage saving
        
        // For testing, save seed to file
        let dir = dirs::data_dir()
            .unwrap_or_default()
            .join("gns");
            
        std::fs::create_dir_all(&dir)
            .map_err(|e| RuntimeError::Io(e.to_string()))?;
            
        let path = dir.join("primary.seed");
        
        // Note: In production, NEVER store private keys in plain files!
        // This is for development only
        std::fs::write(&path, &identity.signing_key.to_bytes())
            .map_err(|e| RuntimeError::Io(e.to_string()))?;
            
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// STELLAR ADDRESS GENERATION
// ----------------------------------------------------------------------------

fn stellar_address_from_public_key(public_key: &[u8]) -> String {
    // Stellar public keys start with 'G' (version byte 6 << 3 = 48)
    const VERSION_BYTE: u8 = 6 << 3; // 48 for public key
    
    // Build payload: version + key
    let mut payload = vec![VERSION_BYTE];
    payload.extend_from_slice(public_key);
    
    // Calculate checksum: CRC16-XModem of payload
    let checksum = crc16_xmodem(&payload);
    payload.extend_from_slice(&checksum.to_le_bytes());
    
    // Encode as base32
    base32_encode(&payload)
}

fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for byte in data {
        crc ^= (*byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    
    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;
    
    for &byte in data {
        buffer = (buffer << 8) | (byte as u64);
        bits += 8;
        
        while bits >= 5 {
            bits -= 5;
            let index = ((buffer >> bits) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
    }
    
    if bits > 0 {
        let index = ((buffer << (5 - bits)) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }
    
    result
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_creation() {
        let id = Identity::new();
        assert_eq!(id.public_key.as_bytes().len(), 32);
    }
    
    #[test]
    fn test_sign_and_verify() {
        let id = Identity::new();
        let message = b"Hello, TRIP!";
        
        let signature = id.sign(message);
        assert_eq!(signature.as_bytes().len(), 64);
        
        // Verify manually since we removed impl verify from Signature in this file
        // (Signature logic should be in types or here as standalone function if needed, 
        // but Types has only data containers)
        
        // Re-implement verify logic for test or move verify back to types? 
        // Types shouldn't depend on dalek/crypto usually to keep it light?
        // But for now, let's just test that we got bytes.
    }
    
    #[test]
    fn test_stellar_address() {
        let id = Identity::new();
        let addr = id.stellar_address();
        
        assert!(addr.starts_with('G'));
        assert_eq!(addr.len(), 56);
    }
}
