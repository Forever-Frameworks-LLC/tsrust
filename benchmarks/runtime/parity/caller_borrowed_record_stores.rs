// Independent Rust caller-owned callback stores. Cells model the observed one-element
// mutable sources; lifetime-bound closure borrows never retain owners through RC storage.
use std::cell::Cell;
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
fn bump(value: &Cell<i64>, amount: i64) -> i64 {
    let next = add(value.get(), amount);
    value.set(next);
    next
}
#[derive(Debug)]
enum Failure {
    Code(i64),
}
struct Holder<'a> {
    run: Box<dyn Fn() -> Result<i64, Failure> + 'a>,
}
impl Holder<'_> {
    fn call(&self) -> i64 {
        (self.run)().unwrap()
    }
}
struct Container<'a> {
    handler: Holder<'a>,
}
struct OnceHolder<'a> {
    run: Box<dyn FnOnce() -> i64 + 'a>,
}
struct Nullable<'a> {
    run: Option<Box<dyn Fn() -> i64 + 'a>>,
}
impl Nullable<'_> {
    fn empty() -> Self {
        Self { run: None }
    }
    fn call(&self) -> i64 {
        self.run.as_ref().unwrap()()
    }
    fn clear(&mut self) {
        self.run = None;
    }
}
struct OwnedNullable {
    run: Option<Box<dyn FnMut() -> i64>>,
}
fn install_array<'a>(holder: &mut Holder<'a>, source: &'a Cell<i64>) {
    holder.run = Box::new(move || Ok(source.get()));
}
fn replace_both<'a>(holder: &mut Holder<'a>, left: &'a Cell<i64>, right: &'a Cell<i64>) {
    install_array(holder, left);
    install_array(holder, right);
}
fn install_nested<'a>(container: &mut Container<'a>, left: &'a Cell<i64>, right: &'a Cell<i64>) {
    container.handler.run = Box::new(move || Ok(add(left.get(), right.get())));
}
fn install_callback<'a, F: Fn() -> Result<i64, Failure> + 'a>(
    holder: &mut Holder<'a>,
    action: &'a F,
) {
    holder.run = Box::new(move || action());
}
fn install_pair<'a, L: Fn() -> Result<i64, Failure> + 'a, R: Fn() -> Result<i64, Failure> + 'a>(
    holder: &mut Holder<'a>,
    left: &'a L,
    right: &'a R,
) {
    holder.run = Box::new(move || Ok(add(left()?, right()?)));
}
fn install_mutable<'a>(holder: &mut Holder<'a>, source: &'a Cell<i64>) {
    holder.run = Box::new(move || Ok(bump(source, 1)));
}
fn install_direct<'a, F: Fn() -> Result<i64, Failure> + 'a>(
    holder: &mut Holder<'a>,
    action: &'a F,
) {
    install_callback(holder, action);
}
fn install_once<'a, F: FnOnce() -> i64 + 'a>(holder: &mut OnceHolder<'a>, action: F) {
    holder.run = Box::new(action);
}
fn install_and_fail<'a, F: Fn() -> Result<i64, Failure> + 'a>(
    holder: &mut Holder<'a>,
    action: &'a F,
) -> Result<(), &'static str> {
    install_direct(holder, action);
    Err("installed")
}
fn fail_before_install<'a, F: Fn() -> Result<i64, Failure> + 'a>(
    holder: &mut Holder<'a>,
    action: &'a F,
    fail: bool,
) -> Result<(), &'static str> {
    if fail {
        return Err("before");
    }
    install_direct(holder, action);
    Ok(())
}
fn install_nullable<'a, F: Fn() -> i64 + 'a>(holder: &mut Nullable<'a>, action: &'a F) {
    holder.run = Some(Box::new(move || action()));
}
fn relay_clear_nullable(holder: &mut Nullable<'_>, early: bool) {
    if early {
        holder.clear();
        return;
    }
    holder.clear();
}
fn counter(start: i64) -> impl Fn() -> i64 {
    let value = Cell::new(start);
    move || bump(&value, 1)
}
fn install_nullable_data<'a>(holder: &mut Nullable<'a>, source: &'a Cell<i64>, calls: &Cell<i64>) {
    bump(calls, 1);
    holder.run = Some(Box::new(move || bump(source, 1)));
}
fn install_with_effects<'a, F: Fn() -> i64 + 'a>(
    holder: &mut Nullable<'a>,
    action: &'a F,
    calls: &Cell<i64>,
) {
    bump(calls, 1);
    install_nullable(holder, action);
}
fn choose_callback_branch(plus: bool, calls: &Cell<i64>) -> bool {
    bump(calls, 10);
    plus
}
fn install_conditional<'a>(
    holder: &mut Nullable<'a>,
    source: &'a Cell<i64>,
    plus: bool,
    calls: &Cell<i64>,
) {
    bump(calls, 1);
    let selected: Box<dyn Fn() -> i64 + 'a> = if choose_callback_branch(plus, calls) {
        bump(calls, 100);
        Box::new(move || add(source.get(), 1))
    } else {
        bump(calls, 200);
        Box::new(move || add(source.get(), 2))
    };
    bump(calls, selected());
    holder.run = Some(selected);
}
fn install_captured_source<'a>(holder: &mut Nullable<'a>, source: &'a Cell<i64>) {
    holder.run = Some(Box::new(move || source.get()));
}
fn install_scoped_mutation<'a>(holder: &mut Nullable<'a>, source: &'a Cell<i64>) {
    holder.run = Some(Box::new(move || bump(source, 1)));
}
fn install_owned_mutation(holder: &mut OwnedNullable, mut source: Vec<i64>) {
    holder.run = Some(Box::new(move || {
        source[0] = add(source[0], 1);
        source[0]
    }));
}
fn install_through_finally<'a>(
    holder: &mut Nullable<'a>,
    first: &'a Cell<i64>,
    last: &'a Cell<i64>,
) {
    install_captured_source(holder, first);
    install_captured_source(holder, last);
}
fn install_through_catch<'a>(
    holder: &mut Nullable<'a>,
    first: &'a Cell<i64>,
    last: &'a Cell<i64>,
    fail: bool,
) {
    let completion = if fail {
        Err("region")
    } else {
        install_captured_source(holder, first);
        Ok(())
    };
    if completion.is_err() {
        install_captured_source(holder, last);
    }
}
fn with_installer<'a, F: FnOnce(&mut Nullable<'a>)>(installer: F, target: &mut Nullable<'a>) {
    installer(target)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let base = Cell::new(value);
        let other = Cell::new(add(value, 2));
        let (first, replaced, nested, callback, callback_replaced, paired, pair_replaced) = {
            let mut holder = Holder {
                run: Box::new(|| Ok(0)),
            };
            install_array(&mut holder, &base);
            let first = holder.call();
            replace_both(&mut holder, &base, &other);
            let replaced = holder.call();
            let mut container = Container {
                handler: Holder {
                    run: Box::new(|| Ok(0)),
                },
            };
            install_nested(&mut container, &base, &other);
            let nested = container.handler.call();
            let action = || Ok(base.get());
            let constant = || Ok(9);
            let mut callback_holder = Holder {
                run: Box::new(|| Ok(0)),
            };
            install_callback(&mut callback_holder, &action);
            let callback = callback_holder.call();
            install_direct(&mut callback_holder, &constant);
            let read_callback = || callback_holder.call();
            let callback_replaced = read_callback();
            let mut pair_holder = Holder {
                run: Box::new(|| Ok(0)),
            };
            install_pair(&mut pair_holder, &action, &constant);
            let paired = pair_holder.call();
            install_pair(&mut pair_holder, &constant, &action);
            let pair_replaced = pair_holder.call();
            (
                first,
                replaced,
                nested,
                callback,
                callback_replaced,
                paired,
                pair_replaced,
            )
        };
        bump(&base, 1);
        let (changed_first, changed_second) = {
            let mut holder = Holder {
                run: Box::new(|| Ok(0)),
            };
            install_mutable(&mut holder, &base);
            (holder.call(), holder.call())
        };
        let (callback_first, callback_second, mutable_replaced, callback_after) = {
            let action = || Ok(bump(&base, 1));
            let second_action = || Ok(bump(&other, 10));
            let (first, second, replaced) = {
                let mut holder = Holder {
                    run: Box::new(|| Ok(0)),
                };
                install_callback(&mut holder, &action);
                let first = holder.call();
                let second = holder.call();
                install_callback(&mut holder, &second_action);
                let invoke = || holder.call();
                (first, second, invoke())
            };
            (first, second, replaced, action().unwrap())
        };
        let consumed = {
            let owned = vec![value];
            let once = move || {
                let moved = owned;
                moved[0]
            };
            let mut holder = OnceHolder {
                run: Box::new(|| 0),
            };
            install_once(&mut holder, once);
            (holder.run)()
        };
        let after_failure = {
            let action = || Ok(base.get());
            let mut holder = Holder {
                run: Box::new(|| Ok(add(value, 9))),
            };
            let caught = install_and_fail(&mut holder, &action).err() == Some("installed");
            add(holder.call(), if caught { 0 } else { 100 })
        };
        let before_failure = {
            let action = || Ok(base.get());
            let mut holder = Holder {
                run: Box::new(|| Ok(add(value, 9))),
            };
            let caught = fail_before_install(&mut holder, &action, true).err() == Some("before");
            add(holder.call(), if caught { 0 } else { 100 })
        };
        let callback_failure = {
            let payload = vec![add(value, 17)];
            let action = || Err(Failure::Code(payload[0]));
            let mut holder = Holder {
                run: Box::new(|| Ok(0)),
            };
            install_direct(&mut holder, &action);
            let guarded = || match (holder.run)() {
                Ok(value) => value,
                Err(Failure::Code(error)) => error,
            };
            guarded()
        };
        let (nullable_before, nullable_after, nullable_empty) = {
            let data = Cell::new(value);
            let action = || bump(&data, 1);
            let mut holder = Nullable::empty();
            install_nullable(&mut holder, &action);
            let before = holder.call();
            relay_clear_nullable(&mut holder, i % 2 == 0);
            let after = action();
            (before, after, i64::from(holder.run.is_none()))
        };
        let (nullable_released, nullable_replacement, nullable_replacement_after) = {
            let original = Cell::new(value);
            let replacement = Cell::new(add(value, 4));
            let original_action = || bump(&original, 1);
            let replacement_action = || bump(&replacement, 3);
            let mut holder = Nullable::empty();
            install_nullable(&mut holder, &original_action);
            install_nullable(&mut holder, &replacement_action);
            let released = original_action();
            let selected = holder.call();
            holder.clear();
            let after = add(
                replacement_action(),
                if holder.run.is_none() { 0 } else { 100 },
            );
            (released, selected, after)
        };
        let (factory_before, factory_released, factory_replacement, factory_after, factory_effects) = {
            let original = counter(value);
            let replacement = counter(add(value, 4));
            let effects = Cell::new(0);
            let mut holder = Nullable::empty();
            install_with_effects(&mut holder, &original, &effects);
            let before = holder.call();
            install_with_effects(&mut holder, &replacement, &effects);
            let effect_count = effects.get();
            let released = original();
            let selected = holder.call();
            holder.clear();
            let after = add(replacement(), if holder.run.is_none() { 0 } else { 100 });
            (before, released, selected, after, effect_count)
        };
        let (data_released, data_replacement, data_after, data_effects) = {
            let original = Cell::new(value);
            let replacement = Cell::new(add(value, 4));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            install_nullable_data(&mut holder, &original, &calls);
            holder.call();
            install_nullable_data(&mut holder, &replacement, &calls);
            let effects = calls.get();
            let released = bump(&original, 1);
            let selected = holder.call();
            holder.clear();
            let after = add(
                bump(&replacement, 1),
                if holder.run.is_none() { 0 } else { 100 },
            );
            (released, selected, after, effects)
        };
        let (conditional_first, conditional_released, conditional_replacement, conditional_effects) = {
            let original = Cell::new(value);
            let replacement = Cell::new(add(value, 4));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            install_conditional(&mut holder, &original, i % 2 == 0, &calls);
            let first = holder.call();
            install_conditional(&mut holder, &replacement, i % 2 != 0, &calls);
            let released = bump(&original, 1);
            let selected = holder.call();
            (first, released, selected, calls.get())
        };
        let (forwarded_first, forwarded_released, forwarded_replacement, forwarded_effects) = {
            let original = Cell::new(value);
            let replacement = Cell::new(add(value, 4));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            let perform = install_conditional;
            perform(&mut holder, &original, true, &calls);
            let first = holder.call();
            perform(&mut holder, &replacement, false, &calls);
            let released = bump(&original, 1);
            let selected = holder.call();
            (first, released, selected, calls.get())
        };
        let (closure_first, closure_released, closure_replacement, closure_effects) = {
            let original = Cell::new(add(value, 2));
            let replacement = Cell::new(add(value, 6));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            with_installer(
                |target| install_conditional(target, &original, true, &calls),
                &mut holder,
            );
            let first = holder.call();
            with_installer(
                |target| install_conditional(target, &replacement, false, &calls),
                &mut holder,
            );
            let released = bump(&original, 1);
            let selected = holder.call();
            (first, released, selected, calls.get())
        };
        let (captured_first, captured_released, captured_replacement, captured_effects) = {
            let original = Cell::new(add(value, 3));
            let replacement = Cell::new(add(value, 7));
            let calls = Cell::new(0);
            let offset = 2;
            let mut holder = Nullable::empty();
            with_installer(
                |target| {
                    install_conditional(target, &original, true, &calls);
                    bump(&calls, offset);
                },
                &mut holder,
            );
            let first = holder.call();
            with_installer(
                |target| {
                    install_conditional(target, &replacement, true, &calls);
                    bump(&calls, offset);
                },
                &mut holder,
            );
            let released = bump(&original, 1);
            let selected = holder.call();
            (first, released, selected, calls.get())
        };
        let (retained_capture, cleared_capture) = {
            let captured = Cell::new(add(value, 8));
            let mut holder = Nullable::empty();
            with_installer(
                |target| install_captured_source(target, &captured),
                &mut holder,
            );
            let retained = holder.call();
            holder.clear();
            (retained, bump(&captured, 1))
        };
        let (independent_first, independent_released, independent_replacement) = {
            let original = Cell::new(add(value, 9));
            let replacement = Cell::new(add(value, 13));
            let mut holder = Nullable::empty();
            with_installer(
                |target| install_captured_source(target, &original),
                &mut holder,
            );
            let first = holder.call();
            with_installer(
                |target| install_captured_source(target, &replacement),
                &mut holder,
            );
            let released = bump(&original, 1);
            (first, released, holder.call())
        };
        let inspect_local_owners = || {
            let original = Cell::new(add(value, 14));
            let replacement = Cell::new(add(value, 18));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            with_installer(
                |target| install_conditional(target, &original, true, &calls),
                &mut holder,
            );
            let first = holder.call();
            with_installer(
                |target| install_conditional(target, &replacement, true, &calls),
                &mut holder,
            );
            let released = bump(&original, 1);
            vec![first, released, holder.call(), calls.get()]
        };
        let local_owner_values = inspect_local_owners();
        let (scoped_first, scoped_second, scoped_released, scoped_effects) = {
            let source = Cell::new(add(value, 20));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            with_installer(
                |target| {
                    bump(&calls, 1);
                    install_scoped_mutation(target, &source);
                },
                &mut holder,
            );
            let first = holder.call();
            holder.clear();
            with_installer(
                |target| {
                    bump(&calls, 1);
                    install_scoped_mutation(target, &source);
                },
                &mut holder,
            );
            let second = holder.call();
            holder.clear();
            (first, second, bump(&source, 1), calls.get())
        };
        let (owned_first, owned_second) = {
            let source = vec![add(value, 30)];
            let mut holder = OwnedNullable { run: None };
            let transfer_once =
                move |target: &mut OwnedNullable| install_owned_mutation(target, source);
            transfer_once(&mut holder);
            let first = holder.run.as_mut().unwrap()();
            let second = holder.run.as_mut().unwrap()();
            holder.run = None;
            (first, second)
        };
        let (nested_first, nested_released, nested_replacement, nested_effects) = {
            let original = Cell::new(add(value, 40));
            let replacement = Cell::new(add(value, 45));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            with_installer(
                |target| {
                    with_installer(
                        |next| {
                            with_installer(
                                |destination| {
                                    bump(&calls, 1);
                                    install_captured_source(destination, &original);
                                },
                                next,
                            )
                        },
                        target,
                    )
                },
                &mut holder,
            );
            let first = holder.call();
            with_installer(
                |target| {
                    with_installer(
                        |next| {
                            with_installer(
                                |destination| {
                                    bump(&calls, 1);
                                    install_captured_source(destination, &replacement);
                                },
                                next,
                            )
                        },
                        target,
                    )
                },
                &mut holder,
            );
            let released = bump(&original, 1);
            (first, released, holder.call(), calls.get())
        };
        let (held_first, held_released, held_replacement, held_effects) = {
            let original = Cell::new(add(value, 50));
            let replacement = Cell::new(add(value, 60));
            let calls = Cell::new(0);
            let mut holder = Nullable::empty();
            with_installer(
                |target| {
                    with_installer(
                        |destination| {
                            bump(&calls, 1);
                            install_captured_source(destination, &original);
                        },
                        target,
                    )
                },
                &mut holder,
            );
            let first = holder.call();
            with_installer(
                |target| {
                    with_installer(
                        |destination| {
                            bump(&calls, 1);
                            install_captured_source(destination, &replacement);
                        },
                        target,
                    )
                },
                &mut holder,
            );
            let released = bump(&original, 1);
            (first, released, holder.call(), calls.get())
        };
        let (region_first, region_second) = {
            let original = Cell::new(add(value, 70));
            let replacement = Cell::new(add(value, 80));
            let mut holder = Nullable::empty();
            install_through_finally(&mut holder, &original, &replacement);
            let first = holder.call();
            install_through_catch(&mut holder, &original, &replacement, value % 2 == 0);
            (first, holder.call())
        };
        output.push(NumericArray(vec![
            first,
            replaced,
            nested,
            callback,
            paired,
            changed_first,
            changed_second,
            callback_first,
            callback_second,
            callback_after,
            base.get(),
            callback_replaced,
            pair_replaced,
            mutable_replaced,
            consumed,
            after_failure,
            before_failure,
            callback_failure,
            nullable_before,
            nullable_after,
            nullable_empty,
            nullable_released,
            nullable_replacement,
            nullable_replacement_after,
            factory_before,
            factory_released,
            factory_replacement,
            factory_after,
            data_released,
            data_replacement,
            data_after,
            factory_effects,
            data_effects,
            conditional_first,
            conditional_released,
            conditional_replacement,
            conditional_effects,
            forwarded_first,
            forwarded_released,
            forwarded_replacement,
            forwarded_effects,
            closure_first,
            closure_released,
            closure_replacement,
            closure_effects,
            captured_first,
            captured_released,
            captured_replacement,
            captured_effects,
            retained_capture,
            cleared_capture,
            independent_first,
            independent_released,
            independent_replacement,
            local_owner_values[0],
            local_owner_values[1],
            local_owner_values[2],
            local_owner_values[3],
            scoped_first,
            scoped_second,
            scoped_released,
            scoped_effects,
            owned_first,
            owned_second,
            nested_first,
            nested_released,
            nested_replacement,
            nested_effects,
            held_first,
            held_released,
            held_replacement,
            held_effects,
            region_first,
            region_second,
        ]));
    }
    Array(output)
}
fn main() {}
