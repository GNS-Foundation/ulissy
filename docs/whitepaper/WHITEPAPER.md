# ULissy
## A Programming Language for Moving Machines

**WHITEPAPER**
*Technical Edition for Language Designers & Protocol Engineers*
*2025 Draft — Version 0.1.0*

**ULISSY Foundation / GNS Protocol**

---

## Abstract

ULissy is a domain-specific programming language designed for machines that move through physical space—mobile phones, vehicles, drones, wearables, and IoT devices. By treating **identity**, **location**, **time**, **cryptography**, and **energy** as first-class language primitives rather than library imports, ULissy enables developers to write secure, efficient, spatially-aware applications with compile-time safety guarantees.

ULissy transpiles to Rust and integrates natively with the GNS Protocol (Geospatial Naming System), providing a unified development experience from source code to deployed application across iOS, Android, desktop, and WebAssembly targets.

The name honors Ulysses (Odysseus)—the mythological figure who proved his identity not through credentials, but through his journey. In ULissy, as in the Odyssey, **trajectory is identity**.

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

### 2.1 A First ULissy Program

```ulissy
// Collect breadcrumbs to prove humanity
identity me = Keychain.primary

every 10.minutes when battery > 20% {
    let crumb = breadcrumb(
        cell: here.h3(10),
        context: sensors.digest,
        previous: me.trajectory.last
    ).signed(me)
    
    me.trajectory.append(crumb)
}

when me.trajectory.count >= 100 {
    print("Ready to claim @handle!")
}
```

This program:
- Retrieves the user's cryptographic identity from secure storage
- Every 10 minutes (if battery permits), records an H3-quantized location
- Chains each breadcrumb cryptographically to the previous
- Notifies when proof-of-humanity threshold is reached

The equivalent in Swift + CoreLocation + CryptoKit would be 200+ lines with manual coordination between frameworks.

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
breadcrumbs.ul
messaging.ul
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

### 4.2 Facet Addressing

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

### 4.3 Temporal Constructs

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

// Timeout
within 10.seconds {
    let response = await server.request()
} timeout {
    showOfflineMode()
}
```

### 4.4 Spatial Constructs

```ulissy
// Current location (quantized)
let cell = here.h3(10)          // Resolution 10 ≈ 15,000 m²
let cell = here.h3(12)          // Resolution 12 ≈ 300 m²

// Geofencing
when here within region {
    notify("Welcome!")
}

when here exits homeZone {
    arm(securitySystem)
}

// Distance
let d = distance(from: here, to: destination)
if d < 100.meters {
    notify("Arriving soon")
}

// Spatial queries
let nearby = friends.filter { distance(to: $0) < 1.kilometer }
```

### 4.5 Cryptographic Constructs

Encryption is implicit and correct by construction:

```ulissy
// Sending encrypted message (X25519 + ChaCha20 + Ed25519 automatic)
send to @alice {
    message: "Meeting at 3pm"
    attachment: document
}

// Signing data
let signed = data.signed(me)

// Verifying
if crumb.signature.valid(for: crumb, by: author) {
    accept(crumb)
}

// Envelope is automatic
// Developer never sees: key exchange, nonce generation, AEAD, etc.
```

### 4.6 Energy-Aware Constructs

```ulissy
// Conditional on battery
when battery < 20% {
    reduce(locationAccuracy: .low)
    pause(backgroundSync)
}

// Power mode blocks
with powerMode: .performance {
    // High-accuracy operations here
    let precise = here.h3(15)
}

// Energy budget
budget 5% battery {
    performSync()
} exceeded {
    deferSync()
}
```

### 4.7 Control Flow

```ulissy
// Conditionals
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// Pattern matching
match facet {
    case .dix(let handle):
        showBroadcast(handle)
    case .home(let handle, let device):
        controlDevice(device)
    case .pay(let handle):
        showPayment(handle)
}

// Optional handling
if let handle = me.handle {
    greet(handle)
} else {
    promptHandleClaim()
}

// Guard
guard me.trajectory.count >= 100 else {
    return Error.insufficientBreadcrumbs
}
```

### 4.8 Functions

```ulissy
// Function declaration
fn greet(name: String) -> String {
    return "Hello, \(name)!"
}

