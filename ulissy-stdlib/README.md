# ULissy Standard Library

The official standard library for the ULissy programming language.

## Modules

| Module | Description | Key Types |
|--------|-------------|-----------|
| `ulissy.core` | Fundamental types | `Int`, `Float`, `String`, `Option`, `Result` |
| `ulissy.spatial` | Location & geography | `Coordinates`, `H3Cell`, `Distance`, `here` |
| `ulissy.crypto` | Cryptographic primitives | `Identity`, `PublicKey`, `Signature`, `Hash` |
| `ulissy.temporal` | Time & duration | `Moment`, `Duration`, `now` |
| `ulissy.sensors` | Device sensors | `Battery`, `Gps`, `Sensors`, `Connectivity` |
| `ulissy.trajectory` | Proof-of-Trajectory | `Breadcrumb`, `Trajectory`, `Epoch` |
| `ulissy.messaging` | Encrypted messaging | `Message`, `Envelope`, `Conversation` |
| `ulissy.network` | Network operations | `Network`, `SyncStatus`, `GnsRecord` |
| `ulissy.prelude` | All-in-one import | Everything commonly needed |

## Usage

```ulissy
// Import specific module
import ulissy.spatial
import ulissy.crypto

// Or import everything
import ulissy.prelude

// Use types
identity me = Keychain.primary
let cell = here.h3(7)
```

## Module Details

### ulissy.core

Fundamental types available in all programs (implicitly imported):

```ulissy
// Primitive types
let x: Int = 42
let y: Float = 3.14
let s: String = "hello"
let b: Bool = true

// Optional
let maybe: Option<Int> = Option.some(42)
let value = unwrap(maybe, default: 0)

// Result
let result: Result<Int, String> = Result.ok(42)
if isOk(result) { ... }

// Collections
let arr: Array<Int> = [1, 2, 3]
let dict: Dict<String, Int> = {"a": 1, "b": 2}
```

### ulissy.spatial

Location and geographic types:

```ulissy
import ulissy.spatial

// Get current location (quantized for privacy)
let cell = here.h3(7)                    // H3 cell at resolution 7

// Distance with units
let d1 = 500.meters
let d2 = 2.kilometers
let d3 = d1 + d2                         // 2500 meters

// H3 operations
let isNeighbor = neighbors(cell1, cell2)
let dist = cellDistance(cell1, cell2)
let parent = parent(cell, resolution: 5)

// Check reachability (for breadcrumb validation)
let canReach = reachable(from: cell1, to: cell2, within: 10.minutes)
```

### ulissy.crypto

Cryptographic operations:

```ulissy
import ulissy.crypto

// Identity from keychain
identity me = Keychain.primary
identity work = Keychain.facet("microsoft")

// Signing
let sig = me.sign(data)
let valid = sig.verify(data, me.publicKey)

// Hashing
let hash = sha256(data)
let hash2 = sha256String("hello")

// Encryption (with forward secrecy)
let (ephPrivate, ephPublic) = ephemeralKeyPair()
let secret = keyExchange(myPrivate, theirPublic)
let encrypted = encrypt(plaintext, secret)
let decrypted = decrypt(encrypted, secret)

// Random
let bytes = randomBytes(32)
let id = uuid()
```

### ulissy.temporal

Time and duration:

```ulissy
import ulissy.temporal

// Current time
let timestamp = now

// Duration with units
let d1 = 10.minutes
let d2 = 2.hours
let d3 = 7.days

// Moment arithmetic
let future = now.plus(1.hours)
let past = now.minus(30.minutes)
let duration = future.since(past)

// Comparisons
if now.isAfter(deadline) { ... }
if event.isBefore(now) { ... }

// Date components
let year = timestamp.year
let month = timestamp.month
let isToday = isToday(event)
```

### ulissy.sensors

Device sensors and system state:

```ulissy
import ulissy.sensors

// Battery
let level = battery                      // Percent
if battery > 20 { ... }
let state = Battery.state()
if state.isCharging { ... }

// GPS
if gps.available() {
    let coords = Location.current()
}

// Connectivity
if connectivity.available() {
    if connectivity.isWifi() { ... }
}

// Sensor context (for breadcrumbs)
let ctx = sensors.current()
let digest = sensors.digest()            // Hash of sensor data
```

### ulissy.trajectory

Proof-of-Trajectory for GNS:

```ulissy
import ulissy.trajectory

// Create breadcrumb
let crumb = breadcrumb(
    cell: here.h3(7),
    source: .gps,
    context: sensors.digest(),
    previous: me.trajectory.lastHash
).signed(me)

// Add to trajectory
me.trajectory.append(crumb)

// Check status
let count = me.trajectory.count
let pending = me.trajectory.pending
let ready = me.trajectory.meetsHandleThreshold

// Bundle epoch
when me.trajectory.pending >= 100 {
    let epoch = me.trajectory.bundleEpoch()
    let signedEpoch = epoch.signed(me)
    network.publish(signedEpoch)
}

// Validation
let valid = validateTrajectory(trajectory)
let epochValid = validateEpoch(epoch)
```

### ulissy.messaging

Encrypted messaging:

```ulissy
import ulissy.messaging

// Send encrypted message
send to @alice {
    type: "greeting",
    message: "Hello!"
}

// Create envelope manually
let msg = textMessage("Hello!")
let envelope = seal(msg, from: me, to: aliceKey)

// Open received envelope
let message = open(envelope, recipient: me)

// Facets
dix@me.post("Hello world!", visibility: .public)
home@me/lights.set(brightness: 80.percent)
pay@merchant.request(50.USD)
email@alice.send(subject: "Meeting", body: content)
```

### ulissy.network

Network operations:

```ulissy
import ulissy.network

// Check connectivity
if network.available() {
    // Publish epoch
    let result = network.publish(epoch)
    if result.success {
        print("Published: \(result.itemId)")
    }
    
    // Sync data
    let status = network.sync()
}

// GNS lookups
let record = lookupHandle(@alice)
let key = resolveHandle(@alice)
let handle = resolveKey(publicKey)

// Relay channel
let relay = relay("wss://relay.gns.foundation/ws")
relay.connect()
relay.subscribe(me) { envelope in
    let msg = open(envelope, recipient: me)
    // Handle message
}
```

## File Structure

```
ulissy-stdlib/
├── README.md
└── src/
    ├── core.ul        # Fundamental types
    ├── spatial.ul     # Location & H3
    ├── crypto.ul      # Cryptography
    ├── temporal.ul    # Time & duration
    ├── sensors.ul     # Device sensors
    ├── trajectory.ul  # Proof-of-Trajectory
    ├── messaging.ul   # Encrypted messaging
    ├── network.ul     # Network operations
    └── prelude.ul     # All-in-one import
```

## Implementation Notes

The standard library defines ULissy interfaces that map to:

1. **gns-runtime** (Rust) - Runtime implementations
2. **gns-crypto-core** (Rust) - Cryptographic operations
3. **Platform APIs** - iOS CoreLocation, Android FusedLocation, etc.

Functions marked with `builtin.` are implemented in the runtime and linked during compilation.

## License

MIT - GNS Foundation
