# ULissy
## A Programming Language for Moving Machines

**WHITEPAPER**
*Technical Edition for Language Designers & Protocol Engineers*
*2025–2026 — Version 0.4.2 (IETF I-D: draft-ayerbe-trip-protocol-00)*

**ULISSY Foundation / GNS Protocol**

---

## Abstract

ULissy is a domain-specific programming language designed for machines that move through physical space—mobile phones, vehicles, drones, wearables, and IoT devices. By treating **identity**, **location**, **time**, **cryptography**, and **energy** as first-class language primitives rather than library imports, ULissy enables developers to write secure, efficient, spatially-aware applications with compile-time safety guarantees.

ULissy transpiles to Rust and integrates natively with the GNS Protocol (Geospatial Naming System), providing a unified development experience from source code to deployed application across iOS, Android, desktop, and WebAssembly targets.

The name honors Ulysses (Odysseus)—the mythological figure who proved his identity not through credentials, but through his journey. In ULissy, as in the Odyssey, **trajectory is identity**.

---

## Implementation Status

| Component | Status | Description |
|-----------|--------|-------------|
| **Lexer** | ✅ Complete | 68 token types, handles, facets, interpolation, `search`/`nearby`/`ranked`/`by` keywords |
| **Parser** | ✅ Complete | 20 statement types, full expression parsing, search expressions, `if let` binding |
| **Type Checker** | ✅ Complete | Domain-specific types, inference, validation, two-pass function resolution, privacy enforcement |
| **Code Generator** | ✅ Complete | Rust emission, Cargo.toml generation, typed search builder pattern |
| **CLI** | ✅ Complete | `ulissy build`, `check`, `run`, `new`, `lex`, `parse`, `fmt`, `info` |
| **Runtime** | ✅ Complete | `gns-runtime` crate with scheduling, sensors, location |
| **Standard Library** | ✅ Complete | 9 modules: core, spatial, crypto, temporal, sensors, trajectory, messaging, network, prelude |
| **VS Code Extension** | ✅ Complete | Syntax highlighting, 40+ snippets, commands, diagnostics |

---

## Changelog

### Version 0.4.2 (February 2026)

**Optional Binding:**

1. **`if let` statement** — Optional binding syntax for safe unwrapping of `Optional<T>` values. Binds the unwrapped value as `T` in the then-block scope; the else-block executes when the value is nil. Completes ULissy's optional story alongside `?.` chaining and `match/case nil`.
   ```ulissy
   let alice = search @alice              // Optional<SearchResult>
   if let peer = alice {
       print(peer.handle)                 // peer is SearchResult (unwrapped)
   } else {
       print("Not found")
   }
   ```

2. **`else if let` chaining** — `ElseIfLet` variant in `ElseBranch` enables chaining optional bindings in conditional chains.

**Unit Alias Expansion:**

3. **Singular and abbreviated unit suffixes** — Added singular forms (`meter`, `kilometer`, `second`, `minute`, `hour`, `day`) and abbreviations (`m`, `km`, `sec`, `min`, `hr`) alongside existing plural forms. All resolve to correct `Distance` or `Duration` types.
   ```ulissy
   500.m           // Distance (same as 500.meters)
   5.km            // Distance (same as 5.kilometers)
   1.hour          // Duration (same as 1.hours)
   30.min          // Duration (same as 30.minutes)
   ```

**Type System Fixes:**

4. **`Nil` assignable to `Optional<T>`** — `is_assignable_from` now correctly allows assigning `nil` to any optional type.

5. **`TrustScore` comparable with `Float`** — `binary_result` updated to allow trust score comparisons in search filter contexts (e.g., `peer.trust > 0.5`).

### Version 0.4.1 (February 2026)

**TrIP Search — Typed Filters & Full Query Coverage (Patch):**

1. **Expanded `SearchTarget`** — Added `Within { center, radius }`, `Identity { handle }`, and `Text { query }` variants. Parser now handles `search within(center, radius)`, `search @handle`, and `search "text"` in addition to `search nearby`.

2. **Typed `SearchFilter` enum** — Replaced generic `{ field, op, value }` struct with semantic variants: `TrustThreshold`, `FacetMatch`, `ActiveWithin`, `OrgMatch`, `FieldCompare`. Compile-time validation for each filter type.

