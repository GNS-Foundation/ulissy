//! Battery and power management
//! 
//! ULissy: `battery`, `battery > 20`, `PowerMode.low`

use serde::{Serialize, Deserialize};
use crate::percent::Percent;

/// Battery level (wrapper around Percent for semantic clarity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BatteryLevel {
    percent: Percent,
}

impl BatteryLevel {
    /// Create battery level from percentage
    pub fn from_percent(p: Percent) -> Self {
        BatteryLevel { percent: p }
    }
    
    /// Create from integer (0-100)
    pub fn from_int(value: i32) -> Self {
        BatteryLevel {
            percent: Percent::from(value as f64),
        }
    }
    
    /// Get as percentage
    pub fn as_percent(&self) -> Percent {
        self.percent
    }
    
    /// Get as integer (0-100)
    pub fn as_int(&self) -> i32 {
        self.percent.as_int()
    }
    
    /// Check if battery is low (<20%)
    pub fn is_low(&self) -> bool {
        self.percent < 20
    }
    
    /// Check if battery is critical (<5%)
    pub fn is_critical(&self) -> bool {
        self.percent < 5
    }
    
    /// Check if battery is full (>95%)
    pub fn is_full(&self) -> bool {
        self.percent > 95
    }
}

impl std::fmt::Display for BatteryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.percent)
    }
}

// Comparison with i64 for generated code
impl PartialEq<i64> for BatteryLevel {
    fn eq(&self, other: &i64) -> bool {
        self.percent == *other
    }
}

impl PartialOrd<i64> for BatteryLevel {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.percent.partial_cmp(other)
    }
}

/// Power/energy mode for the device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerMode {
    /// Low power mode - reduced functionality
    Low,
    /// Normal operation
    Normal,
    /// Performance mode - maximum capability
    Performance,
}

impl PowerMode {
    /// Check if in low power mode
    pub fn is_low(&self) -> bool {
        matches!(self, PowerMode::Low)
    }
}

impl std::fmt::Display for PowerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerMode::Low => write!(f, "low"),
            PowerMode::Normal => write!(f, "normal"),
            PowerMode::Performance => write!(f, "performance"),
        }
    }
}

/// Battery state including charging status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BatteryState {
    pub level: BatteryLevel,
    pub is_charging: bool,
    pub power_mode: PowerMode,
}

/// Battery service - provides battery information
/// 
/// ULissy: `battery`, `Battery::level()`
pub struct Battery;

impl Battery {
    /// Get current battery level as Percent
    /// ULissy: `battery`, `Battery::level()`
    /// 
    /// Note: In a real implementation, this would call platform battery APIs.
    pub fn level() -> Percent {
        // TODO: Integrate with actual battery APIs
        // For now, return simulated value
        #[cfg(feature = "mobile")]
        {
            // Would call UIDevice.current.batteryLevel on iOS
            // or BatteryManager on Android
            unimplemented!("Mobile battery integration pending")
        }
        
        #[cfg(not(feature = "mobile"))]
        {
            // Simulated battery level for desktop/testing
            Percent::from(85)
        }
    }
    
    /// Get full battery state
    pub fn state() -> BatteryState {
        BatteryState {
            level: BatteryLevel::from_percent(Self::level()),
            is_charging: Self::is_charging(),
            power_mode: Self::power_mode(),
        }
    }
    
    /// Check if device is charging
    pub fn is_charging() -> bool {
        // TODO: Implement actual charging detection
        false
    }
    
    /// Get current power mode
    pub fn power_mode() -> PowerMode {
        // TODO: Implement actual power mode detection
        PowerMode::Normal
    }
    
    /// Check if battery level is sufficient for operation
    pub fn is_sufficient(threshold: Percent) -> bool {
        Self::level() >= threshold
    }
}

/// Global `battery` accessor for generated code
/// ULissy: `battery > 20`
pub fn battery() -> Percent {
    Battery::level()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_battery_level() {
        let level = BatteryLevel::from_int(50);
        assert_eq!(level.as_int(), 50);
        assert!(!level.is_low());
        assert!(!level.is_critical());
    }
    
    #[test]
    fn test_battery_comparison() {
        let level = BatteryLevel::from_int(25);
        assert!(level > 20);
        assert!(level < 30);
    }
    
    #[test]
    fn test_battery_service() {
        let level = Battery::level();
        assert!(level >= 0);
        assert!(level <= 100);
    }
}
