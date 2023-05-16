use std::time::SystemTime;

use rug::{rand::RandState, Integer};

pub struct RandIntGenerator<'a>(RandState<'a>);

impl<'a> RandIntGenerator<'a> {
    pub fn new() -> Self {
        let mut rng = RandState::new();
        let cur = Integer::from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );
        rng.seed(&cur);
        Self(RandState::new())
    }

    pub fn randint(&mut self, bits: usize) -> Integer {
        Integer::from(Integer::random_bits(bits as u32, &mut self.0))
    }

    pub fn randodd(&mut self, bits: usize) -> Integer {
        loop {
            let x = self.randint(bits);
            if x.is_odd() {
                break x;
            }
        }
    }

    pub fn randrange(&mut self, bound: (&Integer, &Integer)) -> Integer {
        loop {
            let r = Integer::from(bound.1.random_below_ref(&mut self.0));
            if r.gt(bound.0) {
                break r;
            }
        }
    }
}
