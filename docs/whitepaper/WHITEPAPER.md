# ULissy
## A Programming Language for Moving Machines

**WHITEPAPER**
*Technical Edition for Language Designers & Protocol Engineers*
*2025 — Version 0.2.0*

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
| **Lexer** | ✅ Complete | 63 token types, handles, facets, interpolation |
| **Parser** | ✅ Complete | 17 statement types, full expression parsing |
| **Type Checker** | ✅ Complete | Domain-specific types, inference, validation |
| **Code Generator** | ✅ Complete | Rust emission, Cargo.toml generation |
| **CLI** | ✅ Complete | `ulissy build`, `check`, `run`, `new`, `lex`, `parse`, `fmt`, `info` |
| **Standard Library** | 🔄 In Progress | Core modules |

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

import ulissy.spatial
import ulissy.crypto

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

### 4.2 Config Blocks (NEW in v0.2)

Module-level configuration with compile-time constants:

```ulissy
config {
    resolution: 7,
    interval: 10.minutes,
    minBreadcrumbsPerEpoch: 100,
    requireGps: true
}

// Access via config.fieldName
every config.interval when battery > 20 {
    let cell = here.h3(config.resolution)
}
```

**Generated Rust:**
```rust
mod config {
    pub const RESOLUTION: i64 = 7;
    pub const INTERVAL: Duration = Duration::from_mins(10);
    pub const MIN_BREADCRUMBS_PER_EPOCH: i64 = 100;
    pub const REQUIRE_GPS: bool = true;
}
```

### 4.3 Computed Properties (NEW in v0.2)

Standalone reactive properties that auto-update:

```ulissy
// Expression form
computed total: Int = items.count

// Object form - constructs a type
computed status: CollectionStatus {
    isActive: collection.running,
    totalCount: me.trajectory.count,
    pendingCount: me.trajectory.pending,
    epochCount: me.trajectory.epochs.count
}
```

**Generated Rust:**
```rust
fn total() -> i64 {
    items.len()
}

fn status() -> CollectionStatus {
    CollectionStatus {
        is_active: collection.running(),
        total_count: me.trajectory().len(),
        pending_count: me.trajectory().pending(),
        epoch_count: me.trajectory().epochs().len(),
    }
}
```

### 4.4 Facet Addressing

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

### 4.5 Temporal Constructs

```ulissy
// Periodic execution
every 10.minutes {
    collectBreadcrumb()
}

// Conditional periodic
every 30.seconds when battery > 20% and connectivity.available {
    sync()
}

// Delayed execution
after 5.seconds {
    dismissNotification()
}

// Condition trigger (reactive)
when me.trajectory.count >= 100 {
    notifyHandleReady()
}
```

### 4.6 Optional Chaining and Nil Coalescing

Safe navigation for optional values:

```ulissy
// Optional chaining - returns nil if any link is nil
let lastCell = me.trajectory.last?.cell

// Nil coalescing - provide default value
let hash = me.trajectory.last?.hash ?? "genesis"

// Combined in real usage
let crumb = breadcrumb(
    cell: here.h3(7),
    previous: me.trajectory.last?.hash ?? "genesis"
).signed(me)
```

### 4.7 Functions

```ulissy
// Basic function
fn collectBreadcrumb() -> Breadcrumb {
    let cell = here.h3(config.resolution)
    return breadcrumb(cell: cell, timestamp: now).signed(me)
}

// With parameters and defaults
fn collectNow(source: LocationSource = .gps) -> Breadcrumb? {
    if !gps.available && source == .gps {
        return nil
    }
    return breadcrumb(cell: here.h3(7), source: source).signed(me)
}

// Async function
async fn syncWithNetwork() -> Result<SyncStatus, NetworkError> {
    let response = await network.sync(me.trajectory)
    return response.status
}

// With constraints
fn distance(from a: H3Cell, to b: H3Cell) -> Distance
    where a.resolution == b.resolution
{
    return a.distanceTo(b)
}
```

### 4.8 Enums with Associated Types

```ulissy
// Simple enum
enum LocationSource { gps, wifi, cell, ip, manual }

// Generic enum
enum Result<T, E> {
    ok(T),
    error(E)
}

enum Option<T> {
    some(T),
    none
}

// Pattern matching
match result {
    case ok(value): {
        process(value)
    }
    case error(e): {
        log("Error: \(e)")
    }
}
```

### 4.9 Send Statements

Encrypted messaging as a language construct:

