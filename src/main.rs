mod montgomery;
mod randint;

use std::{hint::black_box, time::Instant};

use crate::{
    montgomery::{Montgomery, Residue},
    randint::RandIntGenerator,
};

fn main() {
    let mut rng = RandIntGenerator::new();
    let m = rng.randodd(512);
    let x1 = rng.randint(512);
    let x2 = rng.randint(512);
    println!("m = {:x}, x1 = {:x}, x2 = {:x}", &m, &x1, &x2);
    let mont = Montgomery::new(m, 512);
    let x1r = Residue::transform(x1, &mont);
    println!("x1 ^ x2 (mod m) = {} = {}", x1r, x1r.pow_mod(&x2).recover());
    {
        let now = Instant::now();
        for _ in 0..10000 {
            let res = black_box(x1r.pow_mod(&x2));
            black_box(res.recover());
        }
        let elapsed = now.elapsed();
        println!(
            "Average runtime for Residue::pow_mod using x1^x2 mod m: {:.2?}",
            elapsed / 10000
        );
    }
}
