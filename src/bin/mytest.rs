#![allow(warnings)]

use std::{cell::RefCell, collections::VecDeque, ops::DerefMut, rc::Rc};

use rand::seq::SliceRandom;

struct MyStruct {
    inner: usize,
}

mod A {
    pub mod B {
        macro_rules! my_macro {
            () => {
                println!("hello");
            };
        }

        pub(in crate::A::B) use my_macro;

        pub mod C {
            use super::my_macro;

            fn f() {
                my_macro!();
            }
        }
    }
}

fn main() {
    let x = RefCell::new(123);
    {
        let mut y = x.borrow_mut();
        *y += 1;
        dbg!(&y);
    }
    dbg!(&x);

    let x = [1, 2, 3];
    let y = x[2..].iter();
    for yi in y {
        dbg!(yi);
    }

    for i in 0..1 {
        dbg!(i);
    }

    let mut x = Rc::new(RefCell::new(MyStruct { inner: 123 }));
    let y = x.as_ref();
    let mut z = x.borrow_mut();
    let a = z.deref_mut();

    let mut b = [(1, 2), (4, 3), (2, 1), (3, 4), (2, 3)];
    b.sort();
    dbg!(&b);

    let mut c = VecDeque::from([1, 3, 2]);
    c.make_contiguous().shuffle(&mut rand::rng());

    // c.make_contiguous().spltmu
}
