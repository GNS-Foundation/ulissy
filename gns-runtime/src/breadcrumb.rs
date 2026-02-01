// ============================================================================
// gns-runtime/src/breadcrumb.rs - UPDATED VERSION
// Breadcrumb creation, signing, and chaining
// ============================================================================

use sha2::{Sha256, Digest};
use crate::{Identity, Hash, Signature, H3Cell, Moment, RuntimeResult, RuntimeError};

// ----------------------------------------------------------------------------
// TIT - Trajectory Identity Tag (128-bit identifier)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct TIT {
    pub data: Vec<u8>,
}

impl TIT {
    /// Create TIT from public key: SHA256(pubkey)[0:16]
    pub fn from_public_key(public_key: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        
        Self {
            data: hash[0..16].to_vec(),
        }
    }
    
    /// Create TIT from Identity
    pub fn from_identity(identity: &Identity) -> Self {
        Self::from_public_key(&identity.public_key.0)
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }
    
    /// Get length (should always be 16)
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

// ----------------------------------------------------------------------------
// BREADCRUMB - Signed location proof
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub owner: TIT,
    pub index: u64,
    pub timestamp: Moment,
    pub cell: H3Cell,
    pub context: Hash,
    pub previous_hash: Hash,
    pub signature: Signature,
}

impl Breadcrumb {
    /// Create a new breadcrumb (unsigned)
    pub fn new(
        owner: TIT,
        index: u64,
        timestamp: Moment,
        cell: H3Cell,
        context: Hash,
        previous_hash: Hash,
    ) -> Self {
        Self {
            owner,
            index,
            timestamp,
            cell,
            context,
            previous_hash,
            signature: Signature(vec![]), // Empty until signed
        }
    }
    
    /// Create a genesis breadcrumb (first in chain)
    pub fn genesis(identity: &Identity, cell: H3Cell, context: Hash) -> Self {
        Self::new(
            TIT::from_identity(identity),
            0,
            Moment::now(),
            cell,
            context,
            Hash::zero(), // Genesis has no previous
        )
    }
    
    /// Create next breadcrumb in chain
    pub fn next(&self, identity: &Identity, cell: H3Cell, context: Hash) -> Self {
        Self::new(
            TIT::from_identity(identity),
            self.index + 1,
            Moment::now(),
            cell,
            context,
            self.hash(),
        )
    }
    
    /// Sign this breadcrumb with an identity


    pub fn sign(mut self, identity: &Identity) -> Self {
        let signable = self.to_signable_bytes();
        self.signature = identity.sign(&signable);
        self
    }
    
    /// Sign this breadcrumb (mutable reference version)
    pub fn sign_mut(&mut self, identity: &Identity) {
        let signable = self.to_signable_bytes();
        self.signature = identity.sign(&signable);
    }
    
    /// Verify the breadcrumb's signature
    pub fn verify(&self, public_key: &[u8]) -> bool {
        let signable = self.to_signable_bytes();
        self.signature.verify(&signable, &crate::PublicKey(public_key.to_vec()))
    }
    
    /// Get the hash of this breadcrumb
    pub fn hash(&self) -> Hash {
        let bytes = self.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Hash(hasher.finalize().to_vec())
    }
    
    /// Convert to bytes for hashing (includes signature)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.to_signable_bytes();
        bytes.extend(&self.signature.0);
        bytes
    }
    
    /// Convert to bytes for signing (excludes signature)
    pub fn to_signable_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Owner TIT (16 bytes)
        bytes.extend(&self.owner.data);
        
        // Index (8 bytes, big-endian)
        bytes.extend(&self.index.to_be_bytes());
        
        // Timestamp (8 bytes)
        bytes.extend(&self.timestamp.to_bytes());
        
        // Cell (8 bytes)
        bytes.extend(&self.cell.0.to_be_bytes());
        
        // Context hash (32 bytes)
        bytes.extend(&self.context.0);
        
        // Previous hash (32 bytes)
        bytes.extend(&self.previous_hash.0);
        
        bytes
    }
}

// ----------------------------------------------------------------------------
// BREADCRUMB BUILDER - Fluent API
// ----------------------------------------------------------------------------