// With constraints
fn claimHandle(name: String, for identity: Identity) -> Handle
    where identity.breadcrumbs >= 100
{
    // ...
}

// Async functions
async fn fetchProfile(handle: Handle) -> Profile? {
    let record = await gns.resolve(handle)
    return record?.profile
}

// Throwing functions
fn verify(crumb: Breadcrumb) throws -> Bool {
    guard crumb.signature.valid else {
        throw VerificationError.invalidSignature
    }
    return true
}
```

---

## 5. Standard Library

### 5.1 Core Modules

```ulissy
import ulissy.identity      // Keychain, Identity, Handle
import ulissy.spatial       // H3Cell, Distance, here, within
import ulissy.temporal      // Moment, Duration, every, after
import ulissy.crypto        // Signing, encryption, hashing
import ulissy.energy        // Battery, PowerMode
import ulissy.connectivity  // Online, offline, mesh states
```

### 5.2 GNS Protocol Modules

```ulissy
import gns.breadcrumb       // Breadcrumb, Trajectory, Chain
import gns.facets.dix       // Broadcasting
import gns.facets.home      // IoT control
import gns.facets.pay       // Payments, IDUP
import gns.facets.email     // Messaging
import gns.facets.car       // Vehicle control
import gns.record           // GnsRecord, modules
import gns.geoauth          // Location-bound authentication
```

### 5.3 Platform Modules

```ulissy
import platform.sensors     // Accelerometer, gyroscope, compass
import platform.keychain    // Secure enclave access
import platform.network     // HTTP, WebSocket
import platform.storage     // Encrypted local storage
import platform.ui          // Native UI bindings (Tauri)
```

---

## 6. Compilation Model

### 6.1 Transpilation to Rust

ULissy transpiles to Rust, then compiles via `rustc`:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│              │     │              │     │              │
│  source.ul   │────▶│  output.rs   │────▶│   binary     │
│              │     │              │     │              │
│   (ULissy)   │     │    (Rust)    │     │  (native/    │
│              │     │              │     │    wasm)     │
└──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │
       │                    │                    │
   ulissy compile       rustc/cargo         executable
```

### 6.2 Example Transpilation

ULissy source:

```ulissy
identity me = Keychain.primary

let crumb = breadcrumb(
    cell: here.h3(10),
    context: sensors.digest,
    previous: me.trajectory.last
).signed(me)
```

Transpiled Rust:

```rust
use gns_crypto_core::{Identity, Breadcrumb, H3Cell, Sensors};
use gns_crypto_core::keychain::Keychain;

fn main() -> Result<(), gns_crypto_core::Error> {
    let me = Identity::from_keychain(Keychain::primary()?)?;
    
    let cell = H3Cell::from_current_location(10)?;
    let context = Sensors::current_digest()?;
    let previous_hash = me.trajectory().last_hash()?;
    
    let crumb = Breadcrumb::builder()
        .cell(cell)
        .context(context)
        .previous(previous_hash)
        .build()?
        .sign(&me)?;
    
    Ok(())
}
```

### 6.3 Compilation Targets

| Target | Output | Use Case |
|--------|--------|----------|
| `ulissy build --target ios` | .ipa via Tauri | iPhone/iPad apps |
| `ulissy build --target android` | .apk via Tauri | Android apps |
| `ulissy build --target macos` | .dmg via Tauri | macOS apps |
| `ulissy build --target windows` | .exe via Tauri | Windows apps |
| `ulissy build --target linux` | binary via Tauri | Linux apps |
| `ulissy build --target wasm` | .wasm | Panthera, web |
| `ulissy build --target lib` | Rust crate | Library distribution |

### 6.4 Integration with gns-crypto-core

ULissy depends on gns-crypto-core as its cryptographic foundation:

```
┌─────────────────────────────────────────────────────────────┐
│                     ULissy Compiler                         │
└──────────────────────────┬──────────────────────────────────┘
                           │ generates calls to
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    gns-crypto-core                          │
│                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ ed25519-    │ │ x25519-     │ │ chacha20    │           │
│  │ dalek       │ │ dalek       │ │ poly1305    │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │    h3o      │ │    hkdf     │ │    sha2     │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Compiler Architecture

### 7.1 Pipeline Stages

```
Source Code (.ul)
      │
      ▼
