use std::{fmt::Display, ops::Mul};

use num_traits::One;
use rug::Integer;

fn inv_mod_2k(a: &Integer, k: u32) -> Integer {
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

#[allow(dead_code)]
#[derive(Eq, Clone, Debug)]
pub struct Montgomery {
    n: Integer,
    n_neg_inv: Integer,
    r: Integer,
    r2: Integer,
    r_mask: Integer,
    bits: u32,
}

#[derive(Clone, Debug)]
pub struct Residue<'a> {
    x: Integer,
    mont: &'a Montgomery,
}

impl Montgomery {
    pub fn new(modulus: Integer, bits: u32) -> Self {
        let r = Integer::one() << bits;
        let r2 = (Integer::one() << (bits << 1)) % &modulus;
        let r_mask = r.clone() - Integer::one();
        let n_neg_inv: Integer = &r - inv_mod_2k(&modulus, bits);
        Self {
            n: modulus,
            n_neg_inv,
            r,
            r2,
            r_mask,
            bits,
        }
    }

    pub fn reduce(&self, t: &Integer) -> Integer {
        let m = (Integer::from(t & &self.r_mask) * &self.n_neg_inv) & &self.r_mask;
        let new_t = (t + m * &self.n) >> self.bits;
        if new_t >= self.n {
            new_t - &self.n
        } else {
            new_t
        }
    }

    pub fn one(&self) -> Residue {
        Residue::transform(Integer::one(), self)
    }

    // a (mod N) -> a^(-1) * R' (mod N), R' = 2^k, k in [n, 2n], n = MSB(p).
    fn alm_inverse(&self, a: &Integer) -> (Integer, u32) {
        let mut u = self.n.clone();
        let mut v = a.clone();
        let mut r = Integer::ZERO;
        let mut s = Integer::one();
        let mut k = 0;
        while v > 0 {
            if u.is_even() {
                u >>= 1;
                s <<= 1;
            } else if v.is_even() {
                v >>= 1;
                r <<= 1;
            } else if u > v {
                u = Integer::from(&u - &v) >> 1;
                r += &s;
                s <<= 1;
            } else {
                v = Integer::from(&v - &u) >> 1;
                s += &r;
                r <<= 1;
            }
            k += 1;
        }
        if r >= self.n {
            r -= &self.n;
        }
        (&self.n - r, k)
    }

    pub fn inverse(&self, a: &Integer) -> Integer {
        assert_eq!(Integer::from(a.gcd_ref(&self.n)), Integer::one());
        let (mut r, mut k) = self.alm_inverse(a);
        let m = self.bits;
        if k > m {
            r = self.reduce(&r);
            k -= m;
        }
        let res = r * (Integer::one() << (m - k));
        self.reduce(&res)
    }

    pub fn inverse_mod(&self, a: &Integer) -> Integer {
        let am = Integer::from(a % &self.n);
        self.inverse(&am)
    }
}

impl PartialEq for Montgomery {
    fn eq(&self, other: &Self) -> bool {
        self.n == other.n && self.bits == other.bits
    }
}

impl<'a> Residue<'a> {
    pub fn new(x: Integer, mont: &'a Montgomery) -> Self {
        Self { x, mont }
    }

    pub fn transform(x: Integer, mont: &'a Montgomery) -> Self {
        Self::new((x << mont.bits) % &mont.n, mont)
    }

    pub fn recover(&self) -> Integer {
        self.mont.reduce(&self.x)
    }

    pub fn inverse(&self) -> Self {
        let n = self.mont.n.significant_bits();
        let m = self.mont.bits;
        let (r_int, mut k) = self.mont.alm_inverse(&self.x);
        let mut r = Residue::new(r_int, self.mont);
        let r2 = Residue::new(self.mont.r2.clone(), self.mont);
        if k >= n && m >= k {
            r = &r * &r2;
            k += m;
        }
        let b = Residue::new(Integer::one() << (2 * m - k), self.mont);
        r = &r * &r2;
        r * b
    }

    pub fn pow_mod(&self, exp: &Integer) -> Self {
        let mut exp = exp.clone();
        let mut prod = Self::transform(Integer::one(), self.mont);
        let mut base = self.clone();
        while exp > 0 {
            if exp.is_odd() {
                prod = prod * &base;
            }
            exp >>= 1;
            base = &base * &base;
        }
        prod
    }
}

impl<'a> PartialEq for Residue<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.mont == other.mont && self.x == other.x
    }
}

impl<'a> Mul for Residue<'a> {
    type Output = Residue<'a>;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.mont, rhs.mont);
        let new_x = self.x * &rhs.x;
        Self::new(self.mont.reduce(&new_x), self.mont)
    }
}

impl<'a, 'b> Mul for &'b Residue<'a> {
    type Output = Residue<'a>;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.mont, rhs.mont);
        let new_x = Integer::from(&self.x * &rhs.x);
        Residue::new(self.mont.reduce(&new_x), self.mont)
    }
}

impl<'a, 'b> Mul<Residue<'a>> for &'b Residue<'a> {
    type Output = Residue<'a>;
    fn mul(self, rhs: Residue<'a>) -> Self::Output {
        assert_eq!(self.mont, rhs.mont);
        let new_x = Integer::from(&self.x * &rhs.x);
        Residue::new(self.mont.reduce(&new_x), self.mont)
    }
}

impl<'a, 'b> Mul<&'b Residue<'a>> for Residue<'a> {
    type Output = Residue<'a>;
    fn mul(self, rhs: &'b Residue<'a>) -> Self::Output {
        assert_eq!(self.mont, rhs.mont);
        let new_x = self.x * &rhs.x;
        Residue::new(self.mont.reduce(&new_x), self.mont)
    }
}

impl<'a> Display for Residue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = (self.mont.bits / 4) as usize;
        write!(
            f,
            "MontgomeryResidue({:0width$x} mod {:0width$x})",
            self.x, self.mont.n,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{inv_mod_2k, Montgomery, Residue};
    use crate::{
        prime::gen_prime,
        randint::{randint, randodd},
    };
    use num_traits::One;
    use rug::Integer;
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
    fn test_inverse_mod() {
        let bits = 1024;
        let p = gen_prime(bits);
        let mont = Montgomery::new(p.clone(), bits);
        for _ in 0..1000 {
            let a = randint(bits);
            let inv = mont.inverse_mod(&a);
            assert_eq!((a * inv) % &p, Integer::one());
        }
    }
    #[test]
    fn test_residue_inv() {
        let bits = 1024;
        let p = gen_prime(bits);
        let mont = Montgomery::new(p, bits);
        let one = Residue::transform(Integer::one(), &mont);
        for _ in 0..1000 {
            let a = Residue::transform(randint(bits), &mont);
            let inv = a.inverse();
            assert_eq!(a * inv, one);
        }
    }
}
