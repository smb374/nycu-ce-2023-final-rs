use std::{fmt::Display, ops::Mul};

use num_traits::One;
use rug::Integer;

use crate::inverse::inv_mod_2k;

#[derive(Eq, Clone, Debug)]
pub struct Montgomery {
    n: Integer,
    n_neg_inv: Integer,
    r2: Integer,
    r_mask: Integer,
    bits: u32,
}

#[derive(Clone, Debug)]
pub struct Residue<'a> {
    x: Integer,
    mont: &'a Montgomery,
}

fn compute_r2(modulus: &Integer, bits: u32) -> Integer {
    let mut i = 0;
    let mut c = Integer::one() << (bits - 1); // c_0
    while i < bits + 1 {
        c = Integer::from(&c + &c) % modulus; // c_i
        i += 1;
    }
    c
}

impl Montgomery {
    pub fn new(modulus: Integer, bits: u32) -> Self {
        let r = Integer::one() << bits;
        let r2 = compute_r2(&modulus, bits);
        let r_mask = &r - Integer::one();
        let n_neg_inv: Integer = &r - inv_mod_2k(&modulus, bits);
        Self {
            n: modulus,
            n_neg_inv,
            r2,
            r_mask,
            bits,
        }
    }

    pub fn reduce(&self, t: Integer) -> Integer {
        let m = (Integer::from(&t & &self.r_mask) * &self.n_neg_inv) & &self.r_mask;
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

    // From https://www.researchgate.net/publication/3387259_Improved_Montgomery_modular_inverse_algorithm
    fn u_inv(&self, a: &Integer, is_mont_form: bool) -> Integer {
        let m = self.bits;
        let (mut r, k) = self.alm_inverse(a);
        if k != m {
            // correction if k != m
            let r_cor = r * (Integer::one() << ((m << 1) - k));
            r = self.reduce(r_cor);
        }
        if is_mont_form {
            self.reduce(r * &self.r2)
        } else {
            self.reduce(r)
        }
    }

    pub fn inverse(&self, a: &Integer) -> Integer {
        self.u_inv(a, false)
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

    pub fn from_mont_integer(x: Integer, mont: &'a Montgomery) -> Self {
        Self::new(x, mont)
    }

    pub fn to_mont_integer(self) -> Integer {
        self.x
    }

    pub fn transform(x: Integer, mont: &'a Montgomery) -> Self {
        Self::new(mont.reduce(x * &mont.r2), mont)
    }

    pub fn recover(&self) -> Integer {
        self.mont.reduce(self.x.clone())
    }

    pub fn inverse(&self) -> Self {
        let new_x = self.mont.u_inv(&self.x, true);
        Self::new(new_x, self.mont)
    }

    pub fn pow_mod(&self, exp: &Integer) -> Self {
        let mut a = Self::transform(Integer::one(), self.mont);
        let mut t = exp.significant_bits() - 1;
        while t != u32::MAX {
            a = &a * &a;
            if exp.get_bit(t) {
                a = &a * self;
            }
            t = t.wrapping_sub(1);
        }
        a
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
        Self::new(self.mont.reduce(new_x), self.mont)
    }
}

impl<'a, 'b> Mul for &'b Residue<'a> {
    type Output = Residue<'a>;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.mont, rhs.mont);
        let new_x = Integer::from(&self.x * &rhs.x);
        Residue::new(self.mont.reduce(new_x), self.mont)
    }
}

impl<'a, 'b> Mul<Residue<'a>> for &'b Residue<'a> {
    type Output = Residue<'a>;
    fn mul(self, rhs: Residue<'a>) -> Self::Output {
        assert_eq!(self.mont, rhs.mont);
        let new_x = Integer::from(&self.x * &rhs.x);
        Residue::new(self.mont.reduce(new_x), self.mont)
    }
}

