//! Percent type for battery levels and other percentages
//! 
//! ULissy: `80.percent`, `battery > 20%`

use std::ops::{Add, Sub};
use serde::{Serialize, Deserialize};

/// Percentage value (0-100)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Percent {
    /// Value 0.0 to 100.0
    value: f64,
}

impl Percent {
    /// Create percent from value (0-100)
    /// ULissy: `80.percent`
    pub fn from(value: impl Into<f64>) -> Self {
        let v = value.into();
        Percent { value: v.clamp(0.0, 100.0) }
    }
    
    /// Create percent from i64
    pub fn from_i64(value: i64) -> Self {
        Percent::from(value as f64)
    }
    
    /// Create percent from fraction (0.0-1.0)
    pub fn from_fraction(fraction: f64) -> Self {
        Percent { value: (fraction * 100.0).clamp(0.0, 100.0) }
    }
    
    /// Zero percent
    pub fn zero() -> Self {
        Percent { value: 0.0 }
    }
    
    /// Full (100%)
    pub fn full() -> Self {
        Percent { value: 100.0 }
    }
    
    /// Get value as percentage (0-100)
    pub fn value(&self) -> f64 {
        self.value
    }
    
    /// Get value as fraction (0.0-1.0)
    pub fn as_fraction(&self) -> f64 {
        self.value / 100.0
    }
    
    /// Get value as integer (0-100)
    pub fn as_int(&self) -> i32 {
        self.value.round() as i32
    }
    
    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.value == 0.0
    }
    
    /// Check if full (100%)
    pub fn is_full(&self) -> bool {
        self.value >= 100.0
    }
}

// Arithmetic

impl Add for Percent {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self {
        Percent::from(self.value + rhs.value)
    }
}

impl Sub for Percent {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self {
        Percent::from(self.value - rhs.value)
    }
}

// Comparison with i64 (for generated code: `battery > 20`)

impl PartialEq<i64> for Percent {
    fn eq(&self, other: &i64) -> bool {
        self.value == *other as f64
    }
}

impl PartialOrd<i64> for Percent {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&(*other as f64))
    }
}

// Display

impl std::fmt::Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.value.round() as i32)
    }
}

// Default

impl Default for Percent {
    fn default() -> Self {
        Percent::zero()
    }
}

// Eq
impl Eq for Percent {}

// Ord - required for BatteryLevel to derive Ord
// We use total_cmp to handle f64 comparison safely
impl Ord for Percent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.total_cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percent_creation() {
        let p = Percent::from(80);
        assert_eq!(p.value(), 80.0);
        assert_eq!(p.as_fraction(), 0.8);
    }
    
    #[test]
    fn test_percent_clamping() {
        let p = Percent::from(150);
        assert_eq!(p.value(), 100.0);
        
        let p2 = Percent::from(-10);
        assert_eq!(p2.value(), 0.0);
    }
    
    #[test]
    fn test_percent_comparison() {
        let p = Percent::from(80);
        assert!(p > 20);
        assert!(p < 90);
        assert!(p == 80);
    }
}
