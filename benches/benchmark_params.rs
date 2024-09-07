use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use lazy_static::lazy_static;


use std::{clone, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, sync::Arc, thread::{self, sleep}, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use kvs::{thread_pool::{SharedQueueThreadPool, ThreadPool}, KvStore, KvsClient, KvsEngine, KvsServer};
use tempfile::TempDir;

const WRITE_CNT: usize = 10;
lazy_static! (
    static ref PAIRS: Vec<(String, String)> = {
        (0..WRITE_CNT).into_iter()
            .map(|idx| {
                (format!("key_{:05}", idx), String::from("value"))
            })
            .collect()
    };
    static ref ADDR:SocketAddr = "127.0.0.1:4005".parse().unwrap();
    static ref CLI: KvsClient = KvsClient::new();
    static ref CH_PAIR: (crossbeam_channel::Sender<()>, crossbeam_channel::Receiver<()>) = unbounded::<()>();
);
// fn insert_pairs() {
fn insert_pairs<P: ThreadPool>(cli_pool: &P, cli: &'static KvsClient, addr: SocketAddr) {
    let sender: &'static Sender<()> = &CH_PAIR.0;
    
    // let pairs = vec![(format!("1"), format!("value")), (format!("2"), format!("value")), (format!("3"), format!("value"))];
    PAIRS.iter()
        .map(|(key, value)| {
            cli_pool.spawn(move || {
                cli.set(key, value, addr.clone());
                sender.send(()).unwrap();
            });
        })
        .for_each(drop);

}

fn bench_params(c: &mut Criterion) {
    let inputs = &[1u32, 2];
    let mut group = c.benchmark_group("my first group");
    for input in inputs {
        let param_string = format!("{}", input);
        let addr: SocketAddr = "127.0.0.1:4015".parse().unwrap();
        group.bench_with_input(BenchmarkId::new("params",param_string), 
            input, 
            |b, param| {
                // let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 4010u16 + (*param) as u16));
                let kv_server = KvsServer::new(
                    addr.clone(),
                    KvStore::open(TempDir::new().unwrap().into_path()).unwrap(),
                    SharedQueueThreadPool::new(*param).unwrap(),
                );
                let end_handle = thread::spawn(move || {
                    kv_server.start();
                });
                sleep(Duration::from_millis(100));
                
                let pool = SharedQueueThreadPool::new(4).unwrap();
                b.iter(|| {
                    insert_pairs(&pool, &CLI, addr.clone());
                    // insert_pairs();
                    let recver: &'static Receiver<()> = &CH_PAIR.1;
                    for _ in 0..WRITE_CNT {
                        recver.recv().unwrap();
                    }
                });
                CLI.shutdown(addr.clone());
                end_handle.join().unwrap();
            }
        );
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_params
);
criterion_main!(benches);