use rug::{rand::RandState, Integer};

pub struct RandIntGenerator<'a>(RandState<'a>);

impl<'a> RandIntGenerator<'a> {
    pub fn new() -> Self {
        Self(RandState::new())
    }

    pub fn randint(&mut self, bits: u32) -> Integer {
        Integer::from(Integer::random_bits(bits, &mut self.0))
    }

    pub fn randodd(&mut self, bits: u32) -> Integer {
        loop {
            let x = self.randint(bits);
            if x.is_odd() {
                break x;
            }
        }
    }
}
