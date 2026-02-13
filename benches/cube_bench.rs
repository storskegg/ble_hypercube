use criterion::{criterion_group, criterion_main, Criterion};
use ble_cube::{BleCube, BleObservation};

fn bench_insert(c: &mut Criterion) {
    c.bench_function("insert", |b| {
        b.iter(|| {
            let mut cube = BleCube::new();
            for i in 0..1000 {
                cube.insert(BleObservation {
                    rssi: -60,
                    mac: [0, 0, 0, 0, (i >> 8) as u8, i as u8],
                    timestamp: i as i64,
                    lat: 37.7749,
                    lon: -122.4194,
                });
            }
        });
    });
}

criterion_group!(benches, bench_insert);
criterion_main!(benches);
