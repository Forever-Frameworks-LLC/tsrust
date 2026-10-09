//! B7 witness: one generic stored origin must accept distinct input environments.
mod support;

#[test]
fn replacing_a_borrowed_callback_accepts_a_different_environment_type() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = (): int => action();
        }
        function main() {
            const values = [3];
            const first = (): int => values[0];
            const second = (): int => 9;
            const holder: Holder = { run: (): int => 0 };
            install(holder, first);
            strictEqual(holder.run(), 3);
            install(holder, second);
            strictEqual(holder.run(), 9);
        }
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn replacing_a_mutable_callback_preserves_each_input_state_and_releases_the_holder() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = (): int => action();
        }
        function main() {
            let count = 1;
            const first = (): int => { count += 1; return count; };
            const values = [10];
            const second = (): int => { values[0] += 2; return values[0]; };
            {
                const holder: Holder = { run: (): int => 0 };
                install(holder, first);
                strictEqual(holder.run(), 2);
                install(holder, second);
                strictEqual(holder.run(), 12);
                strictEqual(holder.run(), 14);
            }
            strictEqual(first(), 3);
            strictEqual(second(), 16);
        }
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

/// Backend representation witness. One wrapper origin can have multiple concrete
/// environment arms without changing the source contract or allocating a box.
#[test]
fn native_alternatives_preserve_one_wrapper_origin_and_distinct_capture_types() {
    let directory = support::TempDirectory::new();
    let source = directory.0.join("alternatives.rs");
    let executable = directory.0.join("alternatives");
    std::fs::write(&source, r#"
        trait Callback { fn call(&mut self) -> i64; }
        struct Counter<'a>(&'a mut i64);
        impl Callback for Counter<'_> {
            fn call(&mut self) -> i64 { *self.0 += 1; *self.0 }
        }
        struct ArrayCounter<'a>(&'a mut Vec<i64>);
        impl Callback for ArrayCounter<'_> {
            fn call(&mut self) -> i64 { self.0[0] += 2; self.0[0] }
        }
        struct Wrapper<'a, F>(&'a mut F);
        impl<F: Callback> Callback for Wrapper<'_, F> {
            fn call(&mut self) -> i64 { self.0.call() }
        }
        enum Stored<'a, 'b, 'c> {
            Initial,
            First(Wrapper<'a, Counter<'b>>),
            Second(Wrapper<'a, ArrayCounter<'c>>),
        }
        impl Callback for Stored<'_, '_, '_> {
            fn call(&mut self) -> i64 {
                match self {
                    Self::Initial => 0,
                    Self::First(value) => value.call(),
                    Self::Second(value) => value.call(),
                }
            }
        }
        impl<'a, 'b, 'c> From<Wrapper<'a, Counter<'b>>> for Stored<'a, 'b, 'c> {
            fn from(value: Wrapper<'a, Counter<'b>>) -> Self { Self::First(value) }
        }
        impl<'a, 'b, 'c> From<Wrapper<'a, ArrayCounter<'c>>> for Stored<'a, 'b, 'c> {
            fn from(value: Wrapper<'a, ArrayCounter<'c>>) -> Self { Self::Second(value) }
        }
        fn install<'a, F: Callback, S: From<Wrapper<'a, F>>>(slot: &mut S, action: &'a mut F) {
            *slot = Wrapper(action).into();
        }
        fn main() {
            let mut count = 1;
            let mut values = vec![10];
            let mut first = Counter(&mut count);
            let mut second = ArrayCounter(&mut values);
            {
                let mut holder = Stored::Initial;
                install(&mut holder, &mut first);
                assert_eq!(holder.call(), 2);
                install(&mut holder, &mut second);
                assert_eq!(holder.call(), 12);
                assert_eq!(holder.call(), 14);
            }
            assert_eq!(first.call(), 3);
            assert_eq!(second.call(), 16);
        }
    "#).unwrap();
    let output = support::run(support::rustc().arg("--edition=2024").arg(&source).arg("-o").arg(&executable), 60);
    support::success(&output);
    let output = support::run(&mut std::process::Command::new(&executable), 10);
    support::success(&output);
}
