use num_traits::One;
use rug::Integer;

pub fn inv_mod_2k(a: &Integer, k: u32) -> Integer {
    let mut x = Integer::ZERO;
    let mut b = Integer::one();
    for i in 0..k {
        let x_i = &b & Integer::one();
        if x_i == Integer::one() {
            x |= x_i << i;
            b = Integer::from(&b - a) >> 1;
        } else {
            b >>= 1;
        }
    }
    x
}

pub fn inv_mod_coprime(a: &Integer, p: &Integer) -> Integer {
    let mut j = 0;
    let mut z = Integer::ZERO;
    let m = Integer::from(p / a);
    let phi = Integer::from((a - Integer::one()) / a);
    while z != Integer::one() {
        z = a * (j * &m - (&phi - Integer::one())) - j * p;
        j += 1;
    }
    let i = j - 1;
    a * (i * m - phi + Integer::one())
}

#[cfg(test)]
mod tests {
    use num_traits::One;
    use rug::Integer;

    use crate::{
        inverse::{inv_mod_2k, inv_mod_coprime},
        prime::gen_prime,
        randint::{randint, randodd},
    };

    #[test]
    fn test_inv_mod_2k() {
        let base = Integer::one() << 1024;
        for _ in 0..1000 {
            let a = randodd(1024);
            let correct = Integer::from(a.invert_ref(&base).unwrap());
            let result = inv_mod_2k(&a, 1024);
            assert_eq!(result, correct);
        }
    }
    #[test]
    fn test_inv_mod_coprime() {
        let bits = 1024;
        let p = gen_prime(bits);
        for _ in 0..1000 {
            let a = randint(bits);
            let inv = inv_mod_coprime(&a, &p);
            assert_eq!(a * inv % &p, Integer::one());
        }
    }
}
