pub fn split_first<T>(mut array: Vec<T>) -> (T, Vec<T>) {
    assert!(!array.is_empty());

    let res = array.split_off(1);
    let first = array
        .pop()
        .expect("As array is not empty, always can get the first element");
    (first, res)
}
