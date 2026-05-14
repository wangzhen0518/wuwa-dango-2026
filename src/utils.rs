pub fn split_first<T>(mut array: Vec<T>) -> (T, Vec<T>) {
    assert!(!array.is_empty());

    let res = array.split_off(1);
    let first = array.pop().unwrap();
    (first, res)
}
