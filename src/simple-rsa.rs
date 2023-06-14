use num_traits::One;
use rug::Integer;
use simple_rsa::prime::gen_prime;

fn main() {
    let bits = 1024;
    let p = gen_prime(bits);
    let q = gen_prime(bits);
    let n = Integer::from(&p * &q);
    let r = (&p - Integer::one()) * (&q - Integer::one());
    let e = Integer::from(65537);
    let d = Integer::from(e.invert_ref(&r).unwrap());
    println!("N = {:x}", n);
    println!("e = {:x}", e);
    println!("d = {:x}", d);
}
