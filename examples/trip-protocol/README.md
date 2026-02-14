# TRIP Protocol - ULissy Implementation

> **"A language for moving machines testing a protocol for moving identities"**

## Quick Start

```bash
# Clone ULissy compiler (if needed)
git clone https://github.com/GNS-Foundation/ULissy_Program
cd ULissy_Program
cargo build --release

# Run TRIP tests
ulissy run src/trip_tests.ul

# Or type-check only
ulissy check src/trip_tests.ul

# Build to Rust
ulissy build
```

## Project Structure

```
trip-ulissy-v2/
├── ulissy.toml         # Project configuration
├── src/
│   ├── main.ul         # Entry point
│   └── trip_tests.ul   # TRIP test suite
└── README.md
```

## Test Coverage

The test suite covers:

1. **TIT Test Vectors** - SHA-256(pubkey)[0:16] computation
2. **Identity Generation** - Ed25519 keypairs, Stellar addresses
3. **Sign and Verify** - Message signing, verification
4. **Facet Derivation** - Deterministic sub-identities
5. **Handle Validation** - @handle format rules
6. **Breadcrumb Genesis** - First breadcrumb in chain
7. **Breadcrumb Chain** - Linked location proofs
8. **Trust Levels** - Anonymous → Verified → Established → Trusted
9. **Trust Permissions** - canSend, canClaimHandle, canVouch
10. **Trust Score** - Parisi dynamics (k=7)
11. **Trust Decay** - Temporal trust erosion
12. **Parisi Constants** - k=7, percolation threshold
13. **Entity Speeds** - Human, drone, vehicle limits
14. **Complete Lifecycle** - End-to-end demo

## Test Vectors

```
TIT (zeros key):  66687aadf862bd776c8fc18b8e9f8e20
TIT (ones key):   af9613760f72635fbdb44a5a0a63c39f
TIT (seq key):    bb63cf9f8f72d4c9e7f0be8ff64bc259
```

## Expected Output

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   TRIP Protocol Test Suite                                    ║
║   Written in ULissy v0.3.0                                    ║
║   "A language for moving machines"                            ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

── TIT Test Vectors ──
✓ TIT zeros key
✓ TIT ones key
✓ TIT sequential key

── Identity Generation ──
✓ Public key is 32 bytes
✓ TIT is 16 bytes
✓ Stellar address starts with G
✓ Stellar address is 56 chars

... (more tests)

═══════════════════════════════════════════════════════════════
Results: 50/50 passed
All tests passed! ✓
═══════════════════════════════════════════════════════════════
```

## CI/CD Integration

The ULissy compiler has GitHub Actions CI/CD. Tests run automatically on push.

## Philosophy

```
╔═════════════════════════════════════════════════════════════════╗
║                                                                 ║
║   "TRIP identities are MOVING patterns that must be             ║
║    continuously renewed through physical-world activity."       ║
║                                                                 ║
║   The trajectory IS the identity.                               ║
║                                                                 ║
╚═════════════════════════════════════════════════════════════════╝
```

---

*TRIP Protocol v1.1.0 - Written in ULissy v0.3.0*
