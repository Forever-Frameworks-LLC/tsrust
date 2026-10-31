// Independent standard Rust record ownership, recursion and retained arrays.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn size(n: i64) -> usize {
    usize::try_from(n).unwrap()
}
#[derive(Clone)]
struct Tree {
    value: i64,
    children: Vec<Tree>,
}
fn total(tree: &Tree) -> i64 {
    tree.children
        .iter()
        .fold(tree.value, |sum, child| add(sum, total(child)))
}
fn even(depth: i64, seed: i64) -> i64 {
    if depth == 0 {
        seed
    } else {
        add(depth, odd(sub(depth, 1), seed))
    }
}
fn odd(depth: i64, seed: i64) -> i64 {
    if depth == 0 {
        seed
    } else {
        add(depth, even(sub(depth, 1), seed))
    }
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = 0;
    for i in 0..value0 {
        let tree = Tree {
            value: i,
            children: vec![
                Tree {
                    value: value1,
                    children: vec![],
                },
                Tree {
                    value: add(i, 1),
                    children: vec![],
                },
            ],
        };
        sum = add(add(sum, total(&tree)), even(8, value1));
    }
    sum
}
fn main() {}
