use criterion::{black_box, criterion_group, criterion_main, Criterion};
use weird::Weird;

fn encode_benchmark(c: &mut Criterion) {
    let weird = Weird::from_salt("Hello, world!");
    
    c.bench_function("crockford 5111", |b| {
        b.iter(|| crockford::encode(black_box(5111)))
    });

    c.bench_function("weird 5111", |b| {
        b.iter(|| weird.encode(black_box(5111)))
    });

    c.bench_function("crockford 184long", |b| {
        b.iter(|| crockford::encode(black_box(18446744073709551615)))
    });

    c.bench_function("weird 184long", |b| {
        b.iter(|| weird.encode(black_box(18446744073709551615)))
    });

    c.bench_function("crockford into 5111", |b| {
        let mut buffer = String::with_capacity(13);
        b.iter(|| {
            buffer.clear();
            crockford::encode_into(black_box(5111), &mut buffer);
        })
    });

    c.bench_function("weird into 5111", |b| {
        let mut buffer = String::with_capacity(13);
        b.iter(|| {
            buffer.clear();
            weird.encode_into(black_box(5111), &mut buffer).unwrap();
        })
    });

    c.bench_function("crockford into 184long", |b| {
        let mut buffer = String::with_capacity(13);
        b.iter(|| {
            buffer.clear();
            crockford::encode_into(black_box(18446744073709551615), &mut buffer);
        })
    });

    c.bench_function("weird into 184long", |b| {
        let mut buffer = String::with_capacity(13);
        b.iter(|| {
            buffer.clear();
            weird.encode_into(black_box(18446744073709551615), &mut buffer).unwrap();
        })
    });
}

criterion_group!(encode, encode_benchmark);

criterion_main!(encode);