3. **`PresenceProof` member access** — `SearchResult.proof` now exposes `epoch_count`, `spatial_diversity`, `trajectory_continuity`, `verification_level`, and `first_seen` fields.

4. **`Optional<SearchResult>` for identity lookups** — `search @handle` returns `Optional<SearchResult>` instead of `SearchResultSet`, enabling `if let` unwrapping.

5. **Ranking key expansion** — Added `Recency` and `Relevance` (composite TrustRank) ranking keys alongside existing `Trust` and `Distance`. Parser accepts `age` as alias for `recency`.

6. **Typed code generation** — Codegen emits typed builder methods (`.trust_min()`, `.facet()`, `.active_within()`, `.org()`, `.within()`, `.identity()`, `.text()`, `RankingKey::`) instead of generic `.filter()` calls.

### Version 0.4.0 (February 2026)

**TrIP Search Primitives — Language-Level Spatial Discovery:**

1. **`search` keyword** — New expression type for spatial-identity queries. Search is a first-class language construct, not a library call. Composes with `let`, `for`, `every`, `when` blocks.

2. **`nearby`, `ranked`, `by` keywords** — Added to lexer alongside existing `within`/`where`. `search nearby(500.meters) where trust > 0.5 ranked by distance asc` is a complete expression.

3. **Search AST nodes** — `SearchExpr`, `SearchTarget`, `SearchFilter`, `SearchRanking` with `asc`/`desc` ordering.

4. **7 new types** — `TrustScore`, `SearchResult`, `SearchResultSet`, `PresenceProof`, `SearchQuery`, `SpatialFilter`, `IdentityFilter` added to the type system.

5. **Compile-time privacy enforcement** — `reject_private_in_search()` gate blocks `Breadcrumb`, `Trajectory`, `PrivateKey`, `Identity`, and `Array<private>` from appearing in search contexts. If the code compiles, it respects the TrIP privacy model.

6. **Builder-pattern code generation** — Search expressions compile to `gns_search::query()` with chained `.nearby()`, `.trust_min()`, `.rank_by()` calls, emitting idiomatic Rust.

### Version 0.3.1 (January 2025)

**Parser Enhancements:**

1. **`self` keyword support** — Added handling for `SelfLower` token so `self` can be used in expressions (e.g., in type invariants like `signature.verify(self, owner)`)

2. **`Self` keyword support** — Added handling for `SelfUpper` token for type references

3. **`config` keyword as expression** — Added handling for `Config` token so `config.fieldName` works in expressions, enabling compile-time configuration access

4. **`default` case in match statements** — Added `Default` keyword to lexer and parser to support `default:` as a wildcard pattern in match statements

5. **`if` expressions** — Added `parse_if_expression()` so `if` can be used in expression context:
   ```ulissy
   let x = if condition { a } else { b }
   ```

6. **`for` statements** — Added full `ForStatement` AST node and `parse_for_statement()` for iteration:
   ```ulissy
   for item in collection {
       process(item)
   }
   ```

7. **Assignment expressions** — Added `AssignmentExpr` AST node and assignment parsing so variable reassignments work:
   ```ulissy
   var counter = 0
   counter = counter + 1
   ```

**Lexer Enhancements:**

1. **Added `Default` keyword token** — For match statement default/wildcard cases

**AST Enhancements:**

1. Added `ForStatement` struct for for-in loops
2. Added `AssignmentExpr` struct for assignment expressions
3. Added `ForStatement` variant to `Statement` enum
4. Added `Assignment` variant to `Expression` enum

**Type Checker Enhancements:**

1. **Two-pass type checking** — Added a first pass to collect all function declarations before checking statements, enabling forward references to functions. This allows functions to call other functions defined later in the file.

---

## 1. The Problem: Why Moving Machines Need a New Language

### 1.1 The Current State of Mobile Development

Developers building applications for moving machines face a fragmented landscape:

| Concern | Current Approach | Problems |
|---------|------------------|----------|
| Location | GPS libraries (CoreLocation, FusedLocation) | Platform-specific APIs, raw coordinates leak privacy |
| Identity | OAuth, JWT, platform accounts | Centralized, password-based, SIM-swappable |
| Cryptography | OpenSSL, libsodium bindings | Complex APIs, easy to misuse, silent failures |
| Time | Unix timestamps, Date libraries | No duration types, timezone bugs, no intervals |
| Energy | Platform battery APIs | Afterthought, not integrated into program logic |
| Connectivity | Reachability libraries | Boolean online/offline, no mesh or degraded states |

