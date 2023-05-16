use num_traits::One;
use rug::Integer;

use crate::{
    montgomery::{Montgomery, Residue},
    randint::RandIntGenerator,
};

const SMALL_PRIMES: [u32; 168] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
    311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421,
    431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541, 547,
    557, 563, 569, 571, 577, 587, 593, 599, 601, 607, 613, 617, 619, 631, 641, 643, 647, 653, 659,
    661, 673, 677, 683, 691, 701, 709, 719, 727, 733, 739, 743, 751, 757, 761, 769, 773, 787, 797,
    809, 811, 821, 823, 827, 829, 839, 853, 857, 859, 863, 877, 881, 883, 887, 907, 911, 919, 929,
    937, 941, 947, 953, 967, 971, 977, 983, 991, 997,
];

struct PrimeTestCtx<'a> {
    n: &'a Integer,
    n1: Integer,
    mont: &'a Montgomery,
    one: Residue<'a>,
    two: Residue<'a>,
    n1r: Residue<'a>,
}

impl<'a> PrimeTestCtx<'a> {
    fn new(n: &'a Integer, mont: &'a Montgomery) -> Self {
        let one = Residue::transform(Integer::one(), mont);
        let two = Residue::transform(Integer::from(2), mont);
        let n1 = n - Integer::one();
        let n1r = Residue::transform(n - Integer::one(), mont);
        Self {
            n,
            n1,
            mont,
            one,
            two,
            n1r,
        }
    }

    fn fermat(&self) -> bool {
        self.two.pow_mod(&self.n1) == self.one
    }

    fn miller_rabin(&self) -> bool {
        let mut r = 0;
        let mut d = self.n1.clone();
        let mut rng = RandIntGenerator::new();
        let n2 = self.n - Integer::from(2);
        let two = Integer::from(2);
        while d.is_even() {
            d >>= 1;
            r += 1;
        }
        'outer: for _ in 0..10 {
            let a = Residue::transform(rng.randrange((&two, &n2)), self.mont);
            let mut x = a.pow_mod(&d);
            if x == self.one || x == self.n1r {
                continue 'outer;
            }
            for _ in 0..(r - 1) {
                x = &x * &x;
                if x == self.n1r {
                    continue 'outer;
                }
            }
            return false;
        }
        true
    }
}

fn baille_psw(p: &Integer, bits: usize) -> bool {
    if p.eq(&Integer::ZERO) {
        false
    } else if p.is_even() {
        false
    } else {
        for sp in SMALL_PRIMES {
            if p.eq(&sp) {
                return true;
            } else if Integer::from(p % sp) == Integer::ZERO {
                return false;
            }
        }
        let mont = Montgomery::new(p.clone(), bits);
        let ctx = PrimeTestCtx::new(&p, &mont);
        ctx.fermat() && ctx.miller_rabin()
    }
}

pub fn gen_prime(bits: usize) -> Integer {
    let mut rng = RandIntGenerator::new();
    loop {
        let p = rng.randodd(bits);
        if baille_psw(&p, bits) {
            break p;
        }
    }
}
