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

fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = value1;
    for i in 0..value0 {
        let mut tree = Tree {
            value: i,
            children: vec![],
        };
        let width = add(i, value1) % 33;
        for j in 0..width {
            tree.children.push(Tree {
                value: add(value1, j),
                children: vec![],
            });
        }
        if width > 0 {
            let removed = tree.children.remove(size(width / 2));
            sum = add(sum, removed.value);
        }
        sum = add(sum, total(&tree));
    }
    sum
}
fn main() {}
