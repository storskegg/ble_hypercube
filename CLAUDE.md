# CLAUDE.md

## Project Overview

**ble-cube** (crate name) / **ble_hypercube** (repo name) is a high-performance, in-memory Rust data structure for BLE (Bluetooth Low Energy) scanner observations. It provides O(1) to O(log n) access across four dimensions: MAC address, RSSI, Timestamp, and Geolocation.

Design targets: 30-50 inserts/second sustained, ~50M records in 2GB RAM (~56-72 bytes/record).

## Quick Reference

```sh
cargo build                  # Compile
cargo test                   # Run all tests (unit tests in src/ble_cube.rs)
cargo clippy                 # Lint
cargo fmt                    # Format code
cargo fmt -- --check         # Check formatting without modifying
cargo bench                  # Run benchmarks (criterion, bench name: cube_bench)
cargo run --example usage    # Run the usage example
```

## Tech Stack

- **Language:** Rust (edition 2021)
- **Build system:** Cargo
- **Dependencies:**
  - `rstar = "0.12"` — R-tree spatial indexing for geo queries
- **Dev dependencies:**
  - `criterion = "0.5"` — Benchmarking framework
- **License:** Apache 2.0

## Repository Structure

```
ble_hypercube/
├── Cargo.toml              # Package manifest (name: "ble-cube", v0.1.0)
├── LICENSE                  # Apache License 2.0
├── README.md                # User-facing documentation with API examples
├── CLAUDE.md                # This file — AI assistant guide
├── .gitignore               # Ignores: target/, debug/, *.rs.bk, *.pdb, mutants.out*/, .idea/
├── src/
│   ├── lib.rs               # Library root — re-exports BleCube and BleObservation
│   └── ble_cube.rs          # Core implementation (~470 lines, includes unit tests)
└── examples/
    └── usage.rs             # Demonstrates all query types
```

## Architecture

The data structure uses a central `Vec<BleObservation>` as the canonical store, with four secondary indices that map dimension values to record IDs (`usize` positions into the Vec):

| Index | Type | Lookup | Use |
|-------|------|--------|-----|
| MAC | `HashMap<[u8; 6], Vec<usize>>` | O(1) | Exact MAC address lookup |
| RSSI | `BTreeMap<i8, Vec<usize>>` | O(log n) | Range/comparison queries |
| Timestamp | `BTreeMap<i64, Vec<usize>>` | O(log n) | Range/comparison queries |
| Geo | `RTree<GeoPoint>` | O(log n) | Radius, bounding box, polygon queries |

### Key Types

- **`BleObservation`** — Core data record: `rssi: i8`, `mac: [u8; 6]`, `timestamp: i64`, `lat: f64`, `lon: f64`
- **`BleCube`** — Main data structure holding the Vec + 4 indices
- **`GeoPoint`** (internal) — R-tree wrapper storing `[lat, lon]` coords + `record_id`

### Public API Surface

All public items are in `src/ble_cube.rs`, re-exported via `src/lib.rs`:

- `BleCube::new()`, `BleCube::with_capacity(n)` — Constructors
- `insert(obs)` — Insert observation, returns record ID
- `get(id)` — Direct record access by ID
- `len()`, `is_empty()` — Size queries
- `query_mac(&mac)`, `get_all_macs()` — MAC dimension
- `query_rssi(v)`, `query_rssi_range(min, max)`, `query_rssi_gt/gte/lt/lte(v)` — RSSI dimension
- `query_timestamp(ts)`, `query_time_range(start, end)`, `query_time_after/before(ts)` — Time dimension
- `query_geo_radius(lat, lon, radius_m)`, `query_geo_bbox(...)`, `query_geo_polygon(&[(lat, lon)])` — Geo dimension
- `query_multi(mac?, rssi_range?, time_range?, geo_center?)` — Cross-dimensional filtering

### Helper Functions (private)

- `haversine_distance(lat1, lon1, lat2, lon2)` — Haversine formula in meters
- `point_in_polygon(lat, lon, &polygon)` — Ray casting algorithm

## Testing

Unit tests live in `src/ble_cube.rs` under `#[cfg(test)] mod tests`:

- `test_basic_insert_and_query` — Insert + MAC query + len verification
- `test_rssi_range_query` — RSSI range and gte queries
- `test_geo_radius_query` — Geo radius at 10km and 20km distances

Run with: `cargo test`

Benchmarks are configured (criterion, bench name `cube_bench`) but the `benches/` directory does not yet exist. When created, the file should be `benches/cube_bench.rs`.

## Conventions

- **Rust edition:** 2021
- **Formatting:** Run `cargo fmt` before committing. Follow standard rustfmt defaults.
- **Linting:** Run `cargo clippy` and resolve all warnings before committing.
- **Testing:** Unit tests go in `#[cfg(test)]` modules within source files. Integration tests go in `tests/`.
- **Documentation:** Use `///` doc comments on all public items.
- **Error handling:** Prefer `Result` types and `thiserror`/`anyhow` over panics.
- **No unsafe code** unless absolutely necessary and well-justified.
- **Thread safety:** `BleCube` is not `Send`/`Sync` by default. Wrap in `Arc<RwLock<BleCube>>` for concurrent access.
- **Mutation testing:** `.gitignore` includes `mutants.out*/` for `cargo-mutants` output.
- **IDE:** `.idea/` is gitignored (JetBrains/RustRover).

## Known Issues / Notes

- The `Cargo.toml` configures a `[[bench]]` named `cube_bench` but the `benches/` directory does not yet exist.
- `query_multi()` uses `Vec::contains()` for set intersection, which is O(n) per check — could be optimized with `HashSet` for large result sets.
- Geo radius pre-filter uses a simple degree approximation (`radius_m / 111000.0`) that becomes less accurate at high latitudes.
- README states license as MIT, but the LICENSE file is Apache 2.0.
