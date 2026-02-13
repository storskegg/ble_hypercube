use ble_cube::{BleCube, BleObservation};

fn main() {
    let mut cube = BleCube::with_capacity(1000);

    // Insert sample data
    let obs1 = BleObservation {
        rssi: -65,
        mac: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        timestamp: 1700000000,
        lat: 37.7749,
        lon: -122.4194,
    };

    let obs2 = BleObservation {
        rssi: -72,
        mac: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        timestamp: 1700000100,
        lat: 37.7750,
        lon: -122.4195,
    };

    let obs3 = BleObservation {
        rssi: -80,
        mac: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
        timestamp: 1700000200,
        lat: 37.8044,
        lon: -122.2712,
    };

    cube.insert(obs1);
    cube.insert(obs2);
    cube.insert(obs3);

    println!("Total observations: {}\n", cube.len());

    // ========== MAC ADDRESS QUERIES ==========
    println!("=== MAC Address Queries ===");
    
    let mac_results = cube.query_mac(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    println!("Observations for MAC AA:BB:CC:DD:EE:FF: {}", mac_results.len());
    for obs in mac_results {
        println!("  RSSI: {} dBm, Time: {}", obs.rssi, obs.timestamp);
    }

    let all_macs = cube.get_all_macs();
    println!("\nUnique MACs: {}", all_macs.len());
    for mac in all_macs {
        println!("  {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", 
                 mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    }

    // ========== RSSI QUERIES ==========
    println!("\n=== RSSI Queries ===");
    
    let rssi_exact = cube.query_rssi(-72);
    println!("Observations with RSSI = -72 dBm: {}", rssi_exact.len());

    let rssi_range = cube.query_rssi_range(-75, -65);
    println!("Observations with RSSI in [-75, -65] dBm: {}", rssi_range.len());

    let rssi_strong = cube.query_rssi_gt(-70);
    println!("Observations with RSSI > -70 dBm (strong signals): {}", rssi_strong.len());

    let rssi_weak = cube.query_rssi_lte(-75);
    println!("Observations with RSSI <= -75 dBm (weak signals): {}", rssi_weak.len());

    // ========== TIMESTAMP QUERIES ==========
    println!("\n=== Timestamp Queries ===");
    
    let time_exact = cube.query_timestamp(1700000100);
    println!("Observations at timestamp 1700000100: {}", time_exact.len());

    let time_range = cube.query_time_range(1700000000, 1700000150);
    println!("Observations in time range [1700000000, 1700000150]: {}", time_range.len());

    let recent = cube.query_time_after(1700000100);
    println!("Observations after timestamp 1700000100: {}", recent.len());

    // ========== GEOLOCATION QUERIES ==========
    println!("\n=== Geolocation Queries ===");
    
    // 5km radius around San Francisco
    let nearby = cube.query_geo_radius(37.7749, -122.4194, 5000.0);
    println!("Observations within 5km of SF coordinates: {}", nearby.len());

    // 20km radius
    let wider = cube.query_geo_radius(37.7749, -122.4194, 20000.0);
    println!("Observations within 20km of SF coordinates: {}", wider.len());

    // Bounding box query
    let bbox = cube.query_geo_bbox(37.77, -122.42, 37.81, -122.27);
    println!("Observations in bounding box: {}", bbox.len());

    // Polygon query (triangle around SF Bay Area)
    let polygon = vec![
        (37.7, -122.5),
        (37.9, -122.5),
        (37.8, -122.2),
    ];
    let in_poly = cube.query_geo_polygon(&polygon);
    println!("Observations in polygon: {}", in_poly.len());

    // ========== MULTI-DIMENSIONAL QUERIES ==========
    println!("\n=== Multi-Dimensional Queries ===");
    
    // Strong signals from specific MAC in last 2 minutes within 10km
    let combined = cube.query_multi(
        Some(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]), // MAC filter
        Some((-70, -60)),                              // RSSI range
        Some((1700000000, 1700000120)),                // Time range
        Some((37.7749, -122.4194, 10000.0)),          // Geo radius
    );
    println!("Complex query (MAC + RSSI + Time + Geo): {} results", combined.len());
    for obs in combined {
        println!("  RSSI: {} dBm, Lat: {:.4}, Lon: {:.4}, Time: {}", 
                 obs.rssi, obs.lat, obs.lon, obs.timestamp);
    }

    // Direct record access
    println!("\n=== Direct Record Access ===");
    if let Some(record) = cube.get(0) {
        println!("Record 0: RSSI={} dBm, MAC={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                 record.rssi,
                 record.mac[0], record.mac[1], record.mac[2],
                 record.mac[3], record.mac[4], record.mac[5]);
    }
}