```ulissy
// Direct message (automatically encrypted)
send to @alice {
    type: "greeting",
    message: "Hello!"
}

// With facet routing
send to chat@team {
    content: "Meeting in 5",
    priority: .high
}

// Notification on epoch publish
send to dix@me {
    type: "epoch_published",
    epochId: epoch.id,
    breadcrumbCount: epoch.breadcrumbs.count
}
```

### 4.10 Import Statements

```ulissy
import ulissy.spatial
import ulissy.crypto
import ulissy.messaging as msg

// Use imported modules
let cell = spatial.h3(here, resolution: 7)
let sig = crypto.sign(data, with: me)
msg.send(to: @alice, content: "Hello")
```

---

## 5. Compiler Architecture

### 5.1 Pipeline

```
Source Code (.ul)
       │
       ▼
┌─────────────┐
│   LEXER     │  → 63 token types
│             │  → Handles, facets, interpolation
└─────────────┘
       │
       ▼
┌─────────────┐
│   PARSER    │  → 17 statement types
│             │  → Full expression precedence
└─────────────┘
       │
       ▼
┌─────────────┐
│ TYPE CHECK  │  → Domain-specific types
│             │  → Invariant validation
└─────────────┘
       │
       ▼
┌─────────────┐
│  CODEGEN    │  → Rust source code
│             │  → Cargo.toml
└─────────────┘
       │
       ▼
Generated Rust Project
       │
       ▼
   cargo build
       │
       ▼
Native Binary / WASM
```

### 5.2 Lexer Tokens

The lexer recognizes 63 token types:

**Keywords (33):**
```
identity  let       var       const     fn        config
type      struct    enum      trait     impl      computed
if        else      match     case      guard     invariant
for       while     in        where     when      every
after     within    timeout   budget    send      to
from      as        with      return    throw     throws
async     await     import    export    public    private
internal  true      false     nil       self      Self
```

**Operators (22):**
```
+   -   *   /   %   =   ==  !=  <   >   <=  >=
&&  ||  !   +=  -=  *=  /=  ?   ??  ?.  ..  ..<
->  =>
```

**Special:**
```
@handle           // @alice
facet@handle      // dix@alice
facet@handle/path // home@alice/lights
10.minutes        // Unit values
"Hello, \(name)!" // Interpolated strings
```

### 5.3 AST Node Types

**Statements (17):**
- `IdentityDecl` - identity declaration
- `LetDecl` - immutable binding
- `VarDecl` - mutable binding
- `ConstDecl` - constant declaration
- `FnDecl` - function declaration
- `TypeDecl` - type definition
- `EnumDecl` - enum definition
- `ConfigBlock` - module configuration
- `ComputedPropertyDecl` - computed property
- `EveryBlock` - periodic execution
- `WhenBlock` - conditional trigger
- `AfterBlock` - delayed execution
- `SendStatement` - encrypted messaging
- `IfStatement` - conditional
- `MatchStatement` - pattern matching
- `ReturnStatement` - return value
- `ImportStatement` - module import

**Expressions (20+):**
- Literals (Int, Float, String, Bool, Nil)
- Identifiers, Handles, FacetAddresses
- Binary, Unary operations
- Member access, Optional member (`?.`)
- Method calls, Optional method calls
- Nil coalescing (`??`)
- Object literals, Arrays
- Interpolated strings
- Unit values

---

## 6. Code Generation

### 6.1 Rust Emission

ULissy generates idiomatic Rust code:

**ULissy:**
```ulissy
identity me = Keychain.primary

every 10.minutes when battery > 20 {
    let crumb = breadcrumb(
        cell: here.h3(7),
        previous: me.trajectory.last?.hash ?? "genesis"
    ).signed(me)
    
    me.trajectory.append(crumb)
}
```

**Generated Rust:**
```rust
// AUTO-GENERATED BY ULISSY COMPILER
// Do not edit manually

use gns_crypto_core::*;

fn main() -> Result<(), GnsError> {
    let me = Keychain::primary()?;

    // ULissy: every block - scheduled task
    gns_runtime::schedule_every(Duration::from_mins(10), move || {
        if Battery::level() > Percent::from(20) {
            let crumb = Breadcrumb::new(
                Location::current().to_h3(7),
                me.trajectory()
                    .last_hash()
                    .map(|b| b.hash.clone())
                    .unwrap_or_else(|| "genesis".to_string()),
            ).sign(&me)?;
            
            me.trajectory().append(crumb)?;
        }
        Ok(())
    })?;

    Ok(())
}
```

### 6.2 Type Mapping

| ULissy Type | Rust Type |
|-------------|-----------|
| `Int` | `i64` |
| `Float` | `f64` |
| `Bool` | `bool` |
| `String` | `String` |
| `Identity` | `Identity` |
| `PublicKey` | `PublicKey` |
| `H3Cell` | `H3Cell` |
| `Duration` | `Duration` |
| `Moment` | `Moment` |
| `Hash` | `Hash` |
| `Breadcrumb` | `Breadcrumb` |
| `Trajectory` | `Trajectory` |
| `Array<T>` | `Vec<T>` |
| `T?` | `Option<T>` |

### 6.3 Generated Cargo.toml

```toml
[package]
name = "my-ulissy-app"
version = "0.1.0"
edition = "2021"

# AUTO-GENERATED BY ULISSY COMPILER
# Do not edit manually

[dependencies]
gns-crypto-core = { path = "../../gns-crypto-core" }
tokio = { version = "1", features = ["full"] }
thiserror = "1.0"

# GNS Protocol dependencies
ed25519-dalek = "2"
x25519-dalek = "2"
chacha20poly1305 = "0.10"
h3o = "0.6"
sha2 = "0.10"
hkdf = "0.12"
```

---

## 7. Error Messages

ULissy provides domain-aware error messages:

### 7.1 Type Errors

```
error[E0201]: type mismatch
  --> trajectory.ul:15:12
   |
15 |     cell: "hello",
   |           ^^^^^^^ expected H3Cell, found String
   |
   = note: use here.h3(resolution) to get an H3Cell
```

### 7.2 Privacy Errors

```
error[E0301]: privacy violation
  --> tracker.ul:8:5
   |
 8 |     send(gps.current, to: server)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = error: cannot transmit raw Coordinates
   = help: quantize with .h3(resolution) first
   |
 8 |     send(gps.current.h3(10), to: server)
   |                     +++++++
```

### 7.3 Cryptographic Errors

```
error[E0401]: unsigned data
  --> messaging.ul:12:5
   |
12 |     me.trajectory.append(crumb)
   |                          ^^^^^ Breadcrumb requires signature
   |
   = help: sign the breadcrumb before appending
   |
12 |     me.trajectory.append(crumb.signed(me))
   |                              ++++++++++++
```

### 7.4 Protocol Errors

```
error[E0501]: insufficient breadcrumbs
  --> claim.ul:5:1
   |
 5 | claim @myhandle
   | ^^^^^^^^^^^^^^^
   |
   = error: handle registration requires 100 breadcrumbs
   = note: current trajectory has 47 breadcrumbs
   = help: continue collecting breadcrumbs for approximately 9 more hours
```

---

## 8. Tooling

### 8.1 Command Line Interface

The `ulissy` CLI provides a complete development workflow:

```bash
# Create new project
ulissy new my-app

# Compile to Rust
ulissy build src/main.ul
ulissy build src/main.ul --output target/rust --name my-app

# Type check without compiling
ulissy check src/main.ul

# Compile and run
ulissy run src/main.ul

# Debug: show lexer tokens
ulissy lex src/main.ul

# Debug: show parsed AST
ulissy parse src/main.ul

# Format source code
ulissy fmt src/main.ul

# Show compiler info
ulissy info
```

**Installation:**

```bash
cd ULissy_Program/compiler/ulissy
cargo build --release

# Install globally
cargo install --path .

# Now available system-wide
ulissy --help
```

**Example output:**

```
╭─────────────────────────────────────╮
│  ULissy Compiler v0.2.0             │
│  A Language for Moving Machines     │
╰─────────────────────────────────────╯

✓ Parsed 47 tokens
✓ Built AST with 12 statements  
✓ Type check passed
✓ Generated Rust code

Output: target/ulissy/src/main.rs
        target/ulissy/Cargo.toml
```

### 8.2 Project Structure

```
my-app/
├── ulissy.toml          # Project configuration
├── src/
│   ├── main.ul          # Entry point
│   ├── trajectory.ul
│   ├── messaging.ul
│   └── payments.ul
├── tests/
│   └── integration.ul
├── assets/              # Icons, images
└── target/              # Build output
    └── rust/            # Generated Rust code
```

### 8.3 Configuration (ulissy.toml)

```toml
[package]
name = "my-app"
version = "0.1.0"
authors = ["Camilo <camilo@gns.foundation>"]

[dependencies]
gns-crypto-core = "1.0"

[targets]
default = ["ios", "android"]

[identity]
minimum-breadcrumbs = 100
h3-resolution = 10
collection-interval = "10m"

[build]
optimize = "size"  # or "speed"
```

### 8.4 IDE Support

