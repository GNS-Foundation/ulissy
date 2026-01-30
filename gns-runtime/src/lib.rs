//! # GNS Runtime
//! 
//! Runtime support library for ULissy-generated code.
//! 
//! This crate provides the platform abstractions and scheduling primitives
//! that ULissy programs depend on at runtime.
//! 
//! ## Core Components
//! 
//! - **Scheduling**: `schedule_every`, `watch_condition`, `delay`
//! - **Location**: `Location::current()`, H3 cell conversion
//! - **Sensors**: Battery level, device sensors, context digest
//! - **Time**: `Moment`, `Duration` with units
//! - **Network**: Epoch publishing, sync operations
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use gns_runtime::*;
//! 
//! // Schedule a task every 10 minutes
//! schedule_every(Duration::from_mins(10), || {
//!     if Battery::level() > Percent::from(20) {
//!         let cell = Location::current().to_h3(7)?;
//!         println!("Current cell: {}", cell);
//!     }
//!     Ok(())
//! })?;
//! ```

#![allow(dead_code)]
#![allow(unused_variables)]

pub mod duration;
pub mod moment;
pub mod distance;
pub mod percent;
pub mod location;
pub mod battery;
pub mod sensors;
pub mod scheduling;
pub mod network;
pub mod error;

// Re-export all public types
pub use duration::Duration;
pub use moment::Moment;
pub use distance::Distance;
pub use percent::Percent;
pub use location::{Location, Coordinates, H3Cell};
pub use battery::{Battery, BatteryLevel, PowerMode};
pub use sensors::{Sensors, SensorContext};
pub use scheduling::{schedule_every, watch_condition, delay, TaskHandle};
pub use network::{Network, SyncStatus};
pub use error::{RuntimeError, RuntimeResult};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::duration::Duration;
    pub use crate::moment::Moment;
    pub use crate::distance::Distance;
    pub use crate::percent::Percent;
    pub use crate::location::{Location, H3Cell};
    pub use crate::battery::Battery;
    pub use crate::sensors::Sensors;
    pub use crate::scheduling::*;
    pub use crate::error::*;
}
