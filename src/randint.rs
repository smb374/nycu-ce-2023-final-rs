use std::{cell::RefCell, rc::Rc, thread_local, time::SystemTime};

use rug::{rand::RandState, Integer};

thread_local! {
    pub static GLOBAL_RNG: Rc<RefCell<RandIntGenerator<'static>>> = Rc::new(RefCell::new(RandIntGenerator::new()));
}

pub struct RandIntGenerator<'a>(RandState<'a>);

impl<'a> RandIntGenerator<'a> {
    fn new() -> Self {
        let mut rng = RandState::new();
        let cur = Integer::from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );
        rng.seed(&cur);
        Self(rng)
    }
}

pub fn randint(bits: u32) -> Integer {
    GLOBAL_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        Integer::from(Integer::random_bits(bits, &mut rng.0))
    })
}

pub fn randodd(bits: u32) -> Integer {
    loop {
        let x = randint(bits);
        if x.is_odd() {
            break x;
        }
    }
}

pub fn randrange(bound: (&Integer, &Integer)) -> Integer {
    loop {
        let r = GLOBAL_RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            Integer::from(bound.1.random_below_ref(&mut rng.0))
        });
        if r.gt(bound.0) {
            break r;
        }
    }
}
