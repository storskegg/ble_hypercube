use rstar::{RTree, RTreeObject, AABB};
use std::collections::{BTreeMap, HashMap};

/// Single BLE observation record
#[derive(Debug, Clone, Copy)]
pub struct BleObservation {
    pub rssi: i8,
    pub mac: [u8; 6],
    pub timestamp: i64, // Unix timestamp (seconds or microseconds)
    pub lat: f64,
    pub lon: f64,
}

/// Wrapper for R-tree spatial indexing
#[derive(Debug, Clone, Copy)]
struct GeoPoint {
    coords: [f64; 2], // [lat, lon]
    record_id: usize,
}

impl RTreeObject for GeoPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.coords)
    }
}

/// 4-dimensional cube structure for BLE observations
pub struct BleCube {
    // Canonical data store
    records: Vec<BleObservation>,

    // Indices (all store record IDs as usize)
    mac_index: HashMap<[u8; 6], Vec<usize>>,
    rssi_index: BTreeMap<i8, Vec<usize>>,
    time_index: BTreeMap<i64, Vec<usize>>,
    geo_index: RTree<GeoPoint>,
}

impl BleCube {
    /// Create a new empty cube
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            mac_index: HashMap::new(),
            rssi_index: BTreeMap::new(),
            time_index: BTreeMap::new(),
            geo_index: RTree::new(),
        }
    }

    /// Create with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
            mac_index: HashMap::with_capacity(capacity / 100), // estimate unique MACs
            rssi_index: BTreeMap::new(),
            time_index: BTreeMap::new(),
            geo_index: RTree::new(),
        }
    }

    /// Insert a new observation
    pub fn insert(&mut self, obs: BleObservation) -> usize {
        let record_id = self.records.len();
        self.records.push(obs);

        // Update MAC index
        self.mac_index
            .entry(obs.mac)
            .or_insert_with(Vec::new)
            .push(record_id);

        // Update RSSI index
        self.rssi_index
            .entry(obs.rssi)
            .or_insert_with(Vec::new)
            .push(record_id);

        // Update timestamp index
        self.time_index
            .entry(obs.timestamp)
            .or_insert_with(Vec::new)
            .push(record_id);

        // Update geo index
        self.geo_index.insert(GeoPoint {
            coords: [obs.lat, obs.lon],
            record_id,
        });

        record_id
    }

    /// Get observation by record ID
    pub fn get(&self, record_id: usize) -> Option<&BleObservation> {
        self.records.get(record_id)
    }

    /// Total number of observations
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    // ========== MAC ADDRESS QUERIES ==========

    /// Query by exact MAC address
    pub fn query_mac(&self, mac: &[u8; 6]) -> Vec<&BleObservation> {
        self.mac_index
            .get(mac)
            .map(|ids| ids.iter().filter_map(|&id| self.records.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all unique MAC addresses, sorted lexicographically
    pub fn get_all_macs(&self) -> Vec<[u8; 6]> {
        let mut macs: Vec<[u8; 6]> = self.mac_index.keys().copied().collect();
        macs.sort();
        macs
    }

    // ========== RSSI QUERIES ==========

    /// Query by exact RSSI value
    pub fn query_rssi(&self, rssi: i8) -> Vec<&BleObservation> {
        self.rssi_index
            .get(&rssi)
            .map(|ids| ids.iter().filter_map(|&id| self.records.get(id)).collect())
            .unwrap_or_default()
    }

    /// Query RSSI range [min, max] inclusive
    pub fn query_rssi_range(&self, min: i8, max: i8) -> Vec<&BleObservation> {
        self.rssi_index
            .range(min..=max)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    /// Query RSSI greater than threshold
    pub fn query_rssi_gt(&self, threshold: i8) -> Vec<&BleObservation> {
        self.rssi_index
            .range((threshold + 1)..=i8::MAX)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    /// Query RSSI greater than or equal to threshold
    pub fn query_rssi_gte(&self, threshold: i8) -> Vec<&BleObservation> {
        self.rssi_index
            .range(threshold..=i8::MAX)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    /// Query RSSI less than threshold
    pub fn query_rssi_lt(&self, threshold: i8) -> Vec<&BleObservation> {
        self.rssi_index
            .range(i8::MIN..threshold)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    /// Query RSSI less than or equal to threshold
    pub fn query_rssi_lte(&self, threshold: i8) -> Vec<&BleObservation> {
        self.rssi_index
            .range(i8::MIN..=threshold)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    // ========== TIMESTAMP QUERIES ==========

    /// Query by exact timestamp
    pub fn query_timestamp(&self, timestamp: i64) -> Vec<&BleObservation> {
        self.time_index
            .get(&timestamp)
            .map(|ids| ids.iter().filter_map(|&id| self.records.get(id)).collect())
            .unwrap_or_default()
    }

    /// Query timestamp range [start, end] inclusive
    pub fn query_time_range(&self, start: i64, end: i64) -> Vec<&BleObservation> {
        self.time_index
            .range(start..=end)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    /// Query timestamps after (greater than) a point
    pub fn query_time_after(&self, timestamp: i64) -> Vec<&BleObservation> {
        self.time_index
            .range((timestamp + 1)..=i64::MAX)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    /// Query timestamps before (less than) a point
    pub fn query_time_before(&self, timestamp: i64) -> Vec<&BleObservation> {
        self.time_index
            .range(i64::MIN..timestamp)
            .flat_map(|(_, ids)| ids.iter().filter_map(|&id| self.records.get(id)))
            .collect()
    }

    // ========== GEOLOCATION QUERIES ==========

    /// Query by radius (in meters) around a point
    /// Uses Haversine distance for accuracy
    pub fn query_geo_radius(&self, lat: f64, lon: f64, radius_m: f64) -> Vec<&BleObservation> {
        // Convert radius to approximate degrees (rough approximation)
        // 1 degree latitude ≈ 111km
        let radius_deg = radius_m / 111000.0;
        
        let envelope = AABB::from_corners(
            [lat - radius_deg, lon - radius_deg],
            [lat + radius_deg, lon + radius_deg],
        );

        self.geo_index
            .locate_in_envelope(&envelope)
            .filter(|point| {
                let dist = haversine_distance(lat, lon, point.coords[0], point.coords[1]);
                dist <= radius_m
            })
            .filter_map(|point| self.records.get(point.record_id))
            .collect()
    }

    /// Query within a bounding box (min_lat, min_lon, max_lat, max_lon)
    pub fn query_geo_bbox(
        &self,
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
    ) -> Vec<&BleObservation> {
        let envelope = AABB::from_corners([min_lat, min_lon], [max_lat, max_lon]);

        self.geo_index
            .locate_in_envelope(&envelope)
            .filter_map(|point| self.records.get(point.record_id))
            .collect()
    }

    /// Query within a polygon (simple point-in-polygon test)
    /// Polygon vertices as [(lat, lon), ...]
    pub fn query_geo_polygon(&self, polygon: &[(f64, f64)]) -> Vec<&BleObservation> {
        if polygon.len() < 3 {
            return Vec::new();
        }

        // Get bounding box of polygon for initial filtering
        let (min_lat, max_lat, min_lon, max_lon) = polygon.iter().fold(
            (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
            |(min_lat, max_lat, min_lon, max_lon), &(lat, lon)| {
                (
                    min_lat.min(lat),
                    max_lat.max(lat),
                    min_lon.min(lon),
                    max_lon.max(lon),
                )
            },
        );

        let envelope = AABB::from_corners([min_lat, min_lon], [max_lat, max_lon]);

        self.geo_index
            .locate_in_envelope(&envelope)
            .filter(|point| point_in_polygon(point.coords[0], point.coords[1], polygon))
            .filter_map(|point| self.records.get(point.record_id))
            .collect()
    }

    // ========== MULTI-DIMENSIONAL QUERIES ==========

    /// Combined query: filter by multiple dimensions
    /// Returns record IDs for further custom filtering
    pub fn query_multi(
        &self,
        mac: Option<&[u8; 6]>,
        rssi_range: Option<(i8, i8)>,
        time_range: Option<(i64, i64)>,
        geo_center: Option<(f64, f64, f64)>, // (lat, lon, radius_m)
    ) -> Vec<&BleObservation> {
        // Start with the most selective dimension
        let mut result_ids: Vec<usize> = if let Some(mac_addr) = mac {
            self.mac_index.get(mac_addr).cloned().unwrap_or_default()
        } else {
            (0..self.records.len()).collect()
        };

        // Filter by RSSI
        if let Some((min_rssi, max_rssi)) = rssi_range {
            let rssi_ids: Vec<usize> = self
                .rssi_index
                .range(min_rssi..=max_rssi)
                .flat_map(|(_, ids)| ids.iter().copied())
                .collect();
            result_ids.retain(|id| rssi_ids.contains(id));
        }

        // Filter by timestamp
        if let Some((start, end)) = time_range {
            let time_ids: Vec<usize> = self
                .time_index
                .range(start..=end)
                .flat_map(|(_, ids)| ids.iter().copied())
                .collect();
            result_ids.retain(|id| time_ids.contains(id));
        }

        // Filter by geolocation
        if let Some((lat, lon, radius)) = geo_center {
            let geo_results = self.query_geo_radius(lat, lon, radius);
            let geo_ids: Vec<usize> = geo_results
                .iter()
                .filter_map(|obs| {
                    self.records
                        .iter()
                        .position(|r| std::ptr::eq(*obs, r))
                })
                .collect();
            result_ids.retain(|id| geo_ids.contains(id));
        }

        result_ids
            .iter()
            .filter_map(|&id| self.records.get(id))
            .collect()
    }
}

// ========== HELPER FUNCTIONS ==========

/// Haversine distance between two points (lat1, lon1) and (lat2, lon2) in meters
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371000.0; // Earth radius in meters

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    R * c
}

/// Point-in-polygon test using ray casting algorithm
fn point_in_polygon(lat: f64, lon: f64, polygon: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let n = polygon.len();

    let mut j = n - 1;
    for i in 0..n {
        let (lat_i, lon_i) = polygon[i];
        let (lat_j, lon_j) = polygon[j];

        if ((lon_i > lon) != (lon_j > lon))
            && (lat < (lat_j - lat_i) * (lon - lon_i) / (lon_j - lon_i) + lat_i)
        {
            inside = !inside;
        }
        j = i;
    }

    inside
}

impl Default for BleCube {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert_and_query() {
        let mut cube = BleCube::new();

        let obs1 = BleObservation {
            rssi: -65,
            mac: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
            timestamp: 1700000000,
            lat: 37.7749,
            lon: -122.4194,
        };

        let id = cube.insert(obs1);
        assert_eq!(id, 0);
        assert_eq!(cube.len(), 1);

        let results = cube.query_mac(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rssi, -65);
    }

    #[test]
    fn test_rssi_range_query() {
        let mut cube = BleCube::new();

        cube.insert(BleObservation {
            rssi: -50,
            mac: [0; 6],
            timestamp: 0,
            lat: 0.0,
            lon: 0.0,
        });
        cube.insert(BleObservation {
            rssi: -70,
            mac: [0; 6],
            timestamp: 0,
            lat: 0.0,
            lon: 0.0,
        });
        cube.insert(BleObservation {
            rssi: -90,
            mac: [0; 6],
            timestamp: 0,
            lat: 0.0,
            lon: 0.0,
        });

        let results = cube.query_rssi_range(-80, -60);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rssi, -70);

        let results_gte = cube.query_rssi_gte(-70);
        assert_eq!(results_gte.len(), 2); // -50 and -70
    }

    #[test]
    fn test_geo_radius_query() {
        let mut cube = BleCube::new();

        // San Francisco
        cube.insert(BleObservation {
            rssi: -60,
            mac: [0; 6],
            timestamp: 0,
            lat: 37.7749,
            lon: -122.4194,
        });

        // Oakland (about 13km away)
        cube.insert(BleObservation {
            rssi: -60,
            mac: [0; 6],
            timestamp: 0,
            lat: 37.8044,
            lon: -122.2712,
        });

        // Query 10km radius around SF
        let results = cube.query_geo_radius(37.7749, -122.4194, 10000.0);
        assert_eq!(results.len(), 1);

        // Query 20km radius
        let results = cube.query_geo_radius(37.7749, -122.4194, 20000.0);
        assert_eq!(results.len(), 2);
    }
}
