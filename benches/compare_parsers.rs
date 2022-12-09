use buffered_iterator::allocating_parser::AllocatingParser;
use buffered_iterator::buffered_parser::BufferedParser;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::slice;
use std::sync::Mutex;

lazy_static! {
    static ref SIZES: Vec<(u64, Mutex<()>)> = [10, 15, 20, 22, 24, 26, 28, 30]
        .into_iter()
        .map(|i| (1 << i, Mutex::new(())))
        .collect();
}

fn get_bench_file(size_index: usize) -> File {
    let (size, mutex) = SIZES.get(size_index).unwrap();
    let _lock = mutex.lock();
    let path = format!("data/{size}.bin");
    if let Ok(file) = File::open(&path) {
        assert_eq!(file.metadata().unwrap().size(), *size);
        file
    } else {
        let mut file = File::create(&path).unwrap();
        let mut bytes_written = 0;
        let mut rng = thread_rng();
        while bytes_written < *size {
            let max_length = (*size - bytes_written).min(u8::MAX.into()) as u8;
            let complete_length = Uniform::new(1, u8::MAX).sample(&mut rng).min(max_length);
            let data_length = complete_length - 1;
            file.write_all(slice::from_ref(&data_length)).unwrap();
            file.write_all(
                &Uniform::new(0, u8::MAX)
                    .sample_iter(&mut rng)
                    .take(data_length.into())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            bytes_written += u64::from(complete_length);
        }

        File::open(&path).unwrap()
    }
}

pub fn benchmark_streaming(c: &mut Criterion) {
    for size_index in 0..SIZES.len() {
        get_bench_file(size_index);
    }

    let mut group = c.benchmark_group("allocating_parser");
    for size_index in 0..SIZES.len() {
        let size = SIZES[size_index].0;
        let file = get_bench_file(size_index);
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &file, |b, file| {
            b.iter(|| {
                let parser = AllocatingParser::new(file);
                for entry in parser {
                    black_box(entry);
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("buffered_parser");
    for size_index in 0..SIZES.len() {
        let size = SIZES[size_index].0;
        let file = get_bench_file(size_index);
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &file, |b, file| {
            b.iter(|| {
                let parser = BufferedParser::new(file);
                for entry in parser {
                    let slice: &[u8] = &*entry;
                    black_box(slice);
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_streaming);
criterion_main!(benches);
