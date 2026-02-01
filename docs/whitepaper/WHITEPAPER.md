# ULissy
## A Programming Language for Moving Machines

**WHITEPAPER**
*Technical Edition for Language Designers & Protocol Engineers*
*2025 — Version 0.3.1*

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
| **Lexer** | ✅ Complete | 64 token types, handles, facets, interpolation, `default` keyword |
| **Parser** | ✅ Complete | 18 statement types, full expression parsing, `self`/`Self`/`config` support |
| **Type Checker** | ✅ Complete | Domain-specific types, inference, validation, two-pass function resolution |
| **Code Generator** | ✅ Complete | Rust emission, Cargo.toml generation |
| **CLI** | ✅ Complete | `ulissy build`, `check`, `run`, `new`, `lex`, `parse`, `fmt`, `info` |
| **Runtime** | ✅ Complete | `gns-runtime` crate with scheduling, sensors, location |
| **Standard Library** | ✅ Complete | 9 modules: core, spatial, crypto, temporal, sensors, trajectory, messaging, network, prelude |
| **VS Code Extension** | ✅ Complete | Syntax highlighting, 40+ snippets, commands, diagnostics |

---

## Changelog

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

## 8. Roadmap

### Completed (v0.3.1) ✅

#### Compiler
- ✅ Formal EBNF grammar specification
- ✅ Lexer implementation in Rust (64 token types)
- ✅ Parser implementation in Rust (18 statement types)
- ✅ AST with full expression support
- ✅ `self` and `Self` keyword support
- ✅ `config` access in expressions
- ✅ `default` case in match statements
- ✅ `if` expressions
- ✅ `for` loops
- ✅ Assignment expressions
- ✅ Type checker with two-pass function resolution
- ✅ Code generator (Rust emission)
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

### Planned (v1.0)

- ⏳ Language Server Protocol (LSP) - full IDE integration
- ⏳ Go to definition, find references
- ⏳ Debugger integration
- ⏳ Documentation generator
- ⏳ Package manager
- ⏳ Mobile platform integration (iOS/Android)
- ⏳ Self-hosting (compiler written in ULissy)

---

## 9. Conclusion

ULissy represents a new category of programming language—one designed for the physical world rather than abstract computation. By treating space, time, identity, and movement as fundamental rather than incidental, ULissy makes secure, privacy-preserving, spatially-aware applications natural to write.

Combined with the GNS Protocol, ULissy provides the complete stack for identity-anchored computing:

- **GNS**: The protocol (what)
- **ULissy**: The language (how)
- **gns-runtime**: The runtime (execution)
- **gns-crypto-core**: The foundation (security)
- **Tauri**: The delivery (platforms)

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
default
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
500.meters
2.kilometers
10.minutes
24.hours
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
| Match | `match expr { cases }` | `match status { case ok: { } default: { } }` |
| For | `for item in collection { }` | `for bc in trajectory { }` |
| Return | `return expr` | `return result` |
| Import | `import path` | `import ulissy.spatial` |

---

**Document Version**: 0.3.1
**Last Updated**: January 2025
**Authors**: GNS Foundation
**License**: MIT

**Repository**: https://github.com/gns-protocol/ULissy_Program

---

*"The journey is the proof."*
— ULissy Design Principle