impl<'a, 'b> Mul<&'b Residue<'a>> for Residue<'a> {
    type Output = Residue<'a>;
    fn mul(self, rhs: &'b Residue<'a>) -> Self::Output {
        assert_eq!(self.mont, rhs.mont);
        let new_x = self.x * &rhs.x;
        Residue::new(self.mont.reduce(new_x), self.mont)
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
    use super::{compute_r2, Montgomery, Residue};
    use crate::randint::randint;
    use num_traits::One;
    use rug::Integer;
    #[test]
    fn test_inverse_mod() {
        let bits = 1024;
        let p: Integer = Integer::from_str_radix("eb3799588212706009a73cdd7af5a70e30338c1cb8dd13ce9b21a7af003a634c187f14c512ff10cee428549d04097e5417f32ed62904529362653399e09ff1d4dd9c2c043c140bd45d1a694e5d2adbe3cf9072fe7535fe91cc67c070e8087ad0d2c19f5eb1abf06e4d1b28f71d8063fff88576cfb0d38a7f53a590ae913f626b", 16).unwrap();
        let mont = Montgomery::new(p.clone(), bits);
        for _ in 0..1000 {
            let a = randint(bits);
            let inv = mont.inverse(&a);
            assert_eq!((a * inv) % &p, Integer::one());
        }
    }
    #[test]
    fn test_residue_inv() {
        let bits = 1024;
        let p: Integer = Integer::from_str_radix("eb3799588212706009a73cdd7af5a70e30338c1cb8dd13ce9b21a7af003a634c187f14c512ff10cee428549d04097e5417f32ed62904529362653399e09ff1d4dd9c2c043c140bd45d1a694e5d2adbe3cf9072fe7535fe91cc67c070e8087ad0d2c19f5eb1abf06e4d1b28f71d8063fff88576cfb0d38a7f53a590ae913f626b", 16).unwrap();
        let mont = Montgomery::new(p, bits);
        let one = Residue::transform(Integer::one(), &mont);
        for _ in 0..1000 {
            let a = Residue::transform(randint(bits), &mont);
            let inv = a.inverse();
            assert_eq!(a * inv, one);
        }
    }
    #[test]
    fn test_compute_r2() {
        let bits = 1024;
        let p: Integer = Integer::from_str_radix("eb3799588212706009a73cdd7af5a70e30338c1cb8dd13ce9b21a7af003a634c187f14c512ff10cee428549d04097e5417f32ed62904529362653399e09ff1d4dd9c2c043c140bd45d1a694e5d2adbe3cf9072fe7535fe91cc67c070e8087ad0d2c19f5eb1abf06e4d1b28f71d8063fff88576cfb0d38a7f53a590ae913f626b", 16).unwrap();
        let r2 = compute_r2(&p, bits);
        assert_eq!(r2, (Integer::one() << (bits << 1)) % p);
    }
    #[test]
    fn test_transform() {
        let bits = 1024;
        let p: Integer = Integer::from_str_radix("eb3799588212706009a73cdd7af5a70e30338c1cb8dd13ce9b21a7af003a634c187f14c512ff10cee428549d04097e5417f32ed62904529362653399e09ff1d4dd9c2c043c140bd45d1a694e5d2adbe3cf9072fe7535fe91cc67c070e8087ad0d2c19f5eb1abf06e4d1b28f71d8063fff88576cfb0d38a7f53a590ae913f626b", 16).unwrap();
        let mont = Montgomery::new(p, bits);
        for _ in 0..1000 {
            let x = randint(bits);
            let trx = Integer::from(&x * &mont.r2);
            assert_eq!(mont.reduce(trx), (x << bits) % &mont.n);
        }
    }
    #[test]
    fn test_mul() {
        let bits = 1024;
        let p: Integer = Integer::from_str_radix("eb3799588212706009a73cdd7af5a70e30338c1cb8dd13ce9b21a7af003a634c187f14c512ff10cee428549d04097e5417f32ed62904529362653399e09ff1d4dd9c2c043c140bd45d1a694e5d2adbe3cf9072fe7535fe91cc67c070e8087ad0d2c19f5eb1abf06e4d1b28f71d8063fff88576cfb0d38a7f53a590ae913f626b", 16).unwrap();
        let mont = Montgomery::new(p.clone(), bits);
        for _ in 0..1000 {
            let x = randint(bits);
            let y = randint(bits);
            let corr = Integer::from(&x * &y) % &p;
            let xm = Residue::transform(x, &mont);
            let ym = Residue::transform(y, &mont);
            let res = xm * ym;
            assert_eq!(res.recover(), corr);
        }
    }
    #[test]
    fn test_pow_mod() {
        let bits = 1024;
        let p: Integer = Integer::from_str_radix("eb3799588212706009a73cdd7af5a70e30338c1cb8dd13ce9b21a7af003a634c187f14c512ff10cee428549d04097e5417f32ed62904529362653399e09ff1d4dd9c2c043c140bd45d1a694e5d2adbe3cf9072fe7535fe91cc67c070e8087ad0d2c19f5eb1abf06e4d1b28f71d8063fff88576cfb0d38a7f53a590ae913f626b", 16).unwrap();
        let mont = Montgomery::new(p.clone(), bits);
        for _ in 0..1000 {
            let x = randint(bits);
            let y = randint(bits >> 1);
            let corr = Integer::from(x.pow_mod_ref(&y, &p).unwrap());
            let xm = Residue::transform(x, &mont);
            let res = xm.pow_mod(&y);
            assert_eq!(res.recover(), corr);
        }
    }
}
