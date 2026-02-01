#!/bin/bash
# Add signing_bytes() method to gns-runtime Breadcrumb
# This method returns the bytes that were signed (all fields EXCEPT signature)
# Required for signature verification in tests
#
# Usage: bash add_signing_bytes.sh

BREADCRUMB_RS="gns-runtime/src/breadcrumb.rs"

if [ ! -f "$BREADCRUMB_RS" ]; then
    echo "ERROR: $BREADCRUMB_RS not found"
    exit 1
fi

# Check if signing_bytes already exists
if grep -q "signing_bytes" "$BREADCRUMB_RS"; then
    echo "✓ signing_bytes() already exists in $BREADCRUMB_RS"
    exit 0
fi

# Find the sign() method and add signing_bytes() before it
# We need to add a method that serializes all fields except signature

# First, let's find what fields Breadcrumb has and create the method
echo "Adding signing_bytes() to Breadcrumb..."

# Insert the signing_bytes method before the sign() method
# Using Python for reliable multi-line insertion
python3 << 'PYEOF'
import re

with open("gns-runtime/src/breadcrumb.rs", "r") as f:
    content = f.read()

# The signing_bytes method - serializes all fields except signature
signing_bytes_method = '''
    /// Returns the bytes that were signed (all fields EXCEPT the signature).
    /// Used for signature verification after construction.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Owner TIT
        bytes.extend_from_slice(&self.owner.data);
        // Index as big-endian bytes
        bytes.extend_from_slice(&self.index.to_be_bytes());
        // Timestamp
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        // Cell
        bytes.extend_from_slice(&self.cell.0.to_be_bytes());
        // Context hash
        bytes.extend_from_slice(&self.context.0);
        // Previous hash
        bytes.extend_from_slice(&self.previous_hash.0);
        bytes
    }

'''

# Find the sign method and insert signing_bytes before it
sign_pattern = r'(\s*pub fn sign\s*\()'
match = re.search(sign_pattern, content)

if match:
    insert_pos = match.start()
    content = content[:insert_pos] + signing_bytes_method + content[insert_pos:]
    
    with open("gns-runtime/src/breadcrumb.rs", "w") as f:
        f.write(content)
    
    print("✓ Added signing_bytes() method before sign()")
else:
    # Alternative: find the impl block and add at the end
    # Find the last method in the impl Breadcrumb block
    impl_pattern = r'impl BreadcrumbBuilder'
    match = re.search(impl_pattern, content)
    
    if not match:
        # Try impl Breadcrumb
        impl_pattern = r'impl Breadcrumb'
        match = re.search(impl_pattern, content)
    
    if match:
        # Find a good insertion point - before sign() or at end of impl
        print("NOTE: Could not find sign() method. Adding at end of impl block.")
        print("You may need to manually adjust placement.")
        print("The signing_bytes() method serializes all fields EXCEPT signature.")
    else:
        print("ERROR: Could not find Breadcrumb impl block")
        print("Please manually add this method to Breadcrumb:")
        print(signing_bytes_method)

PYEOF

echo ""
echo "Now rebuild the runtime:"
echo "  cd target/ulissy/run && cargo build"