Developers must manually coordinate these concerns, leading to:

- **Security vulnerabilities**: Cryptographic misuse, key management errors
- **Privacy leaks**: Raw GPS coordinates stored and transmitted
- **Battery drain**: Naive polling without power awareness
- **Platform fragmentation**: Rewrite for iOS, Android, web
- **Protocol violations**: Application logic diverges from specification

### 1.2 The Missing Abstraction

We have languages optimized for:

- **Systems programming**: C, Rust → memory, performance
- **Web development**: JavaScript, TypeScript → DOM, async
- **Data science**: Python, R → matrices, statistics
- **Mobile UI**: Swift, Kotlin → screens, gestures

But no language treats **physical space and movement** as fundamental concepts.

ULissy fills this gap.

### 1.3 Design Philosophy

ULissy is built on five principles:

1. **Spatial-First**: Location is a primitive type, not a library call
2. **Secure by Default**: Encryption and signing happen automatically
3. **Energy-Aware**: Power consumption is part of the type system
4. **Protocol-Native**: GNS constructs are language constructs
5. **Single Source**: One codebase compiles to all platforms

---

## 2. Language Overview

### 2.1 A Complete ULissy Program

```ulissy
// trajectory.ul - Proof-of-Trajectory Collection
// Replaces ~250 lines of Rust with ~50 lines of ULissy

import ulissy.prelude

// Identity declaration - loads from secure keychain
identity me = Keychain.primary

// Module configuration - compile-time constants
config {
    resolution: 7,
    interval: 10.minutes,
    minBreadcrumbsPerEpoch: 100
}

// Enum for location sources
enum LocationSource { gps, wifi, cell, ip, manual }

// Type with invariant (data integrity guarantee)
type Breadcrumb {
    cell: H3Cell
    timestamp: Moment
    source: LocationSource
    context: Hash
    previousHash: Hash
    signature: Signature
    
    invariant timestamp <= now
    invariant signature.verify(self, self.identity)
}

// Reactive computed property - auto-updates when dependencies change
computed status: CollectionStatus {
    isActive: collection.running,
    totalCount: me.trajectory.count,
    pendingCount: me.trajectory.pending,
    epochCount: me.trajectory.epochs.count,
    progress: me.trajectory.pending / config.minBreadcrumbsPerEpoch
}

// Main collection loop
every config.interval when battery > 20 && gps.available {
    let crumb = breadcrumb(
        cell: here.h3(config.resolution),
        timestamp: now,
        source: .gps,
        context: sensors.digest,
        previous: me.trajectory.last?.hash ?? "genesis"
    ).signed(me)
    
    me.trajectory.append(crumb)
}

// Epoch publishing trigger
when me.trajectory.pending >= config.minBreadcrumbsPerEpoch {
    let epoch = me.trajectory.bundleEpoch()
    network.publish(epoch)
}
```

This program:
- Retrieves the user's cryptographic identity from secure storage
- Configures collection parameters at module level
- Every 10 minutes (if battery permits), records an H3-quantized location
- Chains each breadcrumb cryptographically to the previous
- Automatically publishes epochs when threshold is reached
- Provides a reactive `status` property that updates automatically

The equivalent in Swift + CoreLocation + CryptoKit would be 300+ lines with manual coordination between frameworks.

### 2.2 Syntax Principles

ULissy syntax follows these guidelines:

- **Familiar**: Influenced by Swift, Rust, and TypeScript
- **Minimal keywords**: Domain concepts over generic constructs
- **Explicit over implicit**: No hidden behavior
- **Readable as specification**: Code reads like protocol documentation

### 2.3 File Extension

ULissy source files use the `.ul` extension:

```
identity.ul
trajectory.ul
messaging.ul
payments.ul
```

---

## 3. Type System

### 3.1 Primitive Types

ULissy's primitive types reflect the domain of moving machines:

#### Identity Primitives

| Type | Description | Size |
|------|-------------|------|
| `PublicKey` | Ed25519 public key | 32 bytes |
| `PrivateKey` | Ed25519 private key (enclave-bound) | 32 bytes |
| `Signature` | Ed25519 signature | 64 bytes |
| `Handle` | GNS @identifier | 1-20 chars |
| `SharedSecret` | X25519 ECDH output | 32 bytes |

