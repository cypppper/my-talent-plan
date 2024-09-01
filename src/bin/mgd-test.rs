extern crate rayon;

use std::sync::Arc;
use std::{thread, time};

// #[test]
// fn main1() {
//     let pool = Arc::new(rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap());
//     let pool2 = Arc::new(rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap());
//     for x in 1..10 {
//         let pool2_c = pool2.clone();
//         pool.spawn(move || {
//             println!("pool1 start job {}", x);
//             pool2_c.install(move || {
//                 println!("pool2 start");
//                 thread::sleep(time::Duration::from_secs(1));
//                 println!("pool2 end");
//             });
//             println!("pool1 end job {}", x);
//         });
//         thread::sleep(time::Duration::from_millis(20));
//         println!("zzzz");
//     }
//     thread::sleep(time::Duration::from_secs(12));
// }

use std::time::Duration;
use std::time::Instant;
use std::thread::sleep;

fn work_barrier(ms: u64) -> () {
    println!("{}...", ms);
    let x = Instant::now();
    while x.elapsed() < Duration::from_millis(ms) {}
    println!("{}...Done", ms);
}

const SLOW_RAYON_ERROR: bool = false;

fn main() {
    let threadpool = Arc::new(rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap());

    let start = Instant::now();

    threadpool.install(|| {
        use rayon::prelude::*;
        let mut iter = Vec::new();
        let mut iter2 = Vec::new();
        for i in 0..100 {
            iter.push((i, || work_barrier(10)));
            iter2.push((i, || work_barrier(10)));
        }
        iter.into_par_iter().map(|(i, task)| {
            task();
            if i == 90 && SLOW_RAYON_ERROR {
                println!("Spawning task");
                threadpool.spawn(|| work_barrier(2000));
                threadpool.spawn(|| work_barrier(500));
                println!("Spawned tasks");
            }
            if i == 99 {
                println!("The first ITER end, at {}ms", start.elapsed().as_millis());
            }
        }).collect::<Vec<_>>();
        if !SLOW_RAYON_ERROR {
            threadpool.spawn(|| work_barrier(2000));
            threadpool.spawn(|| work_barrier(500));
        }
        println!("The first ITER finished, at {}ms", start.elapsed().as_millis());
        iter2.into_par_iter().map(|(i, task)| {
            if i == 0 {
                println!("The second ITER started");
            }
            task();
        }).collect::<Vec<_>>();

    });

    println!("wait for test finish");
    sleep(Duration::from_millis(2100));
    let d = start.elapsed();
    println!("==============");
    println!("Time Passed: {}ms", d.as_millis());
}