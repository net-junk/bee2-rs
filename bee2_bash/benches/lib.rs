extern crate criterion;

use bee2_bash::{bash_f0, Bash256, Bash384, Bash512};
use bee2_bash::{
    BashPrgAEAD2561, BashPrgAEAD2562, BashPrgAEAD3841, BashPrgAEAD3842, BashPrgAEAD5121,
    BashPrgAEAD5122,
};
use bee2_bash::{
    BashPrgHash2561, BashPrgHash2562, BashPrgHash3841, BashPrgHash3842, BashPrgHash5121,
    BashPrgHash5122,
};
use bee2_traits::{Hasher, PrgAEAD, PrgHasher};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::{thread_rng, Rng};

fn bash_256(bytes: &[u8]) {
    let mut basher = Bash256::new();
    basher.step_h(&bytes);
}

fn bash_384(bytes: &[u8]) {
    let mut basher = Bash384::new();
    basher.step_h(&bytes);
}

fn bash_512(bytes: &[u8]) {
    let mut basher = Bash512::new();
    basher.step_h(&bytes);
}

fn bashf_test(bytes: &mut [u64; 24]) {
    bash_f0(bytes)
}

fn bench_bash(c: &mut Criterion) {
    let mut bytes: [u8; 1024] = [0; 1024];
    thread_rng().fill(&mut bytes[..]);

    let mut group = c.benchmark_group("bashBench");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("bash256", |b| b.iter(|| bash_256(&bytes)));
    group.bench_function("bash384", |b| b.iter(|| bash_384(&bytes)));
    group.bench_function("bash512", |b| b.iter(|| bash_512(&bytes)));
    group.finish();

    let mut block: [u64; 24] = [0; 24];
    thread_rng().fill(&mut block[..]);

    let mut group = c.benchmark_group("primitiveBench");
    group.throughput(Throughput::Bytes((block.len() * 8) as u64));
    group.bench_function("f0", |b| b.iter(|| bashf_test(&mut block)));
    group.finish();

    // let mut group = c.benchmark_group("BasPrghBench");
    // group.throughput(Throughput::Bytes(bytes.len() as u64));
    // group.bench_function("bash-prg-hash2561", |b| b.iter(|| bash_256(&bytes)));
    // group.bench_function("bash-prg-hash2561", |b| b.iter(|| bash_384(&bytes)));
    // group.bench_function("bash-prg-hash2561", |b| b.iter(|| bash_256(&bytes)));
    // group.bench_function("bash-prg-hash2561", |b| b.iter(|| bash_384(&bytes)));
    // group.bench_function("bash-prg-hash2561", |b| b.iter(|| bash_256(&bytes)));
    // group.bench_function("bash-prg-hash2561", |b| b.iter(|| bash_384(&bytes)));
    // group.finish();
}

criterion_group!(benches, bench_bash);
criterion_main!(benches);
