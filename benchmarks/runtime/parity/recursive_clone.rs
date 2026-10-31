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
fn project(tree: &Tree, row: &mut Vec<i64>) {
    row.push(tree.value);
    for child in &tree.children {
        project(child, row);
    }
}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {
    let mut roots = Vec::new();
    for i in 0..value0 {
        let mut children = Vec::new();
        for j in 0..add(i, value1) % 5 {
            children.push(Tree {
                value: add(add(mul(3, i), j), value1) % 1009,
                children: vec![],
            });
        }
        let mut tree = Tree {
            value: i % 1009,
            children,
        };
        for depth in 0..add(i, value1) % 4 {
            tree = Tree {
                value: add(add(i, depth), value1) % 1009,
                children: vec![tree],
            };
        }
        roots.push(tree);
    }
    let copied = roots.clone();
    for (index, root) in roots.iter_mut().enumerate() {
        root.value = add(i64::try_from(index).unwrap(), 2000);
        if !root.children.is_empty() {
            root.children[0].value = add(value1, 3000);
        }
    }
    let mut output = Vec::new();
    for tree in copied.iter().chain(roots.iter()) {
        let mut row = Vec::new();
        project(tree, &mut row);
        output.push(NumericArray(row));
    }
    Array(output)
}
fn main() {}
