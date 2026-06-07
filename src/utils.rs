pub fn split_first<T>(mut array: Vec<T>) -> (T, Vec<T>) {
    assert!(!array.is_empty());

    let res = array.split_off(1);
    let first = array
        .pop()
        .expect("As array is not empty, always can get the first element");
    (first, res)
}

// #[cfg(debug_assertions)]
// pub fn debug_print(msg: &str) {
//     println!("{msg}");
// }

// #[cfg(not(debug_assertions))]
// pub fn debug_print(msg: &str) {}

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
