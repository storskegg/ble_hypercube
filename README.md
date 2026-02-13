# BLE Cube: 4-Dimensional BLE Observation Data Structure

High-performance in-memory data structure for BLE scanner observations with O(1) to O(log n) access across four dimensions.

## Features

- **4-dimensional indexing**: RSSI, MAC Address, Timestamp, Geolocation
- **Optimized for high-frequency writes**: 30-50 inserts/second sustained
- **Low-latency queries**: Hash-based (O(1)) and tree-based (O(log n)) access
- **Spatial queries**: Radius search, bounding box, polygon containment
- **Memory efficient**: ~56-72 bytes per record including indices
- **Target scale**: ~50M records in 2GB RAM

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Primary Data Store                       │
│              Vec<BleObservation> (canonical)                │
└─────────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┬───────────────┐
        │                  │                  │               │
        ▼                  ▼                  ▼               ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐    ┌──────────┐
  │   MAC    │      │   RSSI   │      │   TIME   │    │   GEO    │
  │ HashMap  │      │ BTreeMap │      │ BTreeMap │    │  RTree   │
  │  O(1)    │      │ O(log n) │      │ O(log n) │    │ O(log n) │
  └──────────┘      └──────────┘      └──────────┘    └──────────┘
       │                  │                  │               │
       └──────────────────┴──────────────────┴───────────────┘
                           │
                     Vec<usize> (record IDs)
```

## Installation

```toml
[dependencies]
rstar = "0.12"
```

## Usage

### Basic Insert and Query

```rust
use ble_cube::{BleCube, BleObservation};

let mut cube = BleCube::new();

let obs = BleObservation {
    rssi: -65,
    mac: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
    timestamp: 1700000000,
    lat: 37.7749,
    lon: -122.4194,
};

cube.insert(obs);
```

### MAC Address Queries

```rust
// Exact MAC lookup (O(1))
let results = cube.query_mac(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);

// Get all unique MACs
let macs = cube.get_all_macs();
```

### RSSI Queries

```rust
// Exact value
let exact = cube.query_rssi(-65);

// Range query (O(log n))
let range = cube.query_rssi_range(-80, -60);

// Comparison queries
let strong = cube.query_rssi_gt(-70);      // > -70 dBm
let weak = cube.query_rssi_lte(-80);       // <= -80 dBm
```

### Timestamp Queries

```rust
// Exact timestamp
let exact = cube.query_timestamp(1700000000);

// Time window (O(log n))
let recent = cube.query_time_range(start_ts, end_ts);

// Relative queries
let after = cube.query_time_after(1700000000);
let before = cube.query_time_before(1700001000);
```

### Geolocation Queries

```rust
// Radius search (O(log n))
// 5km around San Francisco
let nearby = cube.query_geo_radius(37.7749, -122.4194, 5000.0);

// Bounding box
let bbox = cube.query_geo_bbox(
    37.77, -122.42,  // min_lat, min_lon
    37.81, -122.27,  // max_lat, max_lon
);

// Polygon containment
let polygon = vec![
    (37.7, -122.5),
    (37.9, -122.5),
    (37.8, -122.2),
];
let in_area = cube.query_geo_polygon(&polygon);
```

### Multi-Dimensional Queries

```rust
// Combine filters across dimensions
let results = cube.query_multi(
    Some(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]), // MAC
    Some((-70, -60)),                              // RSSI range
    Some((1700000000, 1700000120)),                // Time window
    Some((37.7749, -122.4194, 10000.0)),          // 10km radius
);
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| `insert()` | O(log n) | 4 index updates per insert |
| `query_mac()` | O(1) | Hash table lookup |
| `query_rssi()` | O(1) | BTreeMap point lookup |
| `query_rssi_range()` | O(log n + k) | k = results |
| `query_timestamp()` | O(1) | BTreeMap point lookup |
| `query_time_range()` | O(log n + k) | k = results |
| `query_geo_radius()` | O(log n + k) | R-tree spatial query |
| `query_geo_bbox()` | O(log n + k) | R-tree envelope query |
| `query_multi()` | O(log n + k) | Intersection of indices |

**Write throughput**: Tested at 30-50 inserts/second sustained with no degradation.

**Memory footprint**:
- ~40 bytes per `BleObservation` struct
- ~16-32 bytes per record in indices (4 indices × 4-8 bytes/pointer)
- **Total**: ~56-72 bytes per record
- **2GB capacity**: ~50 million records

## Data Types

```rust
pub struct BleObservation {
    pub rssi: i8,           // -103 to 0 dBm typical
    pub mac: [u8; 6],       // 48-bit MAC address
    pub timestamp: i64,     // Unix timestamp
    pub lat: f64,           // Latitude
    pub lon: f64,           // Longitude
}
```

## Spatial Query Accuracy

- **Radius queries**: Use Haversine distance for spherical accuracy
- **Bounding box**: Fast approximate pre-filter, exact inside R-tree
- **Polygon**: Ray casting algorithm for point-in-polygon test

## Thread Safety

**Not thread-safe by default.** Wrap in `Arc<RwLock<BleCube>>` for concurrent access:

```rust
use std::sync::{Arc, RwLock};

let cube = Arc::new(RwLock::new(BleCube::new()));

// Write
{
    let mut c = cube.write().unwrap();
    c.insert(obs);
}

// Read
{
    let c = cube.read().unwrap();
    let results = c.query_mac(&mac);
}
```

## Assumptions

- RSSI range: -103 to 0 dBm (i8)
- Timestamps: Unix epoch seconds or microseconds (i64)
- Geolocation: WGS84 coordinates (lat/lon in degrees)
- Write frequency: 30-50 Hz sustained
- Target dataset: ~2GB in-memory (50M records)

## Future Optimizations

- **Batch inserts**: Amortize index update costs
- **Concurrent queries**: Read-optimized locking strategies
- **Compression**: Delta encoding for timestamps, geohash for coordinates
- **Disk persistence**: Memory-mapped file backing for cold data
- **Custom allocators**: Arena allocation for reduced fragmentation

## License

MIT
