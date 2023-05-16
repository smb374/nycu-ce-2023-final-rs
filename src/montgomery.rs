use std::{fmt::Display, ops::Mul};

use num_traits::One;
use rug::Integer;

fn inv_mod_2k(a: &Integer, k: usize) -> Integer {
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
    bits: usize,
}

#[derive(Clone, Debug)]
pub struct Residue<'a> {
    x: Integer,
    mont: &'a Montgomery,
}

impl Montgomery {
    pub fn new(modulus: Integer, bits: usize) -> Self {
        assert!(modulus.is_odd());
        let r = Integer::one() << bits;
        let r2 = (Integer::one() << (bits << 1)) % &modulus;
        let r_mask = r.clone() - Integer::one();
        // let n_neg_inv: Integer = &r - Integer::from(modulus.invert_ref(&r).unwrap());
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
        let width = self.mont.bits / 4;
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
    use super::inv_mod_2k;
    use crate::randint::randodd;
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
}
