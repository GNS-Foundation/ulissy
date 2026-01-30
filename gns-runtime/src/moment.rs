//! Moment type - an instant in time
//! 
//! ULissy: `now`, `crumb.timestamp`

use std::ops::{Add, Sub};
use chrono::{DateTime, Utc, TimeZone};
use serde::{Serialize, Deserialize};
use crate::duration::Duration;

/// A moment in time (instant)
/// 
/// Represents a specific point in time with millisecond precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Moment {
    /// Unix timestamp in milliseconds
    timestamp_ms: i64,
}

impl Moment {
    /// Get the current moment
    /// ULissy: `now`
    pub fn now() -> Self {
        Moment {
            timestamp_ms: Utc::now().timestamp_millis(),
        }
    }
    
    /// Create moment from Unix timestamp (seconds)
    pub fn from_unix(secs: i64) -> Self {
        Moment {
            timestamp_ms: secs * 1000,
        }
    }
    
    /// Create moment from Unix timestamp (milliseconds)
    pub fn from_unix_millis(millis: i64) -> Self {
        Moment {
            timestamp_ms: millis,
        }
    }
    
    /// Create moment from chrono DateTime
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Moment {
            timestamp_ms: dt.timestamp_millis(),
        }
    }
    
    /// Get Unix timestamp in seconds
    pub fn unix(&self) -> i64 {
        self.timestamp_ms / 1000
    }
    
    /// Get Unix timestamp in milliseconds
    pub fn unix_millis(&self) -> i64 {
        self.timestamp_ms
    }
    
    /// Convert to chrono DateTime
    pub fn to_datetime(&self) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(self.timestamp_ms)
            .single()
            .unwrap_or_else(Utc::now)
    }
    
    /// Duration since another moment
    pub fn since(&self, other: Moment) -> Duration {
        let diff = self.timestamp_ms - other.timestamp_ms;
        if diff > 0 {
            Duration::from_millis(diff as u64)
        } else {
            Duration::zero()
        }
    }
    
    /// Duration until another moment
    pub fn until(&self, other: Moment) -> Duration {
        other.since(*self)
    }
    
    /// Check if this moment is before another
    pub fn is_before(&self, other: Moment) -> bool {
        self.timestamp_ms < other.timestamp_ms
    }
    
    /// Check if this moment is after another
    pub fn is_after(&self, other: Moment) -> bool {
        self.timestamp_ms > other.timestamp_ms
    }
    
    /// ISO 8601 formatted string
    pub fn to_iso8601(&self) -> String {
        self.to_datetime().to_rfc3339()
    }
}

// Arithmetic with Duration

impl Add<Duration> for Moment {
    type Output = Self;
    
    fn add(self, rhs: Duration) -> Self {
        Moment {
            timestamp_ms: self.timestamp_ms + rhs.as_millis() as i64,
        }
    }
}

impl Sub<Duration> for Moment {
    type Output = Self;
    
    fn sub(self, rhs: Duration) -> Self {
        Moment {
            timestamp_ms: self.timestamp_ms - rhs.as_millis() as i64,
        }
    }
}

impl Sub for Moment {
    type Output = Duration;
    
    fn sub(self, rhs: Self) -> Duration {
        self.since(rhs)
    }
}

// Display

impl std::fmt::Display for Moment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_iso8601())
    }
}

// Default (epoch)

impl Default for Moment {
    fn default() -> Self {
        Moment::from_unix(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_moment_now() {
        let m1 = Moment::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let m2 = Moment::now();
        
        assert!(m2.is_after(m1));
    }
    
    #[test]
    fn test_moment_arithmetic() {
        let m1 = Moment::from_unix(1000);
        let m2 = m1 + Duration::from_secs(60);
        
        assert_eq!(m2.unix(), 1060);
        
        let diff = m2 - m1;
        assert_eq!(diff.as_secs(), 60);
    }
}