- **VS Code Extension**: Syntax highlighting, autocomplete, error diagnostics
- **Language Server Protocol (LSP)**: Editor-agnostic support
- **Inline Documentation**: Hover for type information and docs

---

## 9. Security Model

### 9.1 Compile-Time Guarantees

The type system prevents entire classes of vulnerabilities:

| Vulnerability | Prevention |
|---------------|------------|
| Unsigned data | `Breadcrumb` type requires `signature` field |
| Privacy leaks | `Coordinates` type cannot be serialized/transmitted |
| Key exposure | `PrivateKey` cannot leave secure enclave |
| Replay attacks | `Nonce` type enforced in `Envelope` |
| Chain breaks | `previous` hash validated at compile time |

### 9.2 Runtime Guarantees

- All cryptographic operations use gns-crypto-core (audited Rust)
- Private keys bound to hardware secure enclave
- No plaintext sensitive data in memory longer than necessary
- Automatic zeroization of secrets

### 9.3 What ULissy Cannot Prevent

- Physical device compromise
- Side-channel attacks on hardware
- Social engineering
- Bugs in gns-crypto-core itself

---

## 10. Relationship to GNS Protocol

ULissy is the **native language** for GNS Protocol development:

| GNS Concept | ULissy Construct |
|-------------|------------------|
| Ed25519 identity | `identity` declaration |
| Breadcrumb chain | `Trajectory` type, `breadcrumb()` function |
| H3 quantization | `here.h3(resolution)` |
| Protocol facets | `dix@`, `home@`, `pay@` syntax |
| Organization facets | `org@` prefix |
| GeoAuth | `geoauth` block |
| Encrypted envelope | Implicit in `send to` |
| GnsRecord | `record` type |
| IDUP payments | `pay@` facet operations |
| Module config | `config { }` block |

ULissy makes GNS protocol compliance automatic. A program that compiles is a program that follows the protocol.

---

## 11. Implementation Status

### Completed (v0.2.0)

- ✅ Formal EBNF grammar specification
- ✅ Lexer implementation in Rust (63 token types)
- ✅ Parser implementation in Rust (17 statement types)
- ✅ AST with full expression support
- ✅ Type checker with domain-specific types
- ✅ Code generator (Rust emission)
- ✅ `config { }` blocks
- ✅ Standalone `computed` properties
- ✅ Optional chaining (`?.`) and nil coalescing (`??`)
- ✅ Interpolated strings
- ✅ Cargo.toml generation
- ✅ CLI tool (`ulissy` command with 8 subcommands)

### In Progress

- 🔄 Standard library modules
- 🔄 gns-runtime crate
- 🔄 VS Code extension

### Planned

- ⏳ Documentation generator
- ⏳ Package manager
- ⏳ Self-hosting (compiler written in ULissy)

---

## 12. Future Directions

### 12.1 Potential Extensions

- **Formal verification**: Prove protocol compliance mathematically
- **GPU support**: Parallel spatial computations
- **Mesh networking**: First-class peer-to-peer constructs
- **Machine learning**: On-device inference primitives
- **Robotics**: Motor control, sensor fusion

### 12.2 Ecosystem Vision

```
2025 Q1 (NOW)         2025 Q2               2025 Q3+
  │                     │                     │
  ▼                     ▼                     ▼

ULissy 0.2 ✅         ULissy 1.0            ULissy Ecosystem
- Compiler complete   - Production ready    - Package registry
- CLI tool complete   - Full GNS support    - Framework ecosystem
- Code generation     - gns-runtime         - Enterprise adoption
- Type system         - VS Code extension   - Industry standard
                      - Standard library    - Self-hosting
```

---

## 13. Conclusion

ULissy represents a new category of programming language—one designed for the physical world rather than abstract computation. By treating space, time, identity, and movement as fundamental rather than incidental, ULissy makes secure, privacy-preserving, spatially-aware applications natural to write.

Combined with the GNS Protocol, ULissy provides the complete stack for identity-anchored computing:

- **GNS**: The protocol (what)
- **ULissy**: The language (how)
- **gns-crypto-core**: The foundation (implementation)
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
| Match | `match expr { cases }` | `match status { case ok: { } }` |
| Return | `return expr` | `return result` |
| Import | `import path` | `import ulissy.spatial` |

---

**Document Version**: 0.2.0
**Last Updated**: January 2025
**Authors**: GNS Foundation
**License**: MIT

**Repository**: https://github.com/gns-protocol/ULissy_Program

---

*"The journey is the proof."*
— ULissy Design Principle
