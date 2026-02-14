# ULissy

[![CI](https://github.com/GNS-Foundation/ulissy/actions/workflows/ci.yml/badge.svg)](https://github.com/GNS-Foundation/ulissy/actions/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/tests-74%2F74-brightgreen)](https://github.com/GNS-Foundation/ulissy/blob/main/examples/trip-protocol/src/trip_tests.ul)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/GNS-Foundation/ulissy/blob/main/LICENSE)
[![Version](https://img.shields.io/badge/version-0.4.2-purple)](https://github.com/GNS-Foundation/ulissy)
[![Playground](https://img.shields.io/badge/try%20it-playground-orange)](https://gns-foundation.github.io/ulissy/playground/)

**A Programming Language for Moving Machines**

ULissy is a domain-specific programming language designed for machines that move through physical space — mobile phones, vehicles, drones, wearables, and IoT devices. It treats identity, location, time, cryptography, and energy as first-class language primitives rather than library imports.

ULissy transpiles to Rust and integrates natively with the [GNS Protocol](https://github.com/GNS-Foundation)

*Named after Ulysses (Odysseus) — who proved his identity not through credentials, but through his journey.*

---

## Quick Start

```bash
# Build the compiler
cd compiler/ulissy && cargo build --release && cd ../..

# Run the TRIP Protocol test suite
./compiler/ulissy/target/release/ulissy run examples/trip-protocol/src/trip_tests.ul
```

**Expected output: 74/74 tests passing**

```
╔═══════════════════════════════════════════════════════════════╗
║   TRIP Protocol Test Suite                                    ║
║   Written in ULissy v0.3.0                                    ║
║   "A language for moving machines"                            ║
╚═══════════════════════════════════════════════════════════════╝

── TIT Test Vectors ──
✓ TIT zeros key
✓ TIT ones key
✓ TIT sequential key
...
═══════════════════════════════════════════════════════════════
Results: 74/74 passed
All tests passed! ✓
═══════════════════════════════════════════════════════════════
```

---

## What Makes ULissy Different

```ulissy
// A complete breadcrumb collection program
// Replaces ~250 lines of Rust with ~30 lines of ULissy

identity me = Keychain.primary

config {
    resolution: 7,
    interval: 10.minutes,
    minBreadcrumbsPerEpoch: 100
}

every config.interval when battery > 20 && gps.available {
    let crumb = breadcrumb(
        cell: here.h3(config.resolution),
        context: sensors.digest,
        previous: me.trajectory.last?.hash ?? "genesis"
    ).signed(me)

    me.trajectory.append(crumb)
}

when me.trajectory.pending >= config.minBreadcrumbsPerEpoch {
    let epoch = me.trajectory.bundleEpoch()
    network.publish(epoch)
}
```

No GPS library imports. No cryptography boilerplate. No battery management code. The language handles it.

---

## Repository Structure

```
compiler/                   # ULissy compiler (Rust)
├── crates/
│   ├── ulissy-lexer/       #   Lexer — 64 token types
│   ├── ulissy-parser/      #   Parser — 18 statement types
│   ├── ulissy-types/       #   Type checker — domain-specific types
│   └── ulissy-codegen/     #   Code generator — Rust emission
└── ulissy/                 #   CLI binary

gns-runtime/                # Rust runtime library
├── src/
│   ├── identity.rs         #   Ed25519 keys, TIT, Keychain
│   ├── breadcrumb.rs       #   Breadcrumb chain, signing
│   ├── types.rs            #   Duration, Moment, H3Cell, Hash
│   └── lib.rs              #   Public API

examples/
└── trip-protocol/          # TRIP Protocol implementation
    └── src/
        └── trip_tests.ul   #   74 tests — TIT, identity, signing,
                            #   handles, breadcrumbs, trust, decay

ulissy-stdlib/              # Standard library (9 modules)
ulissy-vscode/              # VS Code extension
docs/whitepaper/            # Language whitepaper
```

---

## Compiler Pipeline

```
 Source (.ul)
      │
      ▼
 ┌─────────┐
 │  Lexer   │  64 token types: handles, facets, units, interpolation
 └─────────┘
      │
      ▼
 ┌─────────┐
 │  Parser  │  18 statement types, self/Self/config, for loops
 └─────────┘
      │
      ▼
 ┌──────────┐
 │  Types   │  Two-pass resolution, domain-specific type checking
 └──────────┘
      │
      ▼
 ┌──────────┐
 │  Codegen │  Rust emission, Cargo.toml generation, async support
 └──────────┘
      │
      ▼
 Generated Rust → cargo build → Native Binary
```

---

## Primitive Types

ULissy's type system reflects the domain of moving machines:

| Category | Types |
|----------|-------|
| **Identity** | `PublicKey`, `Signature`, `Handle`, `TIT`, `SharedSecret` |
| **Spatial** | `H3Cell`, `Location`, `Distance`, `Coordinates` |
| **Temporal** | `Moment`, `Duration`, `Interval` |
| **Crypto** | `Hash`, `Bytes`, `Ciphertext`, `Nonce` |
| **Energy** | `BatteryLevel`, `PowerMode` |

Privacy is enforced at the language level — raw `Coordinates` cannot be transmitted or printed without H3 quantization.

---

## CLI Commands

```bash
ulissy build              # Compile to Rust, then to native binary
ulissy run <file.ul>      # Compile and execute
ulissy check              # Type-check without compiling
ulissy lex <file.ul>      # Show token stream
ulissy parse <file.ul>    # Show AST
ulissy fmt                # Format source code
ulissy new <project>      # Create new project
ulissy info               # Show compiler version and config
```

---

## Test Suite Coverage

The TRIP Protocol test suite validates the core GNS primitives:

| Section | Tests | What It Validates |
|---------|-------|-------------------|
| TIT Vectors | 3 | SHA-256 → 16-byte Trajectory Identity Token |
| Identity Generation | 4 | Ed25519 keys, Stellar address derivation |
| Sign and Verify | 4 | Ed25519 signatures, wrong-key rejection |
| Facet Derivation | 5 | Deterministic key derivation per facet |
| Handle Validation | 12 | @handle rules (lowercase, 1-20 chars, no leading digits) |
| Breadcrumb Genesis | 3 | Genesis block, signature verification |
| Breadcrumb Chain | 5 | Hash linking, index increment, timestamp ordering |
| Trust Levels | 7 | Epoch → trust level mapping |
| Trust Permissions | 11 | Send, claim handle, vouch capabilities |
| Trust Score | 6 | Parisi k=7 computation, decay factor |
| Trust Decay | 5 | Gaussian decay curve (τ₀ = 30 days) |
| Parisi Constants | 3 | k=7, percolation threshold |
| Entity Speeds | 5 | Human, drone, vehicle, robot, anchor limits |
| Complete Lifecycle | 1 | End-to-end identity creation through trust scoring |
| **Total** | **74** | |

---

## GNS Protocol Integration

ULissy is the native language for the GNS Protocol — a decentralized identity system where **Identity = Public Key** and humanity is proven through physical movement (Proof-of-Trajectory), not biometrics.

Key GNS concepts that are ULissy language constructs:

| GNS Concept | ULissy Construct |
|-------------|-----------------|
| Trajectory Identity Token | `TIT` type |
| Breadcrumb | `type Breadcrumb { ... }` with invariants |
| Epoch | `when trajectory.pending >= 100 { bundleEpoch() }` |
| Handle claiming | `@alice` literal syntax |
| Facet addressing | `dix@me.post(...)`, `pay@merchant.request(...)` |
| Trust scoring | `computeTrustScore()` with Parisi k=7 |

---

## Development

```bash
# Prerequisites
rustup (latest stable Rust)

# Build compiler
cd compiler/ulissy
cargo build --release

# Run tests
cd ../..
./compiler/ulissy/target/release/ulissy run examples/trip-protocol/src/trip_tests.ul

# VS Code extension
cd ulissy-vscode
npm install && npm run package
```

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| v0.1.0 | Jan 2025 | Lexer, parser, type checker, initial codegen |
| v0.2.0 | Jan 2025 | Standard library, VS Code extension |
| v0.3.0 | Jan 2025 | TRIP Protocol test suite in ULissy |
| v0.3.1 | Jan 2025 | `self`/`Self`/`config`, `for` loops, assignments, `default` case |
| **v0.1.4 codegen** | **Feb 2025** | **74/74 TRIP tests passing — first fully working compilation** |

---

## License

MIT

---

## Links

- [GNS Protocol](https://github.com/GNS-Foundation)
- [Whitepaper](docs/whitepaper/WHITEPAPER.md)
- [Provisional Patent #63/948,788](https://www.uspto.gov) — Proof-of-Trajectory

---

*"The journey is the proof."*