pub struct BreadcrumbBuilder {
    cell: Option<H3Cell>,
    context: Option<Hash>,
    previous: Option<Breadcrumb>,
}

impl BreadcrumbBuilder {
    pub fn new() -> Self {
        Self {
            cell: None,
            context: None,
            previous: None,
        }
    }
    
    pub fn cell(mut self, cell: H3Cell) -> Self {
        self.cell = Some(cell);
        self
    }
    
    pub fn context(mut self, context: Hash) -> Self {
        self.context = Some(context);
        self
    }
    
    pub fn previous(mut self, bc: Option<&Breadcrumb>) -> Self {
        self.previous = bc.cloned();
        self
    }
    
    pub fn build(self, identity: &Identity) -> RuntimeResult<Breadcrumb> {
        let cell = self.cell.ok_or(RuntimeError::InvalidInput("cell required".into()))?;
        let context = self.context.unwrap_or(Hash::zero());
        
        let (index, previous_hash) = match &self.previous {
            Some(prev) => (prev.index + 1, prev.hash()),
            None => (0, Hash::zero()),
        };
        
        let bc = Breadcrumb::new(
            TIT::from_identity(identity),
            index,
            Moment::now(),
            cell,
            context,
            previous_hash,
        );
        
        Ok(bc)
    }
    
    pub fn signed(self, identity: &Identity) -> RuntimeResult<Breadcrumb> {
        let bc = self.build(identity)?;
        Ok(bc.sign(identity))
    }

    /// Alias for signed, matching ULissy .sign()
    pub fn sign(self, identity: &Identity) -> RuntimeResult<Breadcrumb> {
        self.signed(identity)
    }
}

/// Helper function for fluent breadcrumb creation
pub fn breadcrumb() -> BreadcrumbBuilder {
    BreadcrumbBuilder::new()
}

// ----------------------------------------------------------------------------
// TRAJECTORY - Chain of breadcrumbs
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Trajectory {
    breadcrumbs: Vec<Breadcrumb>,
}

impl Trajectory {
    pub fn new() -> Self {
        Self { breadcrumbs: vec![] }
    }
    
    pub fn append(&mut self, bc: Breadcrumb) {
        self.breadcrumbs.push(bc);
    }
    
    pub fn last(&self) -> Option<&Breadcrumb> {
        self.breadcrumbs.last()
    }
    
    pub fn count(&self) -> usize {
        self.breadcrumbs.len()
    }
    
    pub fn pending(&self) -> usize {
        // Breadcrumbs not yet bundled into an epoch
        // For now, return total count
        self.breadcrumbs.len()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Breadcrumb> {
        self.breadcrumbs.iter()
    }
}

// ----------------------------------------------------------------------------
// EPOCH - Bundled trajectory proof
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Epoch {
    pub id: String,
    pub owner: TIT,
    pub count: u32,
    pub merkle_root: Hash,
    pub start_index: u64,
    pub end_index: u64,
    pub start_time: Moment,
    pub end_time: Moment,
    pub signature: Signature,
}

impl Epoch {
    /// Create epoch from a range of breadcrumbs
    pub fn from_breadcrumbs(breadcrumbs: &[Breadcrumb], identity: &Identity) -> RuntimeResult<Self> {
        if breadcrumbs.is_empty() {
            return Err(RuntimeError::InvalidInput("Cannot create epoch from empty breadcrumbs".into()));
        }
        
        let first = &breadcrumbs[0];
        let last = &breadcrumbs[breadcrumbs.len() - 1];
        
        // Compute merkle root
        let merkle_root = compute_merkle_root(breadcrumbs);
        
        // Generate epoch ID
        let id = format!("epoch-{}-{}", first.index, last.index);
        
        let mut epoch = Self {
            id,
            owner: first.owner.clone(),
            count: breadcrumbs.len() as u32,
            merkle_root,
            start_index: first.index,
            end_index: last.index,
            start_time: first.timestamp.clone(),
            end_time: last.timestamp.clone(),
            signature: Signature(vec![]),
        };
        
        // Sign the epoch
        let signable = epoch.to_signable_bytes();
        epoch.signature = identity.sign(&signable);
        
        Ok(epoch)
    }
    
