use criterion::{black_box, criterion_group, criterion_main, Criterion};
use weird::Weird;

fn decode_benchmark(c: &mut Criterion) {
    let weird = Weird::from_salt("Hello, world!");

    c.bench_function("crockford fzq", |b| {
        b.iter(|| crockford::decode(black_box("fzq")))
    });

    c.bench_function("weird fzq", |b| b.iter(|| weird.decode(black_box("fzq"))));

    c.bench_function("crockford fzq upper", |b| {
        b.iter(|| crockford::decode(black_box("FZQ")))
    });

    c.bench_function("weird fzq upper", |b| {
        b.iter(|| weird.decode(black_box("FZQ")))
    });

    c.bench_function("crockford fzzlong", |b| {
        b.iter(|| crockford::decode(black_box("fzzzzzzzzzzzz")))
    });

    c.bench_function("weird fzzlong", |b| {
        b.iter(|| weird.decode(black_box("fzzzzzzzzzzzz")))
    });

    c.bench_function("crockford fzzlong upper", |b| {
        b.iter(|| crockford::decode(black_box("FZZZZZZZZZZZZ")))
    });

    c.bench_function("weird fzzlong upper", |b| {
        b.iter(|| weird.decode(black_box("FZZZZZZZZZZZZ")))
    });
}

criterion_group!(decode, decode_benchmark);

criterion_main!(decode);
