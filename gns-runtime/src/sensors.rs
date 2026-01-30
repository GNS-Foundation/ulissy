//! Device sensors and context generation
//! 
//! ULissy: `sensors`, `sensors.digest`

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Sensor context - ambient data from device sensors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorContext {
    /// WiFi networks visible (BSSID hashes)
    pub wifi_signatures: Vec<String>,
    
    /// Cell tower identifiers
    pub cell_towers: Vec<String>,
    
    /// Bluetooth devices nearby (hashed)
    pub bluetooth_signatures: Vec<String>,
    
    /// Accelerometer magnitude (movement indicator)
    pub accelerometer_magnitude: Option<f64>,
    
    /// Barometric pressure (altitude indicator)
    pub barometric_pressure: Option<f64>,
    
    /// Ambient light level
    pub ambient_light: Option<f64>,
    
    /// Timestamp of reading
    pub timestamp: i64,
}

impl SensorContext {
    /// Create empty sensor context
    pub fn empty() -> Self {
        SensorContext {
            wifi_signatures: Vec::new(),
            cell_towers: Vec::new(),
            bluetooth_signatures: Vec::new(),
            accelerometer_magnitude: None,
            barometric_pressure: None,
            ambient_light: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
    
    /// Generate a cryptographic digest of the sensor context
    /// ULissy: `sensors.digest`
    /// 
    /// This creates a SHA-256 hash of the sensor data, providing
    /// a compact proof of the ambient environment without revealing
    /// specific network identifiers.
    pub fn digest(&self) -> String {
        let mut hasher = Sha256::new();
        
        // Hash WiFi signatures
        for sig in &self.wifi_signatures {
            hasher.update(sig.as_bytes());
        }
        
        // Hash cell towers
        for tower in &self.cell_towers {
            hasher.update(tower.as_bytes());
        }
        
        // Hash Bluetooth signatures
        for sig in &self.bluetooth_signatures {
            hasher.update(sig.as_bytes());
        }
        
        // Hash numeric sensors
        if let Some(acc) = self.accelerometer_magnitude {
            hasher.update(acc.to_le_bytes());
        }
        if let Some(pressure) = self.barometric_pressure {
            hasher.update(pressure.to_le_bytes());
        }
        if let Some(light) = self.ambient_light {
            hasher.update(light.to_le_bytes());
        }
        
        // Hash timestamp
        hasher.update(self.timestamp.to_le_bytes());
        
        // Return hex-encoded hash
        let result = hasher.finalize();
        hex::encode(result)
    }
    
    /// Check if context has WiFi data
    pub fn has_wifi(&self) -> bool {
        !self.wifi_signatures.is_empty()
    }
    
    /// Check if context has cell data
    pub fn has_cell(&self) -> bool {
        !self.cell_towers.is_empty()
    }
    
    /// Check if context has motion data
    pub fn has_motion(&self) -> bool {
        self.accelerometer_magnitude.is_some()
    }
}

impl Default for SensorContext {
    fn default() -> Self {
        Self::empty()
    }
}

/// Sensors service - provides access to device sensors
/// 
/// ULissy: `sensors`, `Sensors::current()`
pub struct Sensors;

impl Sensors {
    /// Get current sensor readings
    /// ULissy: `sensors`, `Sensors::current()`
    pub fn current() -> SensorContext {
        // TODO: Integrate with actual sensor APIs
        // For now, return simulated data
        #[cfg(feature = "mobile")]
        {
            // Would call CoreMotion on iOS, SensorManager on Android
            unimplemented!("Mobile sensor integration pending")
        }
        
        #[cfg(not(feature = "mobile"))]
        {
            // Simulated sensor context for desktop/testing
            let mut ctx = SensorContext::empty();
            
            // Simulate some WiFi networks
            ctx.wifi_signatures = vec![
                "simulated_wifi_1".to_string(),
                "simulated_wifi_2".to_string(),
            ];
            
            // Simulate accelerometer
            ctx.accelerometer_magnitude = Some(9.81); // Roughly 1g
            
            ctx
        }
    }
    
    /// Get digest of current sensor state
    /// ULissy: `sensors.digest`
    pub fn digest() -> String {
        Self::current().digest()
    }
    
    /// Check if sensors are available
    pub fn available() -> bool {
        // TODO: Check actual sensor availability
        true
    }
}

/// GPS availability checker
pub struct Gps;

impl Gps {
    /// Check if GPS is available
    /// ULissy: `gps.available`
    pub fn available() -> bool {
        // TODO: Check actual GPS availability
        #[cfg(feature = "mobile")]
        {
            // Would check location permissions and hardware
            unimplemented!("Mobile GPS check pending")
        }
        
        #[cfg(not(feature = "mobile"))]
        {
            true
        }
    }
}

/// Global accessor for generated code
pub mod gps {
    pub fn available() -> bool {
        super::Gps::available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sensor_context_digest() {
        let ctx1 = SensorContext {
            wifi_signatures: vec!["wifi1".to_string()],
            cell_towers: vec!["tower1".to_string()],
            bluetooth_signatures: vec![],
            accelerometer_magnitude: Some(9.81),
            barometric_pressure: None,
            ambient_light: None,
            timestamp: 1000,
        };
        
        let ctx2 = SensorContext {
            wifi_signatures: vec!["wifi1".to_string()],
            cell_towers: vec!["tower1".to_string()],
            bluetooth_signatures: vec![],
            accelerometer_magnitude: Some(9.81),
            barometric_pressure: None,
            ambient_light: None,
            timestamp: 1000,
        };
        
        // Same context should produce same digest
        assert_eq!(ctx1.digest(), ctx2.digest());
        
        // Different context should produce different digest
        let ctx3 = SensorContext {
            wifi_signatures: vec!["wifi2".to_string()],
            ..ctx1.clone()
        };
        assert_ne!(ctx1.digest(), ctx3.digest());
    }
    
    #[test]
    fn test_sensors_service() {
        let ctx = Sensors::current();
        let digest = ctx.digest();
        
        // Digest should be 64 hex chars (SHA-256)
        assert_eq!(digest.len(), 64);
    }
}