#### Spatial Primitives

| Type | Description | Details |
|------|-------------|---------|
| `H3Cell` | Hexagonal grid cell | 64-bit identifier |
| `Resolution` | H3 precision level | 0-15 |
| `Distance` | Unit-aware length | meters, km, miles |
| `Heading` | Compass direction | 0-360 degrees |
| `Coordinates` | Lat/long pair | Internal only, not exposable |

#### Temporal Primitives

| Type | Description | Examples |
|------|-------------|----------|
| `Moment` | Instant in time | `now`, `crumb.timestamp` |
| `Duration` | Time span with units | `10.minutes`, `2.hours` |
| `Interval` | Recurring pattern | `every 30.seconds` |

#### Cryptographic Primitives

| Type | Description | Algorithm |
|------|-------------|-----------|
| `Hash` | Digest value | SHA-256 |
| `Ciphertext` | Encrypted data | ChaCha20-Poly1305 |
| `Nonce` | Single-use value | 12 bytes |

#### Energy Primitives

| Type | Description | Values |
|------|-------------|--------|
| `BatteryLevel` | Current charge | 0-100% |
| `PowerMode` | Energy profile | `.low`, `.normal`, `.performance` |

### 3.2 Composite Types

Primitives compose into protocol structures:

```ulissy
type Breadcrumb {
    index:      Uint64
    timestamp:  Moment
    cell:       H3Cell
    context:    Hash        // WiFi + cell + IMU digest
    previous:   Hash        // Chain link
    signature:  Signature   // Ed25519 proof
    
    invariant timestamp > previous.timestamp
    invariant signature.valid(for: self, by: owner)
    invariant cell.reachable(from: previous.cell, within: timestamp - previous.timestamp)
}

type Trajectory = Chain<Breadcrumb>

type Identity {
    publicKey:   PublicKey
    privateKey:  PrivateKey     // Never leaves enclave
    handle:      Handle?        // Optional until claimed
    trajectory:  Trajectory
    
    computed trustScore: Float = trajectory.count / 100.0
}

type Envelope<T> {
    sender:      PublicKey
    recipient:   PublicKey
    ephemeral:   PublicKey      // Forward secrecy
    nonce:       Nonce
    ciphertext:  Ciphertext     // Encrypted T
    signature:   Signature
}
```

### 3.3 Constrained Types

Types can have compile-time constraints:

```ulissy
// A handle requires proof of humanity
type Handle = String
    where length in 1..20
    where chars in [a-z, 0-9, _]
    where owner.breadcrumbs >= 100

// Distance is unit-aware
type Distance = Number with Unit<Length>

let d1 = 500.meters
let d2 = 2.kilometers
let d3 = d1 + d2              // OK: 2500.meters

let t = 10.minutes
let x = d1 + t                // COMPILE ERROR: cannot add Distance + Duration
```

### 3.4 The Privacy Type Modifier

Raw location data cannot be exposed without explicit quantization:

```ulissy
let raw = gps.current           // Type: Coordinates (restricted)
print(raw)                      // COMPILE ERROR: Coordinates not printable
send(raw, to: server)           // COMPILE ERROR: Coordinates not transmittable

let cell = raw.h3(resolution: 10)   // Type: H3Cell (safe)
print(cell)                         // OK
send(cell, to: server)              // OK
```

This enforces privacy at the language level.

---

## 4. Syntax Specification

### 4.1 Declarations

```ulissy
// Identity declaration
identity me = Keychain.primary
identity work = Keychain.facet("microsoft")

// Variable declaration
let x = 42                      // Immutable
var counter = 0                 // Mutable

// Type annotation (optional, inferred)
let cell: H3Cell = here.h3(10)

// Constants
const BREADCRUMB_THRESHOLD = 100
const COLLECTION_INTERVAL = 10.minutes
```

### 4.2 Control Flow

```ulissy
// If statement
if battery > 20 {
    collectBreadcrumb()
} else {
    enterLowPowerMode()
}

// If expression (v0.3.1+)
let status = if connected { "online" } else { "offline" }

// Match statement with default (v0.3.1+)
match trustLevel {
    case .anonymous: { denyAccess() }
    case .verified: { allowBasic() }
    case .trusted: { allowFull() }
    default: { denyAccess() }
}

// For loops (v0.3.1+)
for breadcrumb in trajectory {
    verify(breadcrumb)
}

for i in 0..10 {
    print(i)
}
```

