use rand::{
    SeedableRng, TryRng,
    rngs::{StdRng, ThreadRng},
};

use ambassador::{Delegate, delegatable_trait_remote};

pub fn split_first<T>(mut array: Vec<T>) -> (T, Vec<T>) {
    assert!(!array.is_empty());

    let res = array.split_off(1);
    let first = array
        .pop()
        .expect("As array is not empty, always can get the first element");
    (first, res)
}

#[cfg(debug_assertions)]
pub fn debug_print(msg: &str) {
    println!("{msg}");
}

#[cfg(not(debug_assertions))]
pub fn debug_print(msg: &str) {}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! mydbg {
    ($($arg:tt)*) => {
        dbg!($($arg)*)
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! mydbg {
    ($($arg:tt)*) => {};
}

#[delegatable_trait_remote]
trait TryRng {
    type Error: core::error::Error;
    fn try_next_u32(&mut self) -> Result<u32, Self::Error>;
    fn try_next_u64(&mut self) -> Result<u64, Self::Error>;
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Delegate)]
#[delegate(TryRng)]
pub enum MyRng {
    StdRng(Box<StdRng>),
    ThreadRng(ThreadRng),
}

impl From<StdRng> for MyRng {
    fn from(rng: StdRng) -> Self {
        MyRng::StdRng(Box::new(rng))
    }
}

impl From<ThreadRng> for MyRng {
    fn from(rng: ThreadRng) -> Self {
        MyRng::ThreadRng(rng)
    }
}

pub fn gen_rng() -> MyRng {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        let seed: u64 = args[1].parse().unwrap();
        mydbg!(seed);
        StdRng::seed_from_u64(seed).into()
    } else {
        rand::rng().into()
    }
}
