use bee2_bash::{Bash256, Bash384, Bash512, Hasher};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::{thread_rng, Rng};

fn bash_256(bytes: &[u8]) {
    let mut hash: [u8; 32] = [0;32];
    let mut basher = Bash256::new();
    basher.step_h(&bytes);
}

fn bash_384(bytes: &[u8]) {
    let mut hash: [u8; 48] = [0;48];
    let mut basher = Bash384::new();
    basher.step_h(&bytes);
}

fn bash_512(bytes: &[u8]) {
    let mut hash: [u8; 64] = [0;64];
    let mut basher = Bash512::new();
    basher.step_h(&bytes);
}


fn bench_bash(c: &mut Criterion) {
    let mut bytes: [u8; 1024] = [0; 1024];
    thread_rng().fill(&mut bytes[..]);

    let mut group = c.benchmark_group("bash 256");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("Bash 256", |b| b.iter(|| bash_256(&bytes)));
    group.finish();


    let mut group = c.benchmark_group("bash 384");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("Bash 384", |b| b.iter(|| bash_384(&bytes)));
    group.finish();

    let mut group = c.benchmark_group("bash 512");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("Bash 512", |b| b.iter(|| bash_512(&bytes)));
    group.finish();

}



criterion_group!(benches, bench_bash);
criterion_main!(benches);

// #![no_std]
// #![feature(test)]

// extern crate bee2_bash;
// extern crate test;

// #[cfg(test)]
// mod tests {
//     use bee2_bash::{Bash256, Hasher};
//     use rand::{thread_rng, Rng};
//     use test::{black_box, Bencher};

//     #[bench]
//     fn bench_bash256(b: &mut Bencher) {
//         let reps = 2000;
//         let mut buf: [u8; 1024] = [0; 1024];
//         thread_rng().fill(&mut buf[..]);
//         let mut hash: [u8; 32] = Default::default();

//         b.bytes = 1024 * 2000;
//         b.iter(|| {
//             let mut basher = Bash256::new();
//             for _x in 0..reps {
//                 basher.step_h(&buf);
//             }
//             basher.step_g(&mut hash);
//             // let mut basher = Bash256::new();
//             // for _x in 0..reps {
//             //     black_box(basher.step_h(&buf));
//             // }
//             // basher.step_g(&mut hash);

//             // // Inner closure, the actual test
//             // for i in 1..100 {
//             //     black_box(x.powf(y).powf(x));
//             // }
//         });
//         b.bytes = 1024;
//     }
// }

// #[test]
// fn hash_bench_bash256() {
//     use std::time::{SystemTime};
//     let now = SystemTime::now();

//     let reps = 2000;
//     let mut buf: [u8; 1024] = [0; 1024];
//     thread_rng().fill(&mut buf[..]);
//     let mut hash: [u8; 32] = Default::default();

//     let mut basher = Bash256::new();
//     for _x in 0..reps {
//         basher.step_h(&buf);
//     }
//     basher.step_g(&mut hash);

//     match now.elapsed() {
//         Ok(elapsed) => {
//             println!(
//                 "{}",
//                 reps as f64 / elapsed.as_nanos() as f64 * 1000000000 as f64
//             );
//         }
//         Err(e) => {
//             println!("Error: {:?}", e);
//         }
//     }
//     assert_eq!(1, 0);
// }

// #[test]
// fn hash_bench_bash384() {
//     use rand::{thread_rng, Rng};
//     use std::time::{SystemTime};
//     let now = SystemTime::now();

//     let reps = 2000;
//     let mut buf: [u8; 1024] = [0; 1024];
//     thread_rng().fill(&mut buf[..]);
//     let mut hash: [u8; 48] = [0; 48];

//     let mut basher = Bash384::new();
//     for _x in 0..reps {
//         basher.step_h(&buf);
//     }
//     basher.step_g(&mut hash);

//     match now.elapsed() {
//         Ok(elapsed) => {
//             println!(
//                 "{}",
//                 reps as f64 / elapsed.as_nanos() as f64 * 1000000000 as f64
//             );
//         }
//         Err(e) => {
//             println!("Error: {:?}", e);
//         }
//     }
//     assert_eq!(1, 0);
// }

// #[test]
// fn hash_bench_bash512() {
//     use rand::{thread_rng, Rng};
//     use std::time::{SystemTime};
//     let now = SystemTime::now();

//     let reps = 2000;
//     let mut buf: [u8; 1024] = [0; 1024];
//     thread_rng().fill(&mut buf[..]);
//     let mut hash: [u8; 64] = [0; 64];

//     let mut basher = Bash512::new();
//     for _x in 0..reps {
//         basher.step_h(&buf);
//     }
//     basher.step_g(&mut hash);

//     match now.elapsed() {
//         Ok(elapsed) => {
//             println!(
//                 "{}",
//                 reps as f64 / elapsed.as_nanos() as f64 * 1000000000 as f64
//             );
//         }
//         Err(e) => {
//             println!("Error: {:?}", e);
//         }
//     }
//     assert_eq!(1, 0);
// }