### 4.3 Assignment (v0.3.1+)

```ulissy
var counter = 0
counter = counter + 1       // Reassignment

var total = 0
for value in values {
    total = total + value   // Accumulator pattern
}
```

### 4.4 Self References (v0.3.1+)

```ulissy
type Breadcrumb {
    signature: Signature
    owner: TIT
    
    // 'self' refers to the current instance
    invariant signature.verify(self, owner)
    
    fn hash() -> Hash {
        return sha256(self.bytes())
    }
}

// 'Self' refers to the current type
impl Breadcrumb {
    fn genesis() -> Self {
        return Self { ... }
    }
}
```

### 4.5 Config Access (v0.3.1+)

```ulissy
config {
    resolution: 7,
    minBreadcrumbs: 100
}

// Access config values in expressions
let cell = here.h3(config.resolution)

if count >= config.minBreadcrumbs {
    publishEpoch()
}
```

### 4.6 Facet Addressing

GNS facets are first-class syntax:

```ulissy
// Protocol facets (hardcoded prefixes)
dix@me.post("Hello world", visibility: .public)
home@me/lights.set(brightness: 80%)
pay@merchant.request(50.USD)
email@alice.send(subject: "Meeting", body: message)
car@me.unlock()

// Organization facets (registered prefixes)
microsoft@me.authenticate()
stanford@researcher.verify()
```

### 4.7 Temporal Constructs

```ulissy
// Periodic execution
every 10.minutes {
    collectBreadcrumb()
}

// Conditional periodic
every 30.seconds when battery > 20 && connectivity.available {
    sync()
}

// Delayed execution
after 5.seconds {
    dismissNotification()
}

// Triggered execution
when me.trajectory.count >= 100 {
    print("Ready to claim @handle!")
}
```

---

## 5. Tooling

### 5.1 Command Line Interface

```bash
# Create new project
ulissy new my-app

# Compile
ulissy build
ulissy build --target ios
ulissy build --release

# Run
ulissy run

# Run specific file
ulissy run src/trip_tests.ul

# Check without compiling
ulissy check

# Format code
ulissy fmt

# Run tests
ulissy test

# Generate documentation
ulissy doc

# Package for distribution
ulissy package

# Debug: show tokens
ulissy lex src/main.ul

# Debug: show AST
ulissy parse src/main.ul
```

### 5.2 Project Structure

```
my-app/
├── ulissy.toml          # Project configuration
├── src/
│   ├── main.ul          # Entry point
│   ├── identity.ul
│   └── breadcrumbs.ul
├── tests/
│   └── integration.ul
├── assets/              # Icons, images
└── target/              # Build output
```

### 5.3 Configuration (ulissy.toml)

```toml
[package]
name = "my-app"
version = "0.1.0"
authors = ["Developer <dev@example.com>"]

[dependencies]
gns-crypto-core = "1.0"
gns-runtime = "1.0"

[targets]
default = ["ios", "android"]

[identity]
minimum-breadcrumbs = 100
h3-resolution = 10
collection-interval = "10m"

[build]
optimize = "size"  # or "speed"
```

### 5.4 IDE Support

- **VS Code Extension**: Syntax highlighting, autocomplete, error diagnostics
- **Language Server Protocol (LSP)**: Editor-agnostic support (planned v1.0)
- **Inline Documentation**: Hover for type information and docs

---

## 6. Standard Library Reference

```ulissy
// Import everything
import ulissy.prelude

// Identity
identity me = Keychain.primary
identity work = Keychain.facet("company")

// Location
let cell = here.h3(7)                    // Current H3 cell
let dist = 500.meters                    // Distance
let nearby = neighbors(cell1, cell2)     // Adjacency check

// Time
let timestamp = now                      // Current moment
let delay = 10.minutes                   // Duration
let future = now.plus(1.hours)           // Arithmetic

// Crypto
let hash = sha256(data)                  // Hashing
let sig = me.sign(data)                  // Signing
let valid = sig.verify(data, key)        // Verification

// Sensors
if battery > 20 { ... }                  // Battery level
if gps.available { ... }                 // GPS state
let digest = sensors.digest              // Sensor context

// Trajectory
let crumb = breadcrumb(cell: cell).signed(me)
me.trajectory.append(crumb)
let epoch = me.trajectory.bundleEpoch()

// Network
network.publish(epoch)
network.sync()
let record = lookupHandle(@alice)
```

---

## 7. Compiler Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     ULissy Compiler Pipeline                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Source (.ul)                                                   │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Lexer     │  64 token types                                │
│  │             │  Handles, facets, units                        │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Parser    │  18 statement types                            │
│  │             │  Full expression support                       │
│  │             │  self/Self/config (v0.3.1)                     │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │ Type Check  │  Two-pass resolution (v0.3.1)                  │
│  │             │  Domain-specific types                         │
│  │             │  Constraint validation                         │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │  Codegen    │  Rust emission                                 │
│  │             │  Cargo.toml generation                         │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  Generated Rust → cargo build → Native Binary                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. Standards Status

The TrIP (Trajectory-based Recognition of Identity Proof) protocol, which defines the Proof-of-Trajectory mechanism implemented by ULissy's `trajectory` module, has been formally submitted to the Internet Engineering Task Force (IETF) as an Individual Internet-Draft:

> **draft-ayerbe-trip-protocol-00**
> *TrIP: Trajectory-based Recognition of Identity Proof*
> February 2026 — Informational
> https://datatracker.ietf.org/doc/draft-ayerbe-trip-protocol/

The Internet-Draft specifies TrIP as a standalone, transport-agnostic protocol independent of ULissy, GNS, or any specific application layer. It defines:

- **Breadcrumb wire format** using CBOR (RFC 8949) with deterministic encoding
- **Epoch aggregation** with Merkle tree integrity proofs
- **Trajectory Identity Token (TIT)** derivation as a 128-bit pseudonym
- **Trust computation** using Gaussian temporal decay and spatial diversity metrics
- **Verification procedures** for breadcrumbs, epochs, and TITs
- **CDDL schema** (RFC 8610) for formal data structure definitions

The I-D references established IETF standards: Ed25519 (RFC 8032) for signatures, SHA-256 (RFC 6234) for hashing, and CBOR (RFC 8949) for serialization.

ULissy serves as the reference implementation language for TrIP. The `ulissy.trajectory` and `ulissy.crypto` standard library modules directly implement the data structures and algorithms specified in the Internet-Draft.

| Component | ULissy Module | I-D Section |
|-----------|--------------|-------------|
| Breadcrumb structure | `ulissy.trajectory` | §3 Breadcrumb Specification |
| H3 spatial quantization | `ulissy.spatial` | §3.3 Spatial Quantization |
| Context digest | `ulissy.sensors` | §3.4 Context Digest Construction |
| Epoch bundling | `ulissy.trajectory` | §5 Epoch Specification |
| Trust scoring | `ulissy.trajectory` | §6 Trust Computation |
| Ed25519 signatures | `ulissy.crypto` | §2 Protocol Architecture |
| CBOR encoding | `ulissy.network` | §8 CBOR Encoding |

---

## 9. Roadmap

### Completed (v0.3.1) ✅

#### Compiler
- ✅ Formal EBNF grammar specification
- ✅ Lexer implementation in Rust (64 token types)
- ✅ Parser implementation in Rust (20 statement types)
- ✅ AST with full expression support
- ✅ `self` and `Self` keyword support
- ✅ `config` access in expressions
- ✅ `default` case in match statements
- ✅ `if` expressions
- ✅ `for` loops
- ✅ Assignment expressions
- ✅ `search` expressions (4 targets: nearby, within, identity, text)
- ✅ `if let` optional binding
- ✅ Compile-time privacy enforcement for search contexts
- ✅ Type checker with two-pass function resolution
- ✅ 7 search types: TrustScore, SearchResult, SearchResultSet, PresenceProof, SearchQuery, SpatialFilter, IdentityFilter
- ✅ Code generator (Rust emission with typed search builder pattern)
- ✅ Cargo.toml generation
- ✅ CLI tool (`ulissy` command with 8 subcommands)

#### Runtime (`gns-runtime`)
- ✅ Duration module (time spans with units)
- ✅ Moment module (instants in time)
- ✅ Distance module (lengths with units)
- ✅ Percent module (percentage values)
- ✅ Location module (GPS, H3 cells)
- ✅ Battery module (power state, modes)
- ✅ Sensors module (device sensors, context digest)
- ✅ Scheduling module (every, when, after blocks)
- ✅ Network module (publish, sync)

#### Standard Library
- ✅ `ulissy.core` - Fundamental types
- ✅ `ulissy.spatial` - Location & H3
- ✅ `ulissy.crypto` - Cryptography
- ✅ `ulissy.temporal` - Time & duration
- ✅ `ulissy.sensors` - Device sensors
- ✅ `ulissy.trajectory` - Proof-of-Trajectory
- ✅ `ulissy.messaging` - Encrypted messaging
- ✅ `ulissy.network` - Network operations
- ✅ `ulissy.prelude` - All-in-one import

#### Developer Tools
- ✅ VS Code extension with syntax highlighting
- ✅ 40+ code snippets
- ✅ Build/Check/Run commands
- ✅ Real-time diagnostics
- ✅ Custom file icon

#### Standards
- ✅ IETF Internet-Draft `draft-ayerbe-trip-protocol-00` submitted and posted
- ✅ TrIP protocol formally specified independently of ULissy
- ✅ CBOR wire format defined with CDDL schema
- ✅ Provisional Patent #63/948,788 filed

### Phase 1: Protocol Hardening (v0.4)

Priority: establish TrIP as an implementable, verifiable protocol standard.

- ⏳ **`trip-core` Rust crate** — Standalone reference implementation of the TrIP protocol (CBOR wire format, breadcrumb verification, epoch Merkle trees, TIT derivation) with zero ULissy dependency. Provides the second independent implementation required for IETF Standards Track advancement.
- ⏳ **`gns-search` Rust crate** — Runtime implementation of the `gns_search::query()` builder pattern that compiled ULissy search expressions target. Bridges ULissy search primitives to the H3 spatial index via PostgreSQL with the H3 extension.
- ⏳ **Mobile FFI bridge** — Compile ULissy→Rust output as a shared library callable from Flutter/Dart via FFI on iOS and Android. Closes the loop from language specification to working mobile application.
- ⏳ **Documentation generator** — `ulissy doc` command that extracts doc comments from `.ul` source files and emits structured Markdown or HTML documentation. Critical for protocol adoption visibility following IETF publication.

### Phase 2: Developer Experience (v0.5)

Priority: make ULissy usable by developers beyond the core team.

- ⏳ **Language Server Protocol (LSP)** — Full LSP implementation providing autocompletion, hover documentation, go-to-definition, find-references, and real-time error diagnostics. Enables ULissy integration with VS Code, Neovim, Helix, and any LSP-compatible editor.
- ⏳ **Module registry** — Protocol-aware package system where modules map to TrIP protocol extensions (e.g., `ulissy add trip.epoch-verifier`, `ulissy add trip.trust-scorer`). Positions packages as protocol components rather than generic code libraries.
- ⏳ **TrIP Search API** — REST endpoint (`GET /v1/search`) backed by the H3 spatial index, consuming GNS backend events. Client SDKs for JavaScript, Python, and Dart. See *TrIP Search Whitepaper* for full specification.

### Phase 3: Language Maturity (v1.0)

Priority: production-grade language infrastructure.

- ⏳ **Debugger integration** — Source-level debugging that maps ULissy source positions to generated Rust code, enabling breakpoints, step-through, and variable inspection in the original `.ul` source.
- ⏳ **Self-hosting** — ULissy compiler rewritten in ULissy, bootstrapping from the existing Rust implementation. Demonstrates language completeness and serves as the definitive test of the type system and code generator.
- ⏳ **IETF Standards Track** — Advance `draft-ayerbe-trip-protocol` from Informational to Standards Track with two verified independent implementations (ULissy reference + `trip-core` Rust crate) and working group adoption (target: RATS or new BOF session).

---

## 10. Conclusion

ULissy represents a new category of programming language—one designed for the physical world rather than abstract computation. By treating space, time, identity, and movement as fundamental rather than incidental, ULissy makes secure, privacy-preserving, spatially-aware applications natural to write.

With the TrIP protocol now formally specified as IETF Internet-Draft `draft-ayerbe-trip-protocol-00`, the Proof-of-Trajectory mechanism has entered the international standards pipeline—the same process that produced TCP, HTTP, TLS, and DNS. ULissy provides the developer-facing implementation of this protocol.

