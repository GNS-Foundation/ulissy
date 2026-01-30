//! Location services - GPS, coordinates, H3 cells
//! 
//! ULissy: `here`, `here.h3(7)`, `Location::current()`

use h3o::{CellIndex, LatLng, Resolution};
use serde::{Serialize, Deserialize};
use crate::error::{RuntimeError, RuntimeResult};
use crate::distance::Distance;

/// Raw GPS coordinates (restricted type in ULissy)
/// 
/// Cannot be directly serialized or transmitted - must be quantized to H3 first.
#[derive(Debug, Clone, Copy)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>,
}

impl Coordinates {
    /// Create new coordinates
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Coordinates {
            latitude,
            longitude,
            altitude: None,
            accuracy: None,
        }
    }
    
    /// Create with altitude
    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = Some(altitude);
        self
    }
    
    /// Create with accuracy
    pub fn with_accuracy(mut self, accuracy: f64) -> Self {
        self.accuracy = Some(accuracy);
        self
    }
    
    /// Convert to H3 cell at given resolution
    /// ULissy: `coordinates.h3(7)`
    pub fn to_h3(&self, resolution: u8) -> RuntimeResult<H3Cell> {
        let res = Resolution::try_from(resolution)
            .map_err(|_| RuntimeError::H3Error(format!("Invalid resolution: {}", resolution)))?;
        
        let ll = LatLng::new(self.latitude, self.longitude)
            .map_err(|e| RuntimeError::H3Error(format!("Invalid coordinates: {}", e)))?;
        
        let cell = ll.to_cell(res);
        
        Ok(H3Cell { index: cell })
    }
    
    /// Calculate distance to another point (haversine)
    pub fn distance_to(&self, other: &Coordinates) -> Distance {
        let r = 6371000.0; // Earth's radius in meters
        
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();
        
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        
        Distance::meters(r * c)
    }
}

/// H3 hexagonal cell index
/// 
/// Safe to serialize and transmit (privacy-preserving quantization).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct H3Cell {
    #[serde(with = "h3_serde")]
    index: CellIndex,
}

mod h3_serde {
    use h3o::CellIndex;
    use serde::{Deserialize, Deserializer, Serializer};
    
    pub fn serialize<S>(cell: &CellIndex, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&cell.to_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<CellIndex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl H3Cell {
    /// Create from H3 index string
    pub fn from_string(s: &str) -> RuntimeResult<Self> {
        let index: CellIndex = s.parse()
            .map_err(|e| RuntimeError::H3Error(format!("Invalid H3 index: {}", e)))?;
        Ok(H3Cell { index })
    }
    
    /// Create from raw u64 index
    pub fn from_u64(value: u64) -> RuntimeResult<Self> {
        let index = CellIndex::try_from(value)
            .map_err(|e| RuntimeError::H3Error(format!("Invalid H3 index: {}", e)))?;
        Ok(H3Cell { index })
    }
    
    /// Get the raw H3 index as u64
    pub fn as_u64(&self) -> u64 {
        self.index.into()
    }
    
    /// Get the H3 index as string
    pub fn as_string(&self) -> String {
        self.index.to_string()
    }
    
    /// Get the resolution (0-15)
    pub fn resolution(&self) -> u8 {
        self.index.resolution().into()
    }
    
    /// Get the center coordinates of this cell
    pub fn center(&self) -> Coordinates {
        let ll = LatLng::from(self.index);
        Coordinates::new(ll.lat(), ll.lng())
    }
    
    /// Check if this cell is a neighbor of another
    pub fn is_neighbor(&self, other: &H3Cell) -> bool {
        self.index.is_neighbor_with(other.index).unwrap_or(false)
    }
    
    /// Get distance to another cell (center to center)
    pub fn distance_to(&self, other: &H3Cell) -> Distance {
        self.center().distance_to(&other.center())
    }
    
    /// Get parent cell at lower resolution
    pub fn parent(&self, resolution: u8) -> RuntimeResult<H3Cell> {
        let res = Resolution::try_from(resolution)
            .map_err(|_| RuntimeError::H3Error(format!("Invalid resolution: {}", resolution)))?;
        
        let parent = self.index.parent(res)
            .ok_or_else(|| RuntimeError::H3Error("Cannot get parent at higher resolution".into()))?;
        
        Ok(H3Cell { index: parent })
    }
    
    /// Check if another cell is within range (in cells, not meters)
    pub fn within_range(&self, other: &H3Cell, max_cells: u32) -> bool {
        if let Ok(distance) = self.index.grid_distance(other.index) {
            distance <= max_cells as i32
        } else {
            false
        }
    }
}

impl std::fmt::Display for H3Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

/// Location service - provides current location
/// 
/// ULissy: `here`, `Location::current()`
pub struct Location;

impl Location {
    /// Get current location
    /// ULissy: `here`, `Location::current()`
    /// 
    /// Note: In a real implementation, this would call platform GPS APIs.
    /// For now, returns a simulated location.
    pub fn current() -> RuntimeResult<Coordinates> {
        // TODO: Integrate with actual GPS APIs
        // For now, return a default (San Francisco)
        #[cfg(feature = "mobile")]
        {
            // Would call CoreLocation on iOS, FusedLocationProvider on Android
            unimplemented!("Mobile GPS integration pending")
        }
        
        #[cfg(not(feature = "mobile"))]
        {
            // Simulated location for desktop/testing
            Ok(Coordinates::new(37.7749, -122.4194)
                .with_accuracy(10.0))
        }
    }
    
    /// Check if GPS is available
    pub fn gps_available() -> bool {
        // TODO: Check actual GPS availability
        true
    }
}

/// Global `here` accessor
/// ULissy: `here.h3(7)`
pub fn here() -> RuntimeResult<Coordinates> {
    Location::current()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coordinates_to_h3() {
        let coords = Coordinates::new(37.7749, -122.4194);
        let cell = coords.to_h3(7).unwrap();
        
        assert_eq!(cell.resolution(), 7);
    }
    
    #[test]
    fn test_h3_roundtrip() {
        let coords = Coordinates::new(40.7128, -74.0060); // NYC
        let cell = coords.to_h3(9).unwrap();
        
        let s = cell.as_string();
        let cell2 = H3Cell::from_string(&s).unwrap();
        
        assert_eq!(cell, cell2);
    }
    
    #[test]
    fn test_distance_calculation() {
        let sf = Coordinates::new(37.7749, -122.4194);
        let la = Coordinates::new(34.0522, -118.2437);
        
        let dist = sf.distance_to(&la);
        // SF to LA is roughly 560 km
        assert!(dist.as_kilometers() > 500.0);
        assert!(dist.as_kilometers() < 600.0);
    }
}
