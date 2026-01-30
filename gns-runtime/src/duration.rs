//! Duration type with unit support
//! 
//! ULissy: `10.minutes`, `2.hours`, `30.seconds`
//! Rust:   `Duration::from_mins(10)`, `Duration::from_hours(2)`

use std::ops::{Add, Sub, Mul, Div};
use std::time::Duration as StdDuration;
use serde::{Serialize, Deserialize};

/// Duration with semantic unit support
/// 
/// Provides human-readable duration construction matching ULissy syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Duration {
    /// Internal storage in milliseconds for precision
    millis: u64,
}

impl Duration {
    /// Create duration from milliseconds
    pub const fn from_millis(millis: u64) -> Self {
        Duration { millis }
    }
    
    /// Create duration from seconds
    /// ULissy: `30.seconds`
    pub const fn from_secs(secs: u64) -> Self {
        Duration { millis: secs * 1000 }
    }
    
    /// Create duration from minutes
    /// ULissy: `10.minutes`
    pub const fn from_mins(mins: u64) -> Self {
        Duration { millis: mins * 60 * 1000 }
    }
    
    /// Create duration from hours
    /// ULissy: `2.hours`
    pub const fn from_hours(hours: u64) -> Self {
        Duration { millis: hours * 60 * 60 * 1000 }
    }
    
    /// Create duration from days
    /// ULissy: `7.days`
    pub const fn from_days(days: u64) -> Self {
        Duration { millis: days * 24 * 60 * 60 * 1000 }
    }
    
    /// Zero duration
    pub const fn zero() -> Self {
        Duration { millis: 0 }
    }
    
    /// Get total milliseconds
    pub const fn as_millis(&self) -> u64 {
        self.millis
    }
    
    /// Get total seconds
    pub const fn as_secs(&self) -> u64 {
        self.millis / 1000
    }
    
    /// Get total minutes
    pub const fn as_mins(&self) -> u64 {
        self.millis / (60 * 1000)
    }
    
    /// Get total hours
    pub const fn as_hours(&self) -> u64 {
        self.millis / (60 * 60 * 1000)
    }
    
    /// Check if duration is zero
    pub const fn is_zero(&self) -> bool {
        self.millis == 0
    }
    
    /// Convert to std::time::Duration
    pub const fn to_std(&self) -> StdDuration {
        StdDuration::from_millis(self.millis)
    }
}

// Arithmetic operations

impl Add for Duration {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self {
        Duration { millis: self.millis + rhs.millis }
    }
}

impl Sub for Duration {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self {
        Duration { millis: self.millis.saturating_sub(rhs.millis) }
    }
}

impl Mul<u64> for Duration {
    type Output = Self;
    
    fn mul(self, rhs: u64) -> Self {
        Duration { millis: self.millis * rhs }
    }
}

impl Div<u64> for Duration {
    type Output = Self;
    
    fn div(self, rhs: u64) -> Self {
        Duration { millis: self.millis / rhs }
    }
}

// Conversion from std

impl From<StdDuration> for Duration {
    fn from(d: StdDuration) -> Self {
        Duration { millis: d.as_millis() as u64 }
    }
}

impl From<Duration> for StdDuration {
    fn from(d: Duration) -> Self {
        StdDuration::from_millis(d.millis)
    }
}

// Display

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.millis == 0 {
            write!(f, "0s")
        } else if self.millis < 1000 {
            write!(f, "{}ms", self.millis)
        } else if self.millis < 60 * 1000 {
            write!(f, "{}s", self.as_secs())
        } else if self.millis < 60 * 60 * 1000 {
            write!(f, "{}m", self.as_mins())
        } else if self.millis < 24 * 60 * 60 * 1000 {
            write!(f, "{}h", self.as_hours())
        } else {
            write!(f, "{}d", self.millis / (24 * 60 * 60 * 1000))
        }
    }
}

// Default

impl Default for Duration {
    fn default() -> Self {
        Duration::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_duration_creation() {
        assert_eq!(Duration::from_secs(60), Duration::from_mins(1));
        assert_eq!(Duration::from_mins(60), Duration::from_hours(1));
        assert_eq!(Duration::from_hours(24), Duration::from_days(1));
    }
    
    #[test]
    fn test_duration_arithmetic() {
        let a = Duration::from_mins(10);
        let b = Duration::from_mins(5);
        
        assert_eq!((a + b).as_mins(), 15);
        assert_eq!((a - b).as_mins(), 5);
        assert_eq!((a * 2).as_mins(), 20);
        assert_eq!((a / 2).as_mins(), 5);
    }
    
    #[test]
    fn test_duration_display() {
        assert_eq!(Duration::from_secs(30).to_string(), "30s");
        assert_eq!(Duration::from_mins(10).to_string(), "10m");
        assert_eq!(Duration::from_hours(2).to_string(), "2h");
    }
}