Combined with the GNS Protocol and TrIP Search, ULissy provides the complete stack for identity-anchored computing:

- **TrIP**: The proof protocol (IETF I-D)
- **GNS**: The naming system (identity resolution)
- **TrIP Search**: The discovery layer (spatial-identity search)
- **ULissy**: The language (implementation)
- **gns-runtime**: The runtime (execution)
- **gns-crypto-core**: The foundation (security)
- **Tauri**: The delivery (platforms)

The architecture spans from cryptographic proof to spatial discovery: TrIP proves you exist, GNS names you, TrIP Search makes you discoverable, and ULissy makes it all natural to build. The search layer — documented in the companion *TrIP Search Whitepaper* — completes the ecosystem by providing privacy-preserving discovery over the spatial-identity graph, where every result is ranked by cryptographic trust rather than advertising spend.

The era of moving machines requires a language that moves with them.

---

## Appendix A: Reserved Keywords

```
identity    let         var         const       fn          config
type        struct      enum        trait       impl        computed
if          else        match       case        guard       invariant
for         while       in          where       when        every
after       within      timeout     budget      send        to
from        as          with        return      throw       throws
async       await       import      export      public      private
internal    true        false       nil         self        Self
default     search      nearby      ranked      by
```

## Appendix B: Operators

```
// Arithmetic
+   -   *   /   %

// Comparison
==  !=  <   >   <=  >=

// Logical
&&  ||  !

// Spatial
within      near        distance    intersects

// Assignment
=   +=  -=  *=  /=

// Optional
?   ??  ?.

// Range
..  ..<

// Arrow
->  =>
```

## Appendix C: Literal Syntax

```ulissy
// Numbers
42              // Int
3.14            // Float
0xFF            // Hex
0b1010          // Binary

// Units
500.meters              // Distance
2.kilometers            // Distance
500.m                   // Distance (abbreviation)
5.km                    // Distance (abbreviation)
10.minutes              // Duration
24.hours                // Duration
1.hour                  // Duration (singular)
30.min                  // Duration (abbreviation)
80.percent

// Strings
"Hello"
"Hello, \(name)!"   // Interpolation

// Collections
[1, 2, 3]                   // Array
{ x: 10, y: 20 }            // Object literal
{ x: 10, y: 20 } as Point   // Typed object

// Handles
@alice
@microsoft

// Facets
dix@alice
home@bob/lights
pay@merchant
```

## Appendix D: Complete Statement Reference

| Statement | Syntax | Example |
|-----------|--------|---------|
| Identity | `identity name = expr` | `identity me = Keychain.primary` |
| Let | `let name = expr` | `let cell = here.h3(7)` |
| Var | `var name = expr` | `var counter = 0` |
| Const | `const NAME = expr` | `const MAX = 100` |
| Config | `config { fields }` | `config { resolution: 7 }` |
| Function | `fn name(params) -> Type { }` | `fn add(a: Int, b: Int) -> Int { }` |
| Type | `type Name { fields }` | `type Point { x: Int, y: Int }` |
| Enum | `enum Name { variants }` | `enum Status { ok, error }` |
| Computed | `computed name: Type = expr` | `computed total: Int = items.count` |
| Every | `every interval when cond { }` | `every 10.minutes { }` |
| When | `when condition { }` | `when count >= 100 { }` |
| After | `after delay { }` | `after 5.seconds { }` |
| Send | `send to recipient { }` | `send to @alice { msg: "hi" }` |
| If | `if cond { } else { }` | `if x > 0 { } else { }` |
| If Let | `if let name = optional { } else { }` | `if let peer = alice { print(peer.handle) }` |
| Match | `match expr { cases }` | `match status { case ok: { } default: { } }` |
| For | `for item in collection { }` | `for bc in trajectory { }` |
| Search | `let r = search target filters ranking` | `let r = search nearby where trust > 0.5 ranked by distance` |
| Return | `return expr` | `return result` |
| Import | `import path` | `import ulissy.spatial` |

---

**Document Version**: 0.4.2
**Last Updated**: February 2026
**Authors**: GNS Foundation
**License**: MIT

**Repository**: https://github.com/gns-protocol/ULissy_Program

---

*"The journey is the proof."*
— ULissy Design Principle