┌─────────────┐
│   LEXER     │  Tokenization: source → tokens
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   PARSER    │  Syntax analysis: tokens → AST
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   TYPE      │  Type checking: AST → typed AST
│  CHECKER    │  Constraint validation
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   CODE      │  Code generation: typed AST → Rust
│ GENERATOR   │
└──────┬──────┘
       │
       ▼
  Rust Code (.rs)
      │
      ▼
┌─────────────┐
│   RUSTC     │  Native compilation
│  + CARGO    │
└──────┬──────┘
       │
       ▼
  Executable
```

### 7.2 Error Messages

ULissy provides helpful, domain-aware error messages:

```
error[UL0042]: Breadcrumb chain integrity violation
  --> src/main.ul:14:5
   |
14 |     me.trajectory.append(crumb)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: breadcrumb.previous does not match trajectory.last.hash
   = help: ensure breadcrumb was created with `previous: me.trajectory.last`

error[UL0107]: Privacy violation - raw coordinates exposure
  --> src/sync.ul:8:12
   |
 8 |     send(location, to: server)
   |          ^^^^^^^^
   |
   = note: type `Coordinates` cannot be transmitted
   = help: quantize with `.h3(resolution)` before sending:
   |
 8 |     send(location.h3(10), to: server)
   |          ~~~~~~~~~~~~~~~~

error[UL0203]: Insufficient breadcrumbs for handle claim
  --> src/profile.ul:22:5
   |
22 |     let handle = claimHandle("alice", for: me)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: handle claim requires >= 100 breadcrumbs
   = note: current trajectory has 47 breadcrumbs
   = help: continue collecting breadcrumbs for approximately 9 more hours
```

---

## 8. Tooling

### 8.1 Command Line Interface

```bash
# Create new project
ulissy new my-app

# Compile
ulissy build
ulissy build --target ios
ulissy build --release

# Run
ulissy run

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
```

### 8.2 Project Structure

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

ULissy makes GNS protocol compliance automatic. A program that compiles is a program that follows the protocol.

---

## 11. Implementation Roadmap

### Phase 1: Bootstrap (Weeks 1-4)

- [ ] Formal EBNF grammar specification
- [ ] Lexer implementation in Rust
- [ ] Parser implementation in Rust
- [ ] Basic AST types

### Phase 2: Type System (Weeks 5-8)

- [ ] Primitive type implementations
- [ ] Type checker
- [ ] Constraint validation
- [ ] Error message system

### Phase 3: Code Generation (Weeks 9-12)

- [ ] Rust code emitter
- [ ] gns-crypto-core integration
- [ ] Cargo.toml generation
- [ ] First compiling programs

### Phase 4: Standard Library (Weeks 13-16)

- [ ] Core modules (identity, spatial, temporal)
- [ ] GNS protocol modules
- [ ] Platform abstraction layer

### Phase 5: Tooling (Weeks 17-20)

- [ ] CLI tool (ulissy command)
- [ ] VS Code extension
- [ ] Documentation generator
- [ ] Package manager design

### Phase 6: Self-Hosting (Weeks 21-24)

- [ ] Rewrite compiler in ULissy
- [ ] Bootstrap from Rust implementation
- [ ] Language becomes self-sufficient

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
2025                    2026                    2027+
  │                       │                       │
  ▼                       ▼                       ▼

ULissy 0.1            ULissy 1.0            ULissy Ecosystem
- Basic compiler      - Production ready    - Package registry
- Core types          - Full GNS support    - Framework ecosystem
- CLI tool            - IDE support         - Enterprise adoption
                      - Self-hosting        - Industry standard
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
identity    let         var         const       fn
type        struct      enum        trait       impl
if          else        match       case        guard
for         while       in          where       when
every       after       within      timeout     budget
send        to          from        as          with
return      throw       throws      async       await
import      export      public      private     internal
true        false       nil         self        Self
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
"Hello, \(name)!"

// Collections
[1, 2, 3]                   // Array
["a": 1, "b": 2]            // Dictionary
{1, 2, 3}                   // Set

// Handles
@alice
@microsoft

// Facets
dix@alice
home@bob/lights
pay@merchant
```

---

**Document Version**: 0.1.0-draft
**Last Updated**: January 2025
**Authors**: GNS Foundation
**License**: MIT

---

*"The journey is the proof."*
— ULissy Design Principle