    fn to_signable_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_bytes());
        bytes.extend(&self.owner.data);
        bytes.extend(&self.count.to_be_bytes());
        bytes.extend(&self.merkle_root.0);
        bytes.extend(&self.start_index.to_be_bytes());
        bytes.extend(&self.end_index.to_be_bytes());
        bytes.extend(&self.start_time.to_bytes());
        bytes.extend(&self.end_time.to_bytes());
        bytes
    }
}

fn compute_merkle_root(breadcrumbs: &[Breadcrumb]) -> Hash {
    if breadcrumbs.is_empty() {
        return Hash::zero();
    }
    
    let mut hashes: Vec<Hash> = breadcrumbs.iter().map(|bc| bc.hash()).collect();
    
    while hashes.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in hashes.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(&chunk[0].0);
            if chunk.len() > 1 {
                hasher.update(&chunk[1].0);
            } else {
                hasher.update(&chunk[0].0); // Duplicate if odd
            }
            next_level.push(Hash(hasher.finalize().to_vec()));
        }
        
        hashes = next_level;
    }
    
    hashes.pop().unwrap_or(Hash::zero())
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tit_creation() {
        let id = Identity::new();
        let tit = TIT::from_identity(&id);
        
        assert_eq!(tit.len(), 16);
    }
    
    #[test]
    fn test_breadcrumb_genesis() {
        let id = Identity::new();
        let cell = H3Cell(0x8a283082a677fff);
        let context = Hash::zero();
        
        let bc = Breadcrumb::genesis(&id, cell, context).sign(&id);
        
        assert_eq!(bc.index, 0);
        assert!(bc.verify(&id.public_key.0));
    }
    
    #[test]
    fn test_breadcrumb_chain() {
        let id = Identity::new();
        let cell = H3Cell(0x8a283082a677fff);
        let context = Hash::zero();
        
        let bc0 = Breadcrumb::genesis(&id, cell.clone(), context.clone()).sign(&id);
        let bc1 = bc0.next(&id, cell.clone(), context.clone()).sign(&id);
        let bc2 = bc1.next(&id, cell, context).sign(&id);
        
        assert_eq!(bc0.index, 0);
        assert_eq!(bc1.index, 1);
        assert_eq!(bc2.index, 2);
        
        // Verify chain links
        assert_eq!(bc1.previous_hash, bc0.hash());
        assert_eq!(bc2.previous_hash, bc1.hash());
    }
    
    #[test]
    fn test_breadcrumb_builder() {
        let id = Identity::new();
        let cell = H3Cell(0x8a283082a677fff);
        
        let bc = breadcrumb()
            .cell(cell)
            .context(Hash::zero())
            .previous(None)
            .signed(&id)
            .unwrap();
        
        assert_eq!(bc.index, 0);
        assert!(bc.verify(&id.public_key.0));
    }
    
    #[test]
    fn test_trajectory() {
        let id = Identity::new();
        let cell = H3Cell(0x8a283082a677fff);
        let context = Hash::zero();
        
        let mut trajectory = Trajectory::new();
        
        let bc0 = Breadcrumb::genesis(&id, cell.clone(), context.clone()).sign(&id);
        trajectory.append(bc0.clone());
        
        let bc1 = bc0.next(&id, cell.clone(), context.clone()).sign(&id);
        trajectory.append(bc1);
        
        assert_eq!(trajectory.count(), 2);
    }
    
    #[test]
    fn test_epoch_creation() {
        let id = Identity::new();
        let cell = H3Cell(0x8a283082a677fff);
        let context = Hash::zero();
        
        let mut breadcrumbs = vec![];
        let mut prev: Option<Breadcrumb> = None;
        
        for _ in 0..10 {
            let bc = match &prev {
                Some(p) => p.next(&id, cell.clone(), context.clone()).sign(&id),
                None => Breadcrumb::genesis(&id, cell.clone(), context.clone()).sign(&id),
            };
            breadcrumbs.push(bc.clone());
            prev = Some(bc);
        }
        
        let epoch = Epoch::from_breadcrumbs(&breadcrumbs, &id).unwrap();
        
        assert_eq!(epoch.count, 10);
        assert_eq!(epoch.start_index, 0);
        assert_eq!(epoch.end_index, 9);
    }
}
