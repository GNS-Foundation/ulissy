# GNS Runtime

Runtime support library for ULissy-generated code.

## Overview

`gns-runtime` provides the platform abstractions and scheduling primitives that ULissy programs depend on at runtime. When the ULissy compiler generates Rust code, that code calls into this library for:

- **Scheduling**: Periodic tasks (`every`), condition watching (`when`), delays (`after`)
- **Location**: GPS access, H3 cell conversion
- **Sensors**: Battery level, device sensors, context digest
- **Time**: `Moment` (instants), `Duration` (with units)
- **Network**: Epoch publishing, sync operations

## Usage

```toml
[dependencies]
gns-runtime = { path = "../gns-runtime" }
```

```rust
use gns_runtime::prelude::*;

fn main() -> RuntimeResult<()> {
    // Schedule a task every 10 minutes
    schedule_every(Duration::from_mins(10), || {
        if Battery::level() > 20 {
            let coords = Location::current()?;
            let cell = coords.to_h3(7)?;
            println!("Current cell: {}", cell);
        }
        Ok(())
    })?;

    // Watch for a condition
    watch_condition(
        || some_counter() >= 100,
        || {
            println!("Threshold reached!");
            Ok(())
        }
    )?;

    Ok(())
}
```

## Modules

| Module | Description | ULissy Syntax |
|--------|-------------|---------------|
| `duration` | Time spans with units | `10.minutes`, `2.hours` |
| `moment` | Instants in time | `now`, `crumb.timestamp` |
| `distance` | Lengths with units | `500.meters`, `2.kilometers` |
| `percent` | Percentage values | `80.percent`, `battery > 20` |
| `location` | GPS and H3 cells | `here`, `here.h3(7)` |
| `battery` | Battery level | `battery`, `battery > 20` |
| `sensors` | Device sensors | `sensors.digest` |
| `scheduling` | Task scheduling | `every`, `when`, `after` |
| `network` | Network operations | `network.publish(epoch)` |

## Features

- `tokio-runtime` (default): Use Tokio for async scheduling
- `async-std-runtime`: Use async-std instead
- `mobile`: Enable mobile-specific integrations

## Generated Code Example

ULissy:
```ulissy
every 10.minutes when battery > 20 {
    let cell = here.h3(7)
    me.trajectory.append(breadcrumb(cell: cell))
}
```

Generated Rust:
```rust
use gns_runtime::*;

schedule_every(Duration::from_mins(10), move || {
    if Battery::level() > Percent::from(20) {
        let cell = Location::current()?.to_h3(7)?;
        // ... breadcrumb logic
    }
    Ok(())
})?;
```

## Platform Support

| Platform | Status |
|----------|--------|
| macOS/Linux | ✅ Full support |
| Windows | ✅ Full support |
| iOS | 🔄 Pending CoreLocation integration |
| Android | 🔄 Pending FusedLocation integration |
| WebAssembly | 🔄 Pending |

## License

MIT - GNS Foundation
