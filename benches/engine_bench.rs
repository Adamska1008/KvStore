use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rand::Rng;
use tempfile::tempdir;
use kvs::engine::{KvsEngine, Sled};
use kvs::KvStore;

pub fn set_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_bench");
    group.bench_function("kvs", |b| {
        b.iter_batched(
            || {
                let tmp_dir = tempdir().unwrap();
                (KvStore::open(tmp_dir.path()).unwrap(), tmp_dir)
            },
            |(mut store, tmp_dir)| {
                for i in 1..(1 << 12) {
                    store.set(&format!("key{}", i), "value").unwrap()
                }
            },
            BatchSize::SmallInput
        )
    });
    group.bench_function("sled", |b| {
        b.iter_batched(
            || {
                let tmp_dir = tempdir().unwrap();
                (Sled::new(sled::open(&tmp_dir).unwrap()), tmp_dir)
            },
            |(mut db, tmp_dir)| {
                for i in 1..(1 << 12) {
                    db.set(&format!("key{}", i), "value").unwrap()
                }
            },
            BatchSize::SmallInput
        )
    });
}

pub fn get_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_bench");
    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("kvs_{}", i), i, |b, i| {
            let temp_dir = tempdir().unwrap();
            let mut store = KvStore::open(temp_dir.path()).unwrap();
            for key_i in 1..(1 << i) {
                store
                    .set(&format!("key{}", key_i), "value")
                    .unwrap();
            }
            let mut rng = rand::thread_rng();
            b.iter(|| {
                store
                    .get(&format!("key{}", rng.gen_range(1..1 << i)))
                    .unwrap();
            })
        });
    }
    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("sled_{}", i), i, |b, i| {
            let temp_dir = tempdir().unwrap();
            let mut db = Sled::new(sled::open(&temp_dir).unwrap());
            for key_i in 1..(1 << i) {
                db.set(&format!("key{}", key_i), "value")
                    .unwrap();
            }
            let mut rng = rand::thread_rng();
            b.iter(|| {
                db.get(&format!("key{}", rng.gen_range(1..1 << i))).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, set_bench, get_bench);
criterion_main!(benches);