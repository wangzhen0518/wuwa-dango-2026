#![allow(warnings)]

use std::{cell::RefCell, ops::DerefMut, rc::Rc};

struct MyStruct {
    inner: usize,
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
}
