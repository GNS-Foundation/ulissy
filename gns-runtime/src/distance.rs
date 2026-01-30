//! Distance type with unit support
//! 
//! ULissy: `500.meters`, `2.kilometers`, `1.miles`

use std::ops::{Add, Sub, Mul, Div};
use serde::{Serialize, Deserialize};

/// Distance with unit awareness
/// 
/// Internally stored in meters for consistency.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Distance {
    /// Internal storage in meters
    meters: f64,
}

impl Distance {
    /// Create distance from meters
    /// ULissy: `500.meters`
    pub fn meters(m: f64) -> Self {
        Distance { meters: m }
    }
    
    /// Create distance from kilometers
    /// ULissy: `2.kilometers`
    pub fn kilometers(km: f64) -> Self {
        Distance { meters: km * 1000.0 }
    }
    
    /// Create distance from miles
    /// ULissy: `1.miles`
    pub fn miles(mi: f64) -> Self {
        Distance { meters: mi * 1609.344 }
    }
    
    /// Create distance from feet
    pub fn feet(ft: f64) -> Self {
        Distance { meters: ft * 0.3048 }
    }
    
    /// Zero distance
    pub fn zero() -> Self {
        Distance { meters: 0.0 }
    }
    
    /// Get value in meters
    pub fn as_meters(&self) -> f64 {
        self.meters
    }
    
    /// Get value in kilometers
    pub fn as_kilometers(&self) -> f64 {
        self.meters / 1000.0
    }
    
    /// Get value in miles
    pub fn as_miles(&self) -> f64 {
        self.meters / 1609.344
    }
    
    /// Get value in feet
    pub fn as_feet(&self) -> f64 {
        self.meters / 0.3048
    }
    
    /// Check if distance is zero
    pub fn is_zero(&self) -> bool {
        self.meters == 0.0
    }
    
    /// Check if distance is within a threshold
    pub fn within(&self, threshold: Distance) -> bool {
        self.meters <= threshold.meters
    }
}

// Arithmetic operations

impl Add for Distance {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self {
        Distance { meters: self.meters + rhs.meters }
    }
}

impl Sub for Distance {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self {
        Distance { meters: (self.meters - rhs.meters).max(0.0) }
    }
}

impl Mul<f64> for Distance {
    type Output = Self;
    
    fn mul(self, rhs: f64) -> Self {
        Distance { meters: self.meters * rhs }
    }
}

impl Div<f64> for Distance {
    type Output = Self;
    
    fn div(self, rhs: f64) -> Self {
        Distance { meters: self.meters / rhs }
    }
}

// Integer multiplication

impl Mul<i64> for Distance {
    type Output = Self;
    
    fn mul(self, rhs: i64) -> Self {
        Distance { meters: self.meters * rhs as f64 }
    }
}

// Display

impl std::fmt::Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.meters < 1.0 {
            write!(f, "{:.2}cm", self.meters * 100.0)
        } else if self.meters < 1000.0 {
            write!(f, "{:.1}m", self.meters)
        } else {
            write!(f, "{:.2}km", self.as_kilometers())
        }
    }
}

// Default

impl Default for Distance {
    fn default() -> Self {
        Distance::zero()
    }
}

// Eq (with tolerance)
impl Eq for Distance {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_distance_conversions() {
        let d = Distance::kilometers(1.0);
        assert_eq!(d.as_meters(), 1000.0);
        
        let d2 = Distance::miles(1.0);
        assert!((d2.as_meters() - 1609.344).abs() < 0.001);
    }
    
    #[test]
    fn test_distance_arithmetic() {
        let a = Distance::meters(500.0);
        let b = Distance::meters(300.0);
        
        assert_eq!((a + b).as_meters(), 800.0);
        assert_eq!((a - b).as_meters(), 200.0);
        assert_eq!((a * 2.0).as_meters(), 1000.0);
    }
    
    #[test]
    fn test_distance_within() {
        let d = Distance::meters(100.0);
        assert!(d.within(Distance::meters(150.0)));
        assert!(!d.within(Distance::meters(50.0)));
    }
}
