// Independently authored Rust scoped callback records and generic environments.
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
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
struct Holder<'a> {
    run: Box<dyn Fn() -> i64 + 'a>,
}
struct Step<F> {
    next: F,
}
struct Pair<'a> {
    left: Holder<'a>,
    right: Holder<'a>,
}
fn call(holder: &Holder<'_>) -> i64 {
    (holder.run)()
}
fn replace(holder: &mut Holder<'_>, amount: i64) {
    holder.run = Box::new(move || amount);
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let mut base = vec![value, add(value, 1)];
        let mut count = 0;
        let (first, stepped, both, replaced, moved) = {
            let mut holder = Holder {
                run: Box::new(|| base[0]),
            };
            let first = call(&holder);
            let mut step = Step {
                next: || {
                    count = add(count, value);
                    count
                },
            };
            (step.next)();
            let stepped = (step.next)();
            let pair = Pair {
                left: Holder {
                    run: Box::new(|| base[0]),
                },
                right: Holder {
                    run: Box::new(|| base[1]),
                },
            };
            let both = add((pair.left.run)(), call(&pair.right));
            replace(&mut holder, add(value, 7));
            let replaced = (holder.run)();
            holder.run = Box::new(|| mul(base[1], 2));
            let run = holder.run;
            let moved = run();
            (first, stepped, both, replaced, moved)
        };
        base[0] = 0;
        output.push(NumericArray(vec![
            first,
            stepped,
            both,
            replaced,
            moved,
            count,
            add(base[0], base[1]),
        ]));
    }
    Array(output)
}
fn main() {}
