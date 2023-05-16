mod montgomery;
mod prime;
mod randint;

use std::{hint::black_box, time::Instant};

use crate::prime::gen_prime;

fn main() {
    let bits = 1024;
    println!("{}-bit Prime generation...", bits);
    println!("Test: p = {}", gen_prime(bits));
    {
        let now = Instant::now();
        for _ in 0..100 {
            black_box(gen_prime(bits));
        }
        let elapsed = now.elapsed();
        println!(
            "Average runtime for gen_prime({}): {:.2?}",
            bits,
            elapsed / 100
        );
    }
}
