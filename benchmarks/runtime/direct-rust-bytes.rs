// Handwritten workload control flow over the same checked byte backend.
// Vec results retain the same rows as the matched Tsrust/TypeScript workloads.
fn construction(n: i64, seed: i64) -> Vec<Vec<i64>> {
    let mut output = Vec::new();
    for i in 0..n {
        let byte = (i + seed) % 256;
        let values = vec![0, byte, 255];
        let (first, second) = match bytes::from_array(&values) {
            Ok(value) => (value.len() as i64, value.len() as i64),
            Err(_) => (-2, -2),
        };
        let empty = match bytes::from_array(&[]) {
            Ok(value) => value.len() as i64,
            Err(_) => -2,
        };
        let invalid = if i % 2 == 0 { -1 } else { 256 };
        let (offset, code_length) = match bytes::from_array(&[byte, invalid, 0]) {
            Ok(value) => (-2, value.len() as i64),
            Err(error) => (error.offset.unwrap_or(-3), error.code.len() as i64),
        };
        output.push(vec![first, second, values[1], empty, offset, code_length]);
    }
    output
}

fn window(owner: &[u8], start: i64, end: i64) -> &[u8] {
    bytes::view(owner, start, end).unwrap_or_else(|error| panic!("{}", error.message))
}
fn first(value: &[u8]) -> i64 {
    bytes::at(value, 0).unwrap_or_else(|error| panic!("{}", error.message))
}
struct Picker<F> { pick: F }
fn make<'capture, 'argument>(captured: &'capture [u8], expected: i64)
    -> Picker<impl FnOnce(&'argument [u8]) -> &'argument [u8] + 'capture>
{
    Picker { pick: move |other: &'argument [u8]| {
        let kept = captured;
        assert_eq!(first(kept), expected, "capture mismatch");
        other
    } }
}
fn make_owner<'capture, 'argument>(captured: &'capture [u8], expected: i64)
    -> Picker<impl FnOnce(&'argument Vec<u8>) -> &'argument [u8] + 'capture>
{
    Picker { pick: move |other: &'argument Vec<u8>| {
        let kept = captured;
        assert_eq!(first(kept), expected, "capture mismatch");
        window(other, 1, 3)
    } }
}
fn view_callbacks(n: i64, seed: i64) -> Vec<Vec<i64>> {
    let mut output = Vec::new();
    for i in 0..n {
        let byte = (i + seed) % 256;
        let next = (byte + 1) % 256;
        let owner = bytes::from_array(&[byte, next, 255])
            .unwrap_or_else(|error| panic!("{}", error.message));
        let picker = make(window(&owner, 0, 1), byte);
        let selected = (picker.pick)(window(&owner, 1, 3));
        let borrowed = (make_owner(window(&owner, 0, 1), byte).pick)(&owner);
        output.push(vec![selected.len() as i64, first(selected), borrowed.len() as i64, first(borrowed)]);
    }
    output
}
