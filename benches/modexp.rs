use criterion::{black_box, criterion_group, criterion_main, Criterion};
use num_traits::{FromPrimitive, One};
use rug::Integer;
use simple_rsa::{
    montgomery::{Montgomery, Residue},
    prime::gen_prime,
};

fn naive_modexp(x: Integer, y: Integer, p: Integer) -> Integer {
    let mut result = Integer::one();
    let mut base = x;
    let mut exp = y;
    while exp > 0 {
        if exp.is_odd() {
            result = (result * &base) % &p;
        }
        base = Integer::from(&base * &base);
        exp >>= 1;
    }
    result % p
}

fn mont_modexp(x: Integer, y: Integer, mont: &Montgomery) -> Integer {
    let xr = Residue::new(x, mont);
    let r = xr.pow_mod(&y);
    r.recover()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("mod pow 1023^65535 % 65537 naive", |b| {
        b.iter(|| {
            naive_modexp(
                black_box(Integer::from_u64(1023).unwrap()),
                black_box(Integer::from_u64(65535).unwrap()),
                black_box(Integer::from_u64(65537).unwrap()),
            )
        })
    });
    c.bench_function("mod pow 1023^65535 % 65537 montgomery", |b| {
        let mont = Montgomery::new(Integer::from_u64(65537).unwrap(), 64);
        b.iter(|| {
            mont_modexp(
                black_box(Integer::from_u64(1023).unwrap()),
                black_box(Integer::from_u64(65535).unwrap()),
                black_box(&mont),
            )
        })
    });
    c.bench_function("mod pow 256-bit montgomnery", |b| {
        let p = Integer::from_str_radix(
            "165e5ebdf8f41ae6ffe2eb4cef8f0406012baa0e6f3a7eb421c6b3842873f61b",
            16,
        )
        .unwrap();
        let x = Integer::from_str_radix(
            "d777c0340aa15828e258119b10b3ad02434c9866567cbca696100b1d64e73c74",
            16,
        )
        .unwrap();
        let y = Integer::from_str_radix(
            "9aa88f28743eb67ad2bc8927fcb5e671fc5b08ccfb944c2c24a393776ee5840f",
            16,
        )
        .unwrap();
        let mont = Montgomery::new(p, 256);
        b.iter(|| {
            mont_modexp(black_box(x.clone()), black_box(y.clone()), &mont);
        });
    });
    c.bench_function("generate prime 256-bit", |b| {
        b.iter(|| gen_prime(black_box(256)))
    });
    c.bench_function("generate prime 512-bit", |b| {
        b.iter(|| gen_prime(black_box(512)))
    });
    c.bench_function("generate prime 1024-bit", |b| {
        b.iter(|| gen_prime(black_box(1024)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
