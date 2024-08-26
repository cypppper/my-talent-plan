use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use kvs::{KvsEngine, SledStore, KvStore};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tempfile::TempDir;


#[inline]
fn gen_pairs() -> Vec<(String, String)> {
    let mut rng = rand::thread_rng();
    let mut ret = vec![];
    for _ in 0..100 {
        let key_len: usize = rng.gen_range(1, 10_0001);
        let key: String = rng
            .sample_iter(&Alphanumeric)
            .take(key_len)
            .map(char::from)
            .collect();
        let value_len: usize = rng.gen_range(1, 10_0001);
        let value: String = rng
            .sample_iter(&Alphanumeric)
            .take(value_len)
            .map(char::from)
            .collect();
        ret.push((key, value));
    }
    ret
}

#[inline]
fn engine_write_kvs() {
    let temp_dir_kvs = TempDir::new().unwrap();
    let mut engine = KvStore::open(temp_dir_kvs.path()).unwrap();
    let pairs = gen_pairs();
    for (key, value) in pairs {
        engine.set(key, value).unwrap();
    }
}

#[inline]
fn engine_write_sled() {
    let temp_dir_sled = TempDir::new().unwrap();
    let mut engine = SledStore::open(temp_dir_sled.path()).unwrap();
    let pairs = gen_pairs();
    for (key, value) in pairs {
        engine.set(key, value).unwrap();
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("kvs rand write", |b| b.iter(
        || engine_write_kvs()
    ));
    c.bench_function("sled rand write", |b| b.iter(
        || engine_write_sled()
    ));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);


