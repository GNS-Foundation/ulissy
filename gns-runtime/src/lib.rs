// ============================================================================
// gns-runtime/src/lib.rs - GNS Runtime Library
// Core runtime for ULissy programs
// ============================================================================

// Re-export all modules
mod types;
mod identity;
mod breadcrumb;

// Public exports
pub use types::*;
pub use identity::*;
pub use breadcrumb::*;

// Re-export commonly used items at crate root
pub use types::{
    Hash, Signature, PublicKey, H3Cell, Moment, Duration, DurationExt,
    Location, SensorContext, Sensors, Bytes,
    RuntimeError, RuntimeResult,
    sha256, here,
};

pub use identity::{Identity, Keychain};

pub use breadcrumb::{
    TIT, Breadcrumb, BreadcrumbBuilder, breadcrumb,
    Trajectory, Epoch,
};

// ----------------------------------------------------------------------------
// PRELUDE - Import everything with `use gns_runtime::prelude::*`
// ----------------------------------------------------------------------------

pub mod prelude {
    pub use crate::{
        // Types
        Hash, Signature, PublicKey, H3Cell, Moment, Duration, DurationExt,
        Location, SensorContext, Sensors, Bytes,
        RuntimeError, RuntimeResult,
        
        // Identity
        Identity, Keychain,
        
        // Breadcrumb
        TIT, Breadcrumb, BreadcrumbBuilder, breadcrumb,
        Trajectory, Epoch,
        
        // Functions
        sha256, here,
    };
}

// ----------------------------------------------------------------------------
// VERSION
// ----------------------------------------------------------------------------

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn version() -> &'static str {
    VERSION
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::prelude::*;
    
    #[test]
    fn test_full_workflow() {
        // 1. Get identity
        let me = Identity::new();
        
        // 2. Get location
        let location = Location::new(41.9028, 12.4964);
        let cell = location.to_h3(7).unwrap();
        
        // 3. Get sensor context
        let context = Sensors::digest();
        
        // 4. Create breadcrumb
        let bc = breadcrumb()
            .cell(cell)
            .context(context)
            .previous(None)
            .signed(&me)
            .unwrap();
        
        // 5. Verify
        assert_eq!(bc.index, 0);
        assert!(bc.verify(&me.public_key.0));
        
        println!("Full workflow test passed!");
        println!("  Identity: {}", me.stellar_address());
        println!("  TIT: {}", TIT::from_identity(&me).to_hex());
        println!("  Breadcrumb hash: {}", bc.hash().to_hex());
    }
}
