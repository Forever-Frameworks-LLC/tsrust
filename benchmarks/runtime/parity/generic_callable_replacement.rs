// Independent standard Rust generics, explicit mutable environments and owned values.
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
fn length(s: &str) -> i64 {
    i64::try_from(s.len()).unwrap()
}

fn first<T>(_value: T, base: i64) -> i64 {
    base
}
fn second<T>(_value: T, base: i64) -> i64 {
    add(base, 7)
}
fn attempt(flag: bool, base: i64) -> i64 {
    fn good<T>(_value: T, base: i64) -> Result<i64, &'static str> {
        Ok(base)
    }
    fn failing<T>(_value: T, _base: i64) -> Result<i64, &'static str> {
        Err("selected failure")
    }
    let action: fn(i64, i64) -> Result<i64, &'static str> =
        if flag { failing::<i64> } else { good::<i64> };
    match action(base, base) {
        Ok(v) => v,
        Err("selected failure") => 11,
        Err(_) => 0,
    }
}
fn first_default<T>(_value: T, offset: Option<i64>) -> i64 {
    offset.unwrap_or(3)
}
fn second_default<T>(_value: T, offset: Option<i64>) -> i64 {
    offset.unwrap_or(7)
}
fn defaulted(flag: bool) -> i64 {
    let action: fn(i64, Option<i64>) -> i64 = if flag {
        second_default::<i64>
    } else {
        first_default::<i64>
    };
    action(1, None)
}
fn owned_head<T>(value: T, _tail: Vec<T>) -> T {
    value
}
fn owned_tail<T>(_value: T, mut tail: Vec<T>) -> T {
    tail.remove(0)
}
fn selected_text(flag: bool) -> std::string::String {
    let action: fn(std::string::String, Vec<std::string::String>) -> std::string::String =
        if flag { owned_tail } else { owned_head };
    action("head".to_owned(), vec!["next-value".to_owned()])
}
fn nullable_read<T>(_value: T) -> Option<std::string::String> {
    Some("read".to_owned())
}
fn nullable_take<T>(value: T) -> T {
    value
}
fn nullable_length(flag: bool, value: Option<std::string::String>) -> i64 {
    let action: fn(Option<std::string::String>) -> Option<std::string::String> =
        if flag { nullable_take } else { nullable_read };
    action(value).as_deref().map_or(0, length)
}
fn callback_first<T, F: Fn() -> Result<i64, &'static str>>(
    _value: T,
    callback: &F,
) -> Result<i64, &'static str> {
    callback()
}
fn callback_second<T, F: Fn() -> Result<i64, &'static str>>(
    _value: T,
    callback: &F,
) -> Result<i64, &'static str> {
    callback().map(|v| add(v, 7))
}
fn callback_result(flag: bool, fail: bool, base: i64) -> i64 {
    let callback = || {
        if fail {
            Err("callback failed")
        } else {
            Ok(base)
        }
    };
    let action: fn(i64, &_) -> Result<i64, &'static str> = if flag {
        callback_second
    } else {
        callback_first
    };
    match action(base, &callback) {
        Ok(v) => v,
        Err("callback failed") => 11,
        Err(_) => 0,
    }
}
fn captured_result(flag: bool, fail: bool, base: i64) -> i64 {
    let left = base;
    let right = add(base, 7);
    let captured_first = |_value: i64| Ok(left);
    let captured_second = |_value: i64| if fail { Err(right) } else { Ok(right) };
    let mut action: &dyn Fn(i64) -> Result<i64, i64> = &captured_first;
    let saved = action;
    action = &captured_first;
    if flag {
        action = &captured_second;
    }
    match saved(0).and_then(|a| action(0).map(|b| add(a, b))) {
        Ok(v) => v,
        Err(_) => 11,
    }
}
#[derive(Clone, Copy)]
enum StateAction {
    First,
    Second,
}
impl StateAction {
    fn call<T>(self, value: T, other: Option<T>) -> T {
        match self {
            Self::First => value,
            Self::Second => other.unwrap(),
        }
    }
}
fn state_factory(initial: StateAction) -> impl FnMut(bool) -> StateAction {
    let mut action = initial;
    move |restore| {
        std::mem::replace(
            &mut action,
            if restore {
                StateAction::First
            } else {
                StateAction::Second
            },
        )
    }
}
fn state_result(base: i64) -> Vec<i64> {
    let mut first = state_factory(StateAction::First);
    let mut second = state_factory(StateAction::Second);
    let original = first(false);
    let changed = first(true);
    let restored = first(false);
    let independent = second(true);
    vec![
        original.call(base, None),
        changed.call(base, Some(add(base, 7))),
        restored.call(base, None),
        independent.call(base, Some(add(base, 9))),
    ]
}
fn pool_result(base: i64) -> Vec<i64> {
    let mut instances = vec![state_factory(StateAction::First)];
    for _ in 0..2 {
        instances.push(state_factory(StateAction::First));
    }
    let _seed = instances.remove(0);
    let mut first = instances.remove(0);
    let mut second = instances.remove(0);
    let original = first(false);
    let independent = second(true);
    let changed = first(true);
    vec![
        original.call(base, None),
        independent.call(base, None),
        changed.call(base, Some(add(base, 9))),
    ]
}
fn nested_factory(initial: StateAction) -> impl FnMut(bool) -> StateAction {
    let mut action = state_factory(initial);
    move |restore| action(restore)
}
fn nested_pool_result(base: i64) -> Vec<i64> {
    let mut instances = vec![nested_factory(StateAction::First)];
    for _ in 0..2 {
        instances.push(nested_factory(StateAction::First));
    }
    let _seed = instances.remove(0);
    let mut first = instances.remove(0);
    let mut second = instances.remove(0);
    let original = first(false);
    let independent = second(true);
    let changed = first(true);
    vec![
        original.call(base, None),
        independent.call(base, None),
        changed.call(base, Some(add(base, 9))),
    ]
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let base = add(add(i % 5, value1 % 5), 1);
        let flag = add(i, value1) % 2 == 1;
        let mut action: fn(i64, i64) -> i64 = first::<i64>;
        let saved = action;
        action = first::<i64>;
        if flag {
            action = second::<i64>;
        }
        let original = saved(base, base);
        let selected = action(base, base);
        let completed = attempt(flag, base);
        let omitted = defaulted(flag);
        let text = selected_text(flag);
        let present = nullable_length(flag, Some("nullable".to_owned()));
        let absent = nullable_length(flag, None);
        let callback = callback_result(flag, i % 3 == 0, base);
        let captured = captured_result(flag, i % 3 == 0, base);
        let state = state_result(base);
        let pool = pool_result(base);
        let nested = nested_pool_result(base);
        output.push(NumericArray(vec![
            original,
            selected,
            completed,
            omitted,
            length(&text),
            present,
            absent,
            callback,
            captured,
            state[0],
            state[1],
            state[2],
            state[3],
            pool[0],
            pool[1],
            pool[2],
            nested[0],
            nested[1],
            nested[2],
        ]));
    }
    Array(output)
}
fn main() {}
