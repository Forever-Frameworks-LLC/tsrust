// Independent standard Rust scoped record/map composition. Borrowed cells model
// the fixture's one-element mutable sources; callbacks never use RC backing storage.
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
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn sum(values: impl IntoIterator<Item = i64>) -> i64 {
    values.into_iter().fold(0, add)
}
fn bump(value: &Cell<i64>, amount: i64) -> i64 {
    let next = add(value.get(), amount);
    value.set(next);
    next
}
struct Holder<'a> {
    run: Box<dyn Fn() -> i64 + 'a>,
}
impl Holder<'_> {
    fn call(&self) -> i64 {
        (self.run)()
    }
}
// ponytail: insertion-order Vec scans fit this <=3-entry corpus; index if it grows.
type Map<'a> = Vec<(&'static str, Holder<'a>)>;
fn put<'a, F: Fn() -> i64 + 'a>(map: &mut Map<'a>, key: &'static str, run: F) {
    let holder = Holder { run: Box::new(run) };
    if let Some((_, old)) = map.iter_mut().find(|(k, _)| *k == key) {
        *old = holder;
    } else {
        map.push((key, holder));
    }
}
fn install<'a>(map: &mut Map<'a>, key: &'static str, source: &'a Cell<i64>) {
    put(map, key, move || source.get());
}
fn get<'a, 'b>(map: &'b Map<'a>, key: &str) -> Option<&'b Holder<'a>> {
    map.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, holder)| holder)
}
fn call(map: &Map<'_>, key: &str) -> i64 {
    get(map, key).map_or(0, Holder::call)
}
fn delete(map: &mut Map<'_>, key: &str) -> bool {
    if let Some(at) = map.iter().position(|(k, _)| *k == key) {
        map.remove(at);
        true
    } else {
        false
    }
}
fn size(map: &Map<'_>) -> i64 {
    i64::try_from(map.len()).unwrap()
}
fn cleanup(map: &mut Map<'_>, trace: &mut [i64], at: usize) {
    trace[at] = add(trace[at], size(map));
    map.clear();
    trace[at + 1] = add(trace[at + 1], size(map));
}
struct Store<'a> {
    handlers: Map<'a>,
}
struct Refill<'a> {
    marker: i64,
    handlers: Map<'a>,
}
fn prefix_result(fail: bool) -> Result<i64, i64> {
    if fail { Err(5) } else { Ok(1) }
}
fn no_catch_renewal(fail: bool, value: i64, trace: &mut [i64]) -> Result<i64, i64> {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let pending: Result<i64, i64> = (|| {
        prefix_result(fail)?;
        map.clear();
        install(&mut map, "next", &next);
        Ok(size(&map))
    })();
    trace[0] = size(&map);
    map.clear();
    trace[1] = size(&map);
    let retained = pending?;
    old.set(add(value, 70));
    next.set(add(value, 71));
    Ok(sum([retained, old.get(), next.get()]))
}
fn no_catch_return(early: bool, value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "old", &old);
    prefix_result(false).unwrap();
    let pending = if early {
        Some(add(value, 80))
    } else {
        map.clear();
        install(&mut map, "next", &next);
        None
    };
    trace[0] = size(&map);
    map.clear();
    trace[1] = size(&map);
    if let Some(value) = pending {
        return value;
    }
    old.set(add(value, 82));
    next.set(add(value, 83));
    add(old.get(), next.get())
}
fn no_catch_loop(value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut turn = 0;
    let mut observed = 0;
    while turn < 3 {
        let mut map = Map::new();
        install(&mut map, "old", &old);
        turn = add(turn, 1);
        prefix_result(false).unwrap();
        map.clear();
        install(&mut map, "next", &next);
        cleanup(&mut map, trace, 0);
        if turn == 1 {
            continue;
        }
        if turn == 3 {
            break;
        }
        old.set(add(value, 90));
        next.set(add(value, 91));
        observed = add(old.get(), next.get());
    }
    add(observed, turn)
}
fn no_catch_outer_loop(value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut turn = 0;
    let mut observed = 0;
    let mut map = Map::new();
    while turn < 3 {
        install(&mut map, "old", &old);
        turn = add(turn, 1);
        prefix_result(false).unwrap();
        map.clear();
        install(&mut map, "next", &next);
        cleanup(&mut map, trace, 0);
        if turn == 1 {
            continue;
        }
        if turn == 3 {
            break;
        }
        old.set(add(value, 90));
        next.set(add(value, 91));
        observed = add(old.get(), next.get());
    }
    sum([observed, turn, size(&map)])
}
fn nested_map_initializer(value: i64) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut record = Refill {
        marker: add(value, 100),
        handlers: Map::new(),
    };
    install(&mut record.handlers, "old", &old);
    install(&mut record.handlers, "next", &next);
    let observed = add(size(&record.handlers), record.marker);
    record.handlers.clear();
    old.set(add(value, 101));
    next.set(add(value, 102));
    sum([observed, old.get(), next.get(), size(&record.handlers)])
}
fn record_map_loop(value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut turn = 0;
    let mut observed = 0;
    let mut record = Refill {
        marker: add(value, 110),
        handlers: Map::new(),
    };
    while turn < 3 {
        install(&mut record.handlers, "old", &old);
        turn = add(turn, 1);
        prefix_result(false).unwrap();
        record.handlers.clear();
        install(&mut record.handlers, "next", &next);
        cleanup(&mut record.handlers, trace, 0);
        if turn == 1 {
            continue;
        }
        if turn == 3 {
            break;
        }
        record.marker = add(record.marker, 1);
        old.set(add(value, 111));
        next.set(add(value, 112));
        observed = sum([old.get(), next.get(), record.marker]);
    }
    let field_order = ["marker", "handlers"];
    let order = if field_order[0] == "marker" { 4 } else { 0 };
    sum([observed, turn, size(&record.handlers), order])
}
fn nested_caught_map_cleanup(fail: bool, value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let observed = match prefix_result(fail) {
        Ok(_) => {
            map.clear();
            install(&mut map, "next", &next);
            size(&map)
        }
        Err(_) => {
            map.clear();
            install(&mut map, "next", &next);
            cleanup(&mut map, trace, 0);
            7
        }
    };
    cleanup(&mut map, trace, 2);
    old.set(add(value, 120));
    next.set(add(value, 121));
    sum([observed, old.get(), next.get()])
}
fn nested_map_cleanup(fail: bool, value: i64, trace: &mut [i64]) -> Result<i64, i64> {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let pending: Result<i64, i64> = (|| {
        prefix_result(fail)?;
        map.clear();
        install(&mut map, "next", &next);
        Ok(size(&map))
    })();
    cleanup(&mut map, trace, 0);
    cleanup(&mut map, trace, 2);
    let observed = pending?;
    old.set(add(value, 120));
    next.set(add(value, 121));
    Ok(sum([observed, old.get(), next.get()]))
}
fn nested_abrupt_cleanup(fail: bool, value: i64, trace: &mut [i64]) -> Result<i64, i64> {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let mut pending = match prefix_result(fail) {
        Ok(_) => {
            map.clear();
            install(&mut map, "next", &next);
            Ok(size(&map))
        }
        Err(_) => {
            map.clear();
            install(&mut map, "next", &next);
            Err(9)
        }
    };
    cleanup(&mut map, trace, 0);
    if fail {
        pending = Err(11);
    }
    cleanup(&mut map, trace, 2);
    let observed = pending?;
    old.set(add(value, 120));
    next.set(add(value, 121));
    Ok(sum([observed, old.get(), next.get()]))
}
fn nested_returned_cleanup(fail: bool, value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let pending = match prefix_result(fail) {
        Ok(_) => {
            map.clear();
            install(&mut map, "next", &next);
            add(old.get(), value)
        }
        Err(_) => {
            map.clear();
            install(&mut map, "next", &next);
            add(next.get(), value)
        }
    };
    cleanup(&mut map, trace, 0);
    map.clear();
    old.set(0);
    next.set(0);
    trace[2] = add(trace[2], size(&map));
    pending
}
fn caught_map_loop(value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut turn = 0;
    let mut observed = 0;
    let mut map = Map::new();
    while turn < 3 {
        install(&mut map, "old", &old);
        turn = add(turn, 1);
        let failed = prefix_result(turn != 2).is_err();
        map.clear();
        install(&mut map, "next", &next);
        cleanup(&mut map, trace, 0);
        old.set(add(value, 140));
        next.set(add(value, 141));
        if failed {
            if turn == 1 {
                continue;
            }
            break;
        }
        observed = add(old.get(), next.get());
    }
    old.set(add(value, 142));
    next.set(add(value, 143));
    sum([observed, turn, size(&map), old.get(), next.get()])
}
fn paired_map_loop(value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut turn = 0;
    let spare: Vec<(&str, i64)> = Vec::new();
    let mut left = Map::new();
    let mut right = Map::new();
    while turn < 3 {
        install(&mut left, "old", &old);
        install(&mut right, "next", &next);
        turn = add(turn, 1);
        let failed = prefix_result(turn != 2).is_err();
        trace[0] = sum([trace[0], size(&left), size(&right)]);
        left.clear();
        right.clear();
        old.set(add(value, 150));
        next.set(add(value, 151));
        trace[1] = sum([trace[1], size(&left), size(&right)]);
        if failed {
            if turn == 1 {
                continue;
            }
            break;
        }
    }
    trace[1] = add(trace[1], i64::try_from(spare.len()).unwrap());
    sum([old.get(), next.get(), turn, size(&left), size(&right)])
}
fn nested_paired_map_loop(value: i64, trace: &mut [i64]) -> i64 {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut turn = 0;
    let mut left = Map::new();
    let mut right = Map::new();
    while turn < 3 {
        install(&mut left, "old", &old);
        install(&mut right, "next", &next);
        turn = add(turn, 1);
        let failed = prefix_result(turn != 2).is_err();
        trace[1] = add(trace[1], turn);
        trace[0] = sum([trace[0], size(&left), size(&right)]);
        left.clear();
        right.clear();
        old.set(add(value, 150));
        next.set(add(value, 151));
        trace[1] = sum([trace[1], size(&left), size(&right)]);
        if failed {
            if turn == 1 {
                continue;
            }
            break;
        }
    }
    sum([old.get(), next.get(), turn, size(&left), size(&right)])
}
fn empty_scoped_map() -> i64 {
    let mut unfilled = Map::new();
    let mut observed = size(&unfilled);
    if get(&unfilled, "none").is_some() {
        observed = add(observed, 100);
    }
    if delete(&mut unfilled, "none") {
        observed = add(observed, 1000);
    }
    let absent = get(&unfilled, "none");
    observed = add(observed, i64::from(absent.is_none()));
    if let Some(holder) = absent {
        observed = add(observed, holder.call());
    }
    for (_, entry) in &unfilled {
        observed = add(observed, entry.call());
    }
    let mut moved = unfilled;
    observed = add(observed, size(&moved));
    moved.clear();
    observed
}
fn counted_map_loop() -> i64 {
    let values = Cell::new(3);
    let mut turn = 0;
    let mut handlers = Map::new();
    for _step in 0..3 {
        install(&mut handlers, "value", &values);
        let failed = prefix_result(turn != 1).is_err();
        let prior = turn;
        handlers.clear();
        bump(&values, 2);
        turn = add(turn, 1);
        if failed {
            if prior == 0 {
                continue;
            }
            break;
        }
    }
    sum([turn, size(&handlers), values.get()])
}
fn post_map_loop() -> i64 {
    let values = Cell::new(3);
    let mut turn = 0;
    let mut handlers = Map::new();
    loop {
        install(&mut handlers, "value", &values);
        let failed = prefix_result(turn != 1).is_err();
        let prior = turn;
        handlers.clear();
        bump(&values, 2);
        turn = add(turn, 1);
        if failed && prior != 0 {
            break;
        }
        if turn >= 3 {
            break;
        }
    }
    sum([turn, size(&handlers), values.get()])
}
fn for_of_map_loop() -> i64 {
    let values = Cell::new(3);
    let steps = [0, 1, 2];
    let mut handlers = Map::new();
    for step in steps {
        install(&mut handlers, "value", &values);
        let failed = prefix_result(step != 1).is_err();
        handlers.clear();
        bump(&values, 2);
        if failed {
            if step == 0 {
                continue;
            }
            break;
        }
    }
    add(size(&handlers), values.get())
}
fn known_entry_release(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut observed = i64::from(delete(&mut map, "first"));
    first.set(add(value, 2));
    observed = add(observed, call(&map, "second"));
    observed = sum([observed, first.get(), size(&map)]);
    map.clear();
    second.set(add(value, 3));
    add(observed, second.get())
}
fn observe(map: &Map<'_>) -> i64 {
    map.iter()
        .fold(0, |total, (_, holder)| add(total, holder.call()))
}
struct ReleaseInner<'a> {
    handlers: Map<'a>,
    label: &'static str,
}
struct ReleaseOuter<'a> {
    inner: ReleaseInner<'a>,
    marker: i64,
}
fn moved_field_entry_release(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let original = ReleaseOuter {
        inner: ReleaseInner {
            handlers: Map::new(),
            label: "kept",
        },
        marker: 23,
    };
    let mut moved = original;
    install(&mut moved.inner.handlers, "first", &first);
    install(&mut moved.inner.handlers, "second", &second);
    let mut observed = i64::from(delete(&mut moved.inner.handlers, "first"));
    first.set(add(value, 2));
    install(&mut moved.inner.handlers, "next", &first);
    observed = add(observed, observe(&moved.inner.handlers));
    observed = sum([
        observed,
        i64::try_from(moved.inner.label.len()).unwrap(),
        moved.marker,
    ]);
    moved.inner.handlers.clear();
    first.set(add(value, 4));
    second.set(add(value, 5));
    sum([observed, first.get(), second.get()])
}
fn replacement_entry_outcome(value: i64, fail: bool) -> i64 {
    let first = Cell::new(value);
    let next = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    let mut observed = match prefix_result(fail) {
        Ok(_) => {
            install(&mut map, "first", &next);
            0
        }
        Err(_) => 5,
    };
    observed = add(observed, call(&map, "first"));
    map.clear();
    first.set(add(value, 4));
    next.set(add(value, 5));
    sum([observed, first.get(), next.get()])
}
fn branch_entry_release(value: i64, branch: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut observed = if branch {
        delete(&mut map, "first");
        1
    } else {
        delete(&mut map, "first");
        2
    };
    first.set(add(value, 2));
    observed = add(observed, observe(&map));
    observed = sum([observed, first.get(), size(&map)]);
    map.clear();
    second.set(add(value, 3));
    add(observed, second.get())
}
fn mixed_branch_entry_release(value: i64, branch: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    if branch {
        map.clear();
    } else {
        delete(&mut map, "first");
    }
    first.set(add(value, 2));
    let observed = sum([size(&map), first.get(), observe(&map)]);
    map.clear();
    second.set(add(value, 3));
    add(observed, second.get())
}
fn paired_branch_entry_release(value: i64, branch: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let mut left = Map::new();
    let mut right = Map::new();
    install(&mut left, "first", &first);
    install(&mut left, "second", &second);
    install(&mut right, "first", &first);
    install(&mut right, "third", &third);
    if branch {
        delete(&mut left, "first");
        delete(&mut right, "first");
    } else {
        delete(&mut right, "first");
        delete(&mut left, "first");
    }
    first.set(add(value, 3));
    let observed = sum([
        first.get(),
        size(&left),
        size(&right),
        observe(&left),
        observe(&right),
    ]);
    left.clear();
    right.clear();
    second.set(add(value, 4));
    third.set(add(value, 5));
    sum([observed, second.get(), third.get()])
}
fn post_entry_loop_release(value: i64, limit: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut turn = 0;
    loop {
        delete(&mut map, "first");
        turn = add(turn, 1);
        if turn >= limit {
            break;
        }
    }
    first.set(add(value, 2));
    let observed = sum([first.get(), turn, size(&map), observe(&map)]);
    map.clear();
    second.set(add(value, 3));
    add(observed, second.get())
}
fn counted_entry_loop_release(value: i64, limit: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    let mut turns = 0;
    for _step in 0..limit {
        install(&mut map, "first", &first);
        install(&mut map, "second", &second);
        delete(&mut map, "first");
        turns = add(turns, 1);
    }
    first.set(add(value, 2));
    let observed = sum([first.get(), turns, size(&map), observe(&map)]);
    map.clear();
    second.set(add(value, 3));
    add(observed, second.get())
}
fn for_of_entry_loop_release(value: i64, limit: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    let steps: Vec<_> = (0..limit).collect();
    let mut turns = 0;
    for _step in steps {
        install(&mut map, "first", &first);
        install(&mut map, "second", &second);
        delete(&mut map, "first");
        turns = add(turns, 1);
    }
    first.set(add(value, 2));
    let observed = sum([first.get(), turns, size(&map), observe(&map)]);
    map.clear();
    second.set(add(value, 3));
    add(observed, second.get())
}
fn loop_entry_refill(value: i64, limit: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let fourth = Cell::new(add(value, 4));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    first.set(add(value, 3));
    let mut turns = 0;
    for _step in 0..limit {
        install(&mut map, "third", &third);
        install(&mut map, "fourth", &fourth);
        turns = add(turns, 1);
    }
    let observed = sum([first.get(), turns, size(&map), observe(&map)]);
    map.clear();
    third.set(add(value, 5));
    fourth.set(add(value, 6));
    sum([observed, third.get(), fourth.get()])
}
fn inner_branch_entry_release(value: i64, branch: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut observed = 0;
    if branch {
        delete(&mut map, "first");
        first.set(add(value, 2));
        observed = add(observed, first.get());
    }
    observed = sum([observed, observe(&map), size(&map)]);
    map.clear();
    first.set(add(value, 3));
    second.set(add(value, 4));
    sum([observed, first.get(), second.get()])
}
fn branch_entry_refill(value: i64, branch: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let fourth = Cell::new(add(value, 3));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    first.set(add(value, 4));
    if branch {
        install(&mut map, "third", &third);
        install(&mut map, "fourth", &fourth);
    }
    let observed = sum([first.get(), size(&map), observe(&map)]);
    map.clear();
    third.set(add(value, 5));
    fourth.set(add(value, 6));
    sum([observed, third.get(), fourth.get()])
}
fn iteration_entry_mutation(value: i64, limit: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut turns = 0;
    for _step in 0..limit {
        install(&mut map, "first", &first);
        delete(&mut map, "first");
        bump(&first, 1);
        turns = add(turns, 1);
    }
    let observed = sum([first.get(), turns, size(&map), observe(&map)]);
    map.clear();
    second.set(add(value, 4));
    add(observed, second.get())
}
fn post_iteration_entry_mutation(value: i64, limit: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut turns = 0;
    loop {
        install(&mut map, "first", &first);
        delete(&mut map, "first");
        bump(&first, 1);
        turns = add(turns, 1);
        if turns >= limit {
            break;
        }
    }
    let observed = sum([first.get(), turns, size(&map), observe(&map)]);
    map.clear();
    second.set(add(value, 4));
    add(observed, second.get())
}
fn abrupt_iteration_entry_mutation(value: i64, stop: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut turns = 0;
    let mut tails = 0;
    while turns < 3 {
        install(&mut map, "first", &first);
        delete(&mut map, "first");
        bump(&first, 1);
        turns = add(turns, 1);
        if stop {
            break;
        }
        if turns < 3 {
            continue;
        }
        tails = add(tails, 1);
    }
    let observed = sum([first.get(), turns, size(&map), tails, observe(&map)]);
    map.clear();
    second.set(add(value, 4));
    add(observed, second.get())
}
fn counted_update_entry_mutation(value: i64, stop: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut turns = 0;
    let mut observed_size = 0;
    let mut step = 0;
    while step < 3 {
        install(&mut map, "first", &first);
        delete(&mut map, "first");
        bump(&first, 1);
        install(&mut map, "temporary", &third);
        turns = add(turns, 1);
        if stop && turns == 2 {
            delete(&mut map, "temporary");
            break;
        }
        observed_size = add(observed_size, size(&map));
        delete(&mut map, "temporary");
        step = add(step, 1);
    }
    third.set(add(value, 4));
    let observed = sum([
        first.get(),
        turns,
        size(&map),
        observed_size,
        third.get(),
        observe(&map),
    ]);
    map.clear();
    observed
}
fn finally_entry_mutation(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    first.set(add(value, 4));
    let observed = sum([first.get(), size(&map), observe(&map)]);
    map.clear();
    add(observed, first.get())
}
fn cleanup_iteration_mutation(value: i64, stop: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut turns = 0;
    let mut tails = 0;
    let mut observed = 0;
    while turns < 3 {
        install(&mut map, "first", &first);
        delete(&mut map, "first");
        bump(&first, 1);
        turns = add(turns, 1);
        let leave = stop;
        if !leave && turns >= 3 {
            tails = add(tails, 1);
        }
        observed = sum([observed, size(&map), observe(&map)]);
        if leave {
            break;
        }
        if turns < 3 {
            continue;
        }
    }
    map.clear();
    second.set(add(value, 4));
    sum([first.get(), turns, tails, observed, second.get()])
}
fn return_cleanup_entry_result(value: i64, mode: i64, trace: &mut [i64]) -> Result<i64, i64> {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut turns = 0;
    while turns < 3 {
        install(&mut map, "first", &first);
        delete(&mut map, "first");
        bump(&first, 1);
        turns = add(turns, 1);
        let pending = if turns == 2 {
            Some(if mode == 3 || mode == 4 {
                Err(first.get())
            } else {
                Ok(first.get())
            })
        } else {
            None
        };
        trace[0] = sum([trace[0], mul(turns, 10), size(&map)]);
        trace[1] = add(trace[1], observe(&map));
        if turns == 2 {
            if mode == 1 {
                return Ok(add(value, 20));
            }
            if mode == 2 {
                continue;
            }
            if mode == 4 {
                return Ok(add(value, 30));
            }
        }
        if let Some(result) = pending {
            return result;
        }
    }
    map.clear();
    Ok(first.get())
}
fn return_cleanup_entry_mutation(value: i64, mode: i64) -> i64 {
    let mut trace = [0, 0];
    let result = return_cleanup_entry_result(value, mode, &mut trace).unwrap_or_else(|error| error);
    sum([result, trace[0], trace[1]])
}
fn nested_cleanup_entry_mutation(value: i64, stop: bool, override_inner: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut turns = 0;
    let mut tails = 0;
    let mut inner = 0;
    let mut observed = 0;
    'outer: while turns < 3 {
        let control = loop {
            install(&mut map, "first", &first);
            delete(&mut map, "first");
            bump(&first, 1);
            turns = add(turns, 1);
            let control = if stop {
                1
            } else if turns < 3 {
                2
            } else {
                0
            };
            inner = add(inner, turns);
            if override_inner && turns == 1 {
                continue;
            }
            break control;
        };
        if control == 0 {
            tails = add(tails, 1);
        }
        observed = sum([observed, size(&map), observe(&map)]);
        if control == 1 {
            break 'outer;
        }
        if control == 2 {
            continue 'outer;
        }
    }
    map.clear();
    second.set(add(value, 4));
    sum([
        first.get(),
        second.get(),
        turns,
        tails,
        mul(inner, 100),
        observed,
    ])
}
fn direct_nested_entry_mutation(value: i64, stop: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut turns = 0;
    let mut tails = 0;
    'outer: while turns < 3 {
        loop {
            install(&mut map, "first", &first);
            delete(&mut map, "first");
            bump(&first, 1);
            turns = add(turns, 1);
            if stop {
                break 'outer;
            }
            if turns < 3 {
                continue 'outer;
            }
            break;
        }
        tails = add(tails, 1);
    }
    let observed = observe(&map);
    map.clear();
    sum([first.get(), second.get(), observed, turns, tails])
}
fn labelled_block_entry_mutation(value: i64, stop: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "second", &second);
    let mut observed = 0;
    'exit: {
        install(&mut map, "first", &first);
        if !stop {
            delete(&mut map, "first");
            bump(&first, 1);
            install(&mut map, "third", &third);
        }
        observed = sum([observed, size(&map), observe(&map)]);
        if stop {
            break 'exit;
        }
    }
    observed = add(observed, observe(&map));
    map.clear();
    sum([first.get(), second.get(), third.get(), observed])
}
// The labelled loop/block executes once on every path; the same state transitions apply.
fn labelled_loop_block_entry_mutation(value: i64, stop: bool) -> i64 {
    labelled_block_entry_mutation(value, stop)
}
fn ordinary_entry_edges(value: i64, limit: i64, stop: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "old", &first);
    let mut turns = 0;
    while turns < limit {
        delete(&mut map, "old");
        bump(&first, 1);
        turns = add(turns, 1);
        if stop {
            break;
        }
        install(&mut map, "new", &second);
        if turns < limit {
            continue;
        }
    }
    let observed = add(size(&map), observe(&map));
    map.clear();
    sum([first.get(), second.get(), turns, observed])
}
fn narrowed_ordinary_entry_edges(value: i64, limit: i64, stop: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let discard = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "discard", &discard);
    delete(&mut map, "discard");
    bump(&discard, 1);
    install(&mut map, "old", &first);
    let mut turns = 0;
    while turns < limit {
        delete(&mut map, "old");
        bump(&first, 1);
        turns = add(turns, 1);
        if stop {
            break;
        }
        install(&mut map, "new", &second);
        if turns < limit {
            continue;
        }
    }
    let observed = add(size(&map), observe(&map));
    map.clear();
    sum([first.get(), second.get(), turns, observed, discard.get()])
}
fn no_backedge_entry_break(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    loop {
        delete(&mut map, "first");
        bump(&first, 1);
        break;
    }
    let observed = add(size(&map), observe(&map));
    map.clear();
    bump(&first, 10);
    add(first.get(), observed)
}
fn no_backedge_entry_return(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    loop {
        delete(&mut map, "first");
        bump(&first, 1);
        return first.get();
    }
}
fn phase_join_failure(fail: bool) -> Result<(), i64> {
    if fail { Err(913) } else { Ok(()) }
}
fn phase_joined_entry(value: i64, fail: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    match phase_join_failure(fail) {
        Ok(_) => {
            delete(&mut map, "first");
        }
        Err(_) => {
            delete(&mut map, "first");
            bump(&first, 2);
        }
    }
    bump(&first, 1);
    let observed = add(size(&map), observe(&map));
    map.clear();
    add(first.get(), observed)
}
fn finally_normal_joined_entry(value: i64, fail: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let caught = i64::from(phase_join_failure(fail).is_err());
    delete(&mut map, "first");
    bump(&first, 1);
    let observed = add(size(&map), observe(&map));
    map.clear();
    sum([first.get(), observed, caught])
}
fn error_phase_entry(value: i64, fail: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    if phase_join_failure(fail).is_err() {
        bump(&first, 1);
    }
    let observed = add(size(&map), observe(&map));
    map.clear();
    add(first.get(), observed)
}
fn finally_error_entry(value: i64, fail: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    let mut observed = 0;
    if phase_join_failure(fail).is_err() {
        bump(&first, 1);
        observed = add(observed, 10);
    }
    observed = sum([observed, size(&map), observe(&map)]);
    map.clear();
    add(first.get(), observed)
}
fn finally_pending_error(value: i64, fail: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    let pending = phase_join_failure(fail);
    let mut observed = add(size(&map), observe(&map));
    map.clear();
    let caught = if pending.is_err() {
        10
    } else {
        observed = add(observed, 4);
        0
    };
    bump(&first, 1);
    sum([first.get(), observed, caught])
}
fn handler_rethrow_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let initial: Result<(), i64> = Err(17);
    let pending = match initial {
        Err(_) => {
            delete(&mut map, "first");
            bump(&first, 1);
            Err(18)
        }
        Ok(_) => Ok(()),
    };
    let mut observed = 0;
    if pending.is_err() {
        bump(&first, 1);
        observed = add(observed, observe(&map));
    }
    map.clear();
    add(first.get(), observed)
}
fn cleanup_outgoing_entry(value: i64, override_error: bool) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut pending: Result<(), i64> = Err(27);
    delete(&mut map, "first");
    bump(&first, 1);
    if override_error {
        pending = Err(28);
    }
    let mut observed = 0;
    if let Err(error) = pending {
        bump(&first, 1);
        observed = sum([observed, error, observe(&map)]);
    }
    map.clear();
    add(first.get(), observed)
}
fn catch_finally_entry(value: i64, mode: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    let mut pending: Result<(), i64> = if mode != 0 { Err(37) } else { Ok(()) };
    if pending.is_err() {
        bump(&first, 1);
        pending = if mode >= 2 { Err(38) } else { Ok(()) };
    }
    bump(&first, 1);
    let mut observed = add(size(&map), observe(&map));
    if mode == 3 {
        pending = Err(39);
    }
    if let Err(error) = pending {
        bump(&first, 1);
        observed = add(observed, error);
    }
    map.clear();
    add(first.get(), observed)
}
fn return_carrier_result(value: i64, mode: i64, trace: &mut [i64]) -> Result<i64, i64> {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    delete(&mut map, "first");
    bump(&first, 1);
    let pending = first.get();
    bump(&first, 10);
    trace[0] = sum([trace[0], size(&map), observe(&map)]);
    match mode {
        1 => Ok(first.get()),
        2 => Err(first.get()),
        _ => Ok(pending),
    }
}
fn return_carrier_entry(value: i64, mode: i64) -> i64 {
    let mut trace = [0];
    let result = return_carrier_result(value, mode, &mut trace).unwrap_or_else(|e| e);
    add(result, trace[0])
}
// The unconditional caught error precedes the same body/cleanup state transitions.
fn catch_return_carrier_result(value: i64, mode: i64, trace: &mut [i64]) -> Result<i64, i64> {
    return_carrier_result(value, mode, trace)
}
fn catch_return_carrier_entry(value: i64, mode: i64) -> i64 {
    let mut trace = [0];
    let result = catch_return_carrier_result(value, mode, &mut trace).unwrap_or_else(|e| e);
    add(result, trace[0])
}
fn control_carrier_entry(value: i64, mode: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut turn = 0;
    let mut observed = 0;
    while turn < 2 {
        if mode == 0 {
            delete(&mut map, "first");
            bump(&first, 1);
            observed = add(observed, first.get());
            observed = sum([observed, size(&map), observe(&map)]);
            break;
        }
        if mode == 1 {
            delete(&mut map, "second");
            bump(&second, 1);
            observed = add(observed, second.get());
            observed = sum([observed, size(&map), observe(&map)]);
            turn = add(turn, 1);
            continue;
        }
        delete(&mut map, "first");
        bump(&first, 1);
        observed = add(observed, first.get());
        turn = add(turn, 1);
        observed = sum([observed, size(&map), observe(&map)]);
    }
    map.clear();
    sum([observed, first.get(), second.get()])
}
fn handler_control_carrier_entry(value: i64, mode: i64) -> i64 {
    control_carrier_entry(value, mode)
}
// Every pending return/error/control edge is overridden by the cleanup's loop edge.
fn cleanup_override_entry(value: i64, _mode: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut map = Map::new();
    install(&mut map, "first", &first);
    install(&mut map, "second", &second);
    let mut turn = 0;
    let mut observed = 0;
    while turn < 2 {
        delete(&mut map, "first");
        bump(&first, 1);
        turn = add(turn, 1);
        observed = sum([observed, turn, size(&map), observe(&map)]);
        if turn < 2 {
            continue;
        }
        break;
    }
    map.clear();
    sum([first.get(), second.get(), observed])
}
fn handler_cleanup_override_entry(value: i64, mode: i64) -> i64 {
    cleanup_override_entry(value, mode)
}
fn held(source: &Cell<i64>) -> Holder<'_> {
    Holder {
        run: Box::new(move || source.get()),
    }
}
fn constant(value: i64) -> Holder<'static> {
    Holder {
        run: Box::new(move || value),
    }
}
fn indexed_replacement_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    entries[1] = held(&first);
    bump(&second, 2);
    sum([entries[0].call(), entries[1].call(), second.get()])
}
struct IndexedOwner<'a> {
    handlers: Vec<Holder<'a>>,
    marker: i64,
}
fn indexed_field_replacement_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut owner = IndexedOwner {
        handlers: vec![held(&first), held(&second)],
        marker: 7,
    };
    owner.handlers[1] = held(&first);
    bump(&second, 2);
    sum([
        owner.handlers[0].call(),
        owner.handlers[1].call(),
        second.get(),
        owner.marker,
    ])
}
fn indexed_moved_replacement_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let original = vec![held(&first), held(&second)];
    let mut moved = original;
    moved[1] = held(&first);
    bump(&second, 3);
    sum([moved[0].call(), moved[1].call(), second.get()])
}
fn indexed_pop_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    entries.pop();
    bump(&second, 2);
    add(entries[0].call(), second.get())
}
fn indexed_shift_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    entries.remove(0);
    bump(&first, 3);
    add(entries[0].call(), first.get())
}
fn indexed_empty_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    entries.pop();
    entries.pop();
    entries.pop();
    bump(&first, 2);
    bump(&second, 3);
    sum([
        i64::try_from(entries.len()).unwrap(),
        first.get(),
        second.get(),
    ])
}
fn indexed_retained_entry(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    let observed = {
        let removed = entries.pop();
        removed.as_ref().map_or(0, Holder::call)
    };
    bump(&second, 2);
    sum([observed, entries[0].call(), second.get()])
}
fn indexed_independent_result(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    let removed = entries.pop();
    entries[0] = held(&second);
    bump(&first, 3);
    let observed = removed.as_ref().map_or(0, Holder::call);
    sum([observed, entries[0].call(), first.get()])
}
fn indexed_mutable_result(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    let mut removed = entries.pop();
    entries[0] = held(&second);
    bump(&first, 3);
    let observed = removed.as_ref().map_or(0, Holder::call);
    removed = None;
    drop(removed);
    sum([observed, entries[0].call(), first.get()])
}
fn indexed_replaced_result(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let mut entries = vec![held(&first), held(&second)];
    let mut removed = entries.pop();
    removed = Some(held(&third));
    bump(&second, 3);
    let observed = removed.as_ref().map_or(0, Holder::call);
    sum([observed, entries[0].call(), second.get()])
}
fn indexed_comma_removal(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    entries.pop();
    bump(&second, 2);
    add(entries[0].call(), second.get())
}
fn indexed_nested_comma_removal(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    entries.remove(0);
    bump(&first, 3);
    entries.pop();
    bump(&second, 2);
    entries.pop();
    sum([
        first.get(),
        second.get(),
        i64::try_from(entries.len()).unwrap(),
    ])
}
fn indexed_retained_comma_result(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut entries = vec![held(&first), held(&second)];
    let observed = {
        entries.pop();
        bump(&second, 2);
        let removed = if entries.is_empty() {
            None
        } else {
            Some(entries.remove(0))
        };
        removed.as_ref().map_or(0, Holder::call)
    };
    bump(&first, 3);
    sum([observed, first.get(), second.get()])
}
fn indexed_comma_reassignment(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let mut entries = vec![held(&first), held(&second), held(&third)];
    let mut removed = entries.pop();
    removed = {
        entries.pop();
        bump(&second, 2);
        if entries.is_empty() {
            None
        } else {
            Some(entries.remove(0))
        }
    };
    bump(&third, 3);
    let observed = removed.as_ref().map_or(0, Holder::call);
    removed = None;
    drop(removed);
    bump(&first, 4);
    sum([observed, first.get(), second.get(), third.get()])
}
fn nullable_record_replacement(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut saved: Option<Holder<'_>> = None;
    saved = Some(held(&first));
    saved = Some(held(&second));
    bump(&first, 3);
    let observed = saved.as_ref().map_or(0, Holder::call);
    saved = None;
    drop(saved);
    bump(&second, 4);
    sum([observed, first.get(), second.get()])
}
fn nullable_record_null_result(value: i64) -> i64 {
    let values = Cell::new(value);
    let mut saved: Option<Holder<'_>> = None;
    saved = Some(held(&values));
    let observed = saved.as_ref().map_or(0, Holder::call);
    saved = None;
    drop(saved);
    bump(&values, 2);
    add(observed, values.get())
}
struct NullablePair<'a> {
    left: Option<Holder<'a>>,
    right: Option<Holder<'a>>,
}
struct NullableArray<'a> {
    saved: Option<Holder<'a>>,
    items: Vec<Holder<'a>>,
}
struct Slot<'a> {
    saved: Option<Holder<'a>>,
}
fn nullable_field_replacement(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let third = Cell::new(add(value, 2));
    let mut pair = NullablePair {
        left: None,
        right: None,
    };
    pair.left = Some(held(&first));
    pair.right = Some(held(&second));
    pair.left = Some(held(&third));
    bump(&first, 3);
    let observed = match (&pair.left, &pair.right) {
        (Some(left), Some(right)) => add(left.call(), right.call()),
        _ => 0,
    };
    pair.left = None;
    bump(&third, 4);
    pair.right = None;
    bump(&second, 5);
    sum([observed, first.get(), second.get(), third.get()])
}
fn nullable_field_array(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 1));
    let mut record = NullableArray {
        saved: None,
        items: vec![held(&first)],
    };
    record.saved = Some(held(&second));
    record.items.pop();
    bump(&first, 2);
    let observed = record.saved.as_ref().map_or(0, |saved| {
        sum([
            saved.call(),
            first.get(),
            i64::try_from(record.items.len()).unwrap(),
        ])
    });
    record.saved = None;
    bump(&second, 3);
    add(observed, second.get())
}
fn nullable_control_stores(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 2));
    let mut pair = NullablePair {
        left: None,
        right: None,
    };
    let mut turn = 0;
    while turn < 3 {
        if turn % 2 == 0 {
            pair.left = Some(held(&first));
        } else {
            pair.right = Some(held(&second));
        }
        turn = add(turn, 1);
    }
    if value % 2 == 0 {
        pair.right = None;
    }
    sum([
        pair.left.as_ref().map_or(0, Holder::call),
        pair.right.as_ref().map_or(0, Holder::call),
        turn,
    ])
}
fn nullable_owned_result(seed: i64) -> Option<Holder<'static>> {
    if seed % 2 == 0 {
        Some(Holder {
            run: Box::new(move || add(seed, 1)),
        })
    } else {
        None
    }
}
fn nullable_value_joins(value: i64) -> i64 {
    let first = Cell::new(value);
    let second = Cell::new(add(value, 5));
    let made = nullable_owned_result(value);
    let mut observed = made.as_ref().map_or(0, Holder::call);
    let chosen = if value % 2 == 0 {
        Some(held(&first))
    } else {
        None
    };
    observed = add(observed, chosen.as_ref().map_or(0, Holder::call));
    let mut slots = vec![Slot { saved: None }, Slot { saved: None }];
    slots[usize::try_from(value % 2).unwrap()].saved = Some(held(&second));
    for slot in &slots {
        observed = add(observed, slot.saved.as_ref().map_or(0, Holder::call));
    }
    add(observed, i64::try_from(slots.len()).unwrap())
}
fn nullable_field_reader(slot: &Slot<'_>) -> i64 {
    slot.saved.as_ref().map_or(1, Holder::call)
}
fn nullable_parameter_read(value: i64) -> i64 {
    let first = Cell::new(value);
    let mut slot = Slot { saved: None };
    let before = nullable_field_reader(&slot);
    if value % 2 == 0 {
        slot.saved = Some(Holder {
            run: Box::new(|| add(first.get(), 2)),
        });
    }
    add(before, nullable_field_reader(&slot))
}
fn nullable_owned_join(value: i64) -> i64 {
    let mut chosen = None;
    if value % 2 == 0 {
        chosen = Some(Holder {
            run: Box::new(move || add(value, 2)),
        });
    } else if value > 2 {
        chosen = Some(Holder {
            run: Box::new(move || mul(value, 3)),
        });
    }
    chosen.as_ref().map_or(4, Holder::call)
}
fn nullable_field_clearer(slot: &mut Slot<'_>, limit: i64) -> i64 {
    let seen = slot.saved.as_ref().map_or(0, Holder::call);
    if slot.saved.is_some() && seen > limit {
        slot.saved = None;
    }
    seen
}
fn nullable_element_slots(value: i64) -> i64 {
    let first = Cell::new(value);
    let mut slots = vec![None, Some(held(&first))];
    slots[0] = Some(Holder {
        run: Box::new(|| add(first.get(), 2)),
    });
    slots.push(None);
    let mut observed = 0;
    for slot in &slots {
        observed = add(observed, slot.as_ref().map_or(1, Holder::call));
    }
    let removed = slots.remove(0);
    observed = add(observed, removed.as_ref().map_or(0, Holder::call));
    let mut kept = Slot {
        saved: Some(Holder {
            run: Box::new(move || add(value, 4)),
        }),
    };
    observed = add(
        observed,
        nullable_field_clearer(&mut kept, if value % 2 == 0 { 0 } else { 100 }),
    );
    sum([
        observed,
        i64::try_from(slots.len()).unwrap(),
        if kept.saved.is_none() { 10 } else { 20 },
    ])
}
fn container_total(handlers: &[Holder<'_>]) -> i64 {
    handlers
        .iter()
        .fold(0, |total, handler| add(total, handler.call()))
}
fn container_lookup(table: &Map<'_>, key: &str) -> i64 {
    get(table, key).map_or(1, Holder::call)
}
fn container_boundaries(value: i64) -> i64 {
    let first = Cell::new(value);
    let mut handlers = Vec::new();
    handlers.push(held(&first));
    handlers.push(Holder {
        run: Box::new(move || add(value, 2)),
    });
    let mut table = Map::new();
    put(&mut table, "first", || add(first.get(), 3));
    sum([
        container_total(&handlers),
        container_lookup(&table, if value % 2 == 0 { "first" } else { "none" }),
        i64::try_from(handlers.len()).unwrap(),
    ])
}
fn borrowed_nullable_reader(holder: Option<&Holder<'_>>) -> i64 {
    holder.map_or(1, Holder::call)
}
fn borrowed_nullable_views(value: i64) -> i64 {
    let values = Cell::new(value);
    let mut records = vec![Slot { saved: None }];
    let mut observed = add(
        borrowed_nullable_reader(None),
        borrowed_nullable_reader(None),
    )
    .checked_sub(1)
    .unwrap();
    records[0].saved = Some(held(&values));
    let saved = records[0].saved.as_ref();
    observed = sum([
        observed,
        borrowed_nullable_reader(saved),
        borrowed_nullable_reader(saved),
    ]);
    observed = add(
        observed,
        borrowed_nullable_reader(records[0].saved.as_ref()),
    );
    add(observed, i64::try_from(records.len()).unwrap())
}
fn install_nullable_owned(slot: &mut Slot<'_>, seed: i64) {
    let replacement = Some(constant(seed));
    slot.saved = replacement;
}
fn nullable_parameter_stores(value: i64) -> i64 {
    let mut slot = Slot { saved: None };
    let mut observed = slot.saved.as_ref().map_or(1, Holder::call);
    install_nullable_owned(&mut slot, value);
    observed = add(observed, slot.saved.as_ref().map_or(1, Holder::call));
    install_nullable_owned(&mut slot, add(value, 2));
    observed = add(observed, slot.saved.as_ref().map_or(1, Holder::call));
    slot.saved = None;
    add(observed, i64::from(slot.saved.is_none()))
}
fn independent_parameter_read(left: &Slot<'_>, right: &Slot<'_>) -> i64 {
    add(
        left.saved.as_ref().map_or(1, Holder::call),
        right.saved.as_ref().map_or(1, Holder::call),
    )
}
fn independent_parameter_fill(left: &mut Slot<'_>, right: &mut Slot<'_>, seed: i64) {
    left.saved = Some(constant(seed));
    right.saved = Some(Holder {
        run: Box::new(move || add(seed, 2)),
    });
}
fn independent_array_read(left: &[Holder<'_>], right: &[Holder<'_>]) -> i64 {
    add(left[0].call(), right[0].call())
}
fn independent_map_read(left: &Map<'_>, right: &Map<'_>) -> i64 {
    let mut observed = add(
        get(left, "k").map_or(1, Holder::call),
        get(right, "k").map_or(1, Holder::call),
    );
    observed = add(observed, observe(left));
    add(observed, observe(right))
}
fn independent_nullable_parameters(value: i64) -> i64 {
    let left_capture = Cell::new(value);
    let right_capture = Cell::new(add(value, 3));
    let mut left = Slot {
        saved: Some(held(&left_capture)),
    };
    let mut right = Slot {
        saved: Some(held(&right_capture)),
    };
    let observed = add(
        independent_parameter_read(&left, &right),
        independent_parameter_read(&right, &left),
    );
    independent_parameter_fill(&mut left, &mut right, add(value, 5));
    let left_array = vec![held(&left_capture)];
    let right_array = vec![held(&right_capture)];
    let mut left_map = Map::new();
    let mut right_map = Map::new();
    install(&mut left_map, "k", &left_capture);
    install(&mut right_map, "k", &right_capture);
    sum([
        observed,
        independent_parameter_read(&left, &right),
        independent_array_read(&left_array, &right_array),
        independent_map_read(&left_map, &right_map),
    ])
}

fn renewed(value: i64, fail: bool, branch: &mut i64) -> (i64, i64) {
    let old = Cell::new(value);
    let next = Cell::new(add(value, 12));
    let mut map = Map::new();
    install(&mut map, "old", &old);
    *branch = add(*branch, if value % 2 == 0 { 1 } else { 2 });
    let mut observed = match prefix_result(fail) {
        Ok(v) => v,
        Err(_) => add(2, size(&map)),
    };
    map.clear();
    install(&mut map, "new", &next);
    let cleanup = size(&map);
    old.set(add(value, 13));
    observed = add(observed, call(&map, "new"));
    map.clear();
    next.set(add(value, 14));
    (sum([observed, old.get(), next.get()]), cleanup)
}
fn consumed(value: i64, fail: bool) -> i64 {
    let old = Cell::new(value);
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let path = prefix_result(fail).unwrap_or(2);
    map.clear();
    let moved = map;
    let count = size(&moved);
    old.set(add(value, 20));
    sum([count, path, old.get()])
}
fn restored(value: i64, early: bool) -> i64 {
    let old = Cell::new(value);
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let mut count = -1;
    let pending: Result<(), i64> = (|| {
        prefix_result(early)?;
        map.clear();
        let moved = std::mem::take(&mut map);
        count = size(&moved);
        Err(7)
    })();
    if pending.is_err() {
        map = Map::new();
    }
    old.set(add(value, 30));
    sum([count, size(&map), old.get()])
}
fn expression(value: i64, early: bool) -> i64 {
    let old = Cell::new(value);
    let mut map = Map::new();
    install(&mut map, "old", &old);
    let mut saved = Map::new();
    let mut result = -1;
    let pending: Result<(), i64> = (|| {
        map.clear();
        if early {
            prefix_result(true)?;
            saved = std::mem::take(&mut map);
            result = 1;
        } else {
            saved = std::mem::take(&mut map);
            result = prefix_result(true)?;
        }
        Err(8)
    })();
    if pending.is_err() {
        map = Map::new();
    }
    old.set(add(value, if early { 40 } else { 41 }));
    sum([result, size(&saved), size(&map), old.get()])
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let _ = value2;
    let mut output = Vec::new();
    let mut i = 0;
    while i < value0 {
        let value = sum([i % 5, value1 % 5, 1]);
        let even = value % 2 == 0;
        let limit = if even { 0 } else { 2 };
        let base0 = Cell::new(value);
        let base1 = Cell::new(add(value, 1));
        let (
            removed_value,
            array_value,
            map_first,
            map_second,
            replaced,
            iterated,
            peer_value,
            populated,
            cleared,
        ) = {
            let mut items = vec![held(&base0)];
            let removed = items.pop();
            items.push(held(&base1));
            let removed_value = removed.as_ref().map_or(0, Holder::call);
            let array_value = items[0].call();
            let mut handlers = Map::new();
            install(&mut handlers, "first", &base0);
            let mut peer = Map::new();
            put(&mut peer, "peer", || add(base1.get(), 2));
            let mut boxes = vec![Store { handlers }, Store { handlers: peer }];
            install(&mut boxes[0].handlers, "second", &base1);
            let first = call(&boxes[0].handlers, "first");
            let second = call(&boxes[0].handlers, "second");
            let peer_value = call(&boxes[1].handlers, "peer");
            let populated = size(&boxes[0].handlers);
            let iterated = observe(&boxes[0].handlers);
            let mut replacement = Map::new();
            put(&mut replacement, "replacement", || add(base1.get(), 1));
            boxes[0].handlers = replacement;
            let replaced = call(&boxes[0].handlers, "replacement");
            boxes[0].handlers.clear();
            (
                removed_value,
                array_value,
                first,
                second,
                replaced,
                iterated,
                peer_value,
                populated,
                size(&boxes[0].handlers),
            )
        };
        let old = Cell::new(value);
        let next = Cell::new(add(value, 4));
        let mut refill = Refill {
            marker: 1,
            handlers: Map::new(),
        };
        install(&mut refill.handlers, "old", &old);
        refill.handlers.clear();
        install(&mut refill.handlers, "new", &next);
        old.set(add(value, 5));
        let mut refilled = call(&refill.handlers, "new");
        let moved_size = if even {
            refill.handlers.clear();
            let moved = std::mem::take(&mut refill.handlers);
            size(&moved)
        } else {
            refill.handlers.clear();
            size(&refill.handlers)
        };
        next.set(add(value, 6));
        refilled = sum([refilled, old.get(), next.get(), refill.marker]);
        let before = Cell::new(value);
        let after = Cell::new(add(value, 8));
        let mut caught = Map::new();
        install(&mut caught, "old", &before);
        caught.clear();
        install(&mut caught, "new", &after);
        let thrown: Result<(), i64> = Err(value);
        let mut caught_value = if thrown.is_err() { size(&caught) } else { 0 };
        before.set(add(value, 9));
        caught_value = add(caught_value, call(&caught, "new"));
        caught.clear();
        after.set(add(value, 10));
        caught_value = sum([caught_value, before.get(), after.get()]);
        let mut branch_trace = 0;
        let (normal_renewed, normal_cleanup) = renewed(value, false, &mut branch_trace);
        let (caught_renewed, caught_cleanup) = renewed(value, true, &mut branch_trace);
        let normal_consumed = consumed(value, false);
        let caught_consumed = consumed(value, true);
        let early_restored = restored(value, true);
        let late_restored = restored(value, false);
        let early_expression = expression(value, true);
        let late_expression = expression(value, false);
        let conditional_renewed = {
            let old = Cell::new(value);
            let mut owner = Store {
                handlers: Map::new(),
            };
            install(&mut owner.handlers, "old", &old);
            let count = if even {
                owner.handlers.clear();
                let moved = std::mem::take(&mut owner.handlers);
                size(&moved)
            } else {
                size(&owner.handlers)
            };
            let thrown: Result<(), i64> = Err(9);
            if thrown.is_err() {
                owner.handlers = Map::new();
            }
            old.set(add(value, 50));
            sum([count, size(&owner.handlers), old.get()])
        };
        let survivor = Cell::new(value);
        let mut normal_survivor = {
            let mut map = Map::new();
            install(&mut map, "old", &survivor);
            prefix_result(false).unwrap();
            let count = if even {
                map.clear();
                let moved = map;
                size(&moved)
            } else {
                size(&map)
            };
            add(count, survivor.get())
        };
        survivor.set(add(value, 51));
        normal_survivor = add(normal_survivor, survivor.get());
        let outside_restored = {
            let old = Cell::new(value);
            let mut map = Map::new();
            install(&mut map, "old", &old);
            let count = if even {
                map.clear();
                let moved = std::mem::take(&mut map);
                size(&moved)
            } else {
                size(&map)
            };
            map = Map::new();
            old.set(add(value, 60));
            sum([count, size(&map), old.get()])
        };
        let mut trace = [0, -1];
        let finally_renewed = sum([
            no_catch_renewal(even, value, &mut trace).unwrap_or(7),
            mul(trace[0], 100),
            trace[1],
        ]);
        let mut trace = [0, -1];
        let finally_returned = sum([
            no_catch_return(even, value, &mut trace),
            mul(trace[0], 100),
            trace[1],
        ]);
        let mut trace = [0, 0];
        let finally_looped = sum([
            no_catch_loop(value, &mut trace),
            mul(trace[0], 100),
            trace[1],
        ]);
        let mut trace = [0, 0];
        let finally_outer_looped = sum([
            no_catch_outer_loop(value, &mut trace),
            mul(trace[0], 100),
            trace[1],
        ]);
        let initialized_record_map = nested_map_initializer(value);
        let mut trace = [0, 0];
        let looped_record_map = sum([
            record_map_loop(value, &mut trace),
            mul(trace[0], 100),
            trace[1],
        ]);
        let mut trace = [0, 0, 0, 0];
        let nested_cleaned = sum([
            nested_map_cleanup(even, value, &mut trace).unwrap_or(7),
            mul(trace[0], 100),
            mul(trace[1], 10),
            mul(trace[2], 1000),
            trace[3],
        ]);
        base0.set(0);
        let mut trace = [0, 0, 0, 0];
        let nested_caught = sum([
            nested_caught_map_cleanup(even, value, &mut trace),
            mul(trace[0], 100),
            mul(trace[1], 10),
            mul(trace[2], 1000),
            trace[3],
        ]);
        let mut trace = [0, 0, 0, 0];
        let nested_abrupt = sum([
            nested_abrupt_cleanup(even, value, &mut trace).unwrap_or(11),
            mul(trace[0], 100),
            mul(trace[1], 10),
            mul(trace[2], 1000),
            trace[3],
        ]);
        let mut trace = [0, 0, 0];
        let nested_returned = sum([
            nested_returned_cleanup(even, value, &mut trace),
            mul(trace[0], 100),
            mul(trace[1], 10),
            mul(trace[2], 1000),
        ]);
        let mut trace = [0, 0];
        let caught_looped = sum([
            caught_map_loop(value, &mut trace),
            mul(trace[0], 100),
            trace[1],
        ]);
        let mut trace = [0, 0];
        let paired_looped = sum([
            paired_map_loop(value, &mut trace),
            mul(trace[0], 100),
            trace[1],
        ]);
        let mut trace = [0, 0];
        let nested_pair_looped = sum([
            nested_paired_map_loop(value, &mut trace),
            mul(trace[0], 100),
            trace[1],
        ]);
        let for_of_map_observed = for_of_map_loop();
        let counted_map_observed = counted_map_loop();
        let post_map_observed = post_map_loop();
        let empty_map_observed = empty_scoped_map();
        output.push(NumericArray(vec![
            removed_value,
            array_value,
            map_first,
            map_second,
            replaced,
            iterated,
            peer_value,
            populated,
            cleared,
            add(base0.get(), base1.get()),
            refilled,
            moved_size,
            caught_value,
            normal_renewed,
            caught_renewed,
            normal_cleanup,
            caught_cleanup,
            branch_trace,
            normal_consumed,
            caught_consumed,
            early_restored,
            late_restored,
            early_expression,
            late_expression,
            conditional_renewed,
            normal_survivor,
            outside_restored,
            finally_renewed,
            finally_returned,
            finally_looped,
            finally_outer_looped,
            initialized_record_map,
            looped_record_map,
            nested_cleaned,
            nested_caught,
            nested_abrupt,
            nested_returned,
            caught_looped,
            paired_looped,
            nested_pair_looped,
            empty_map_observed,
            counted_map_observed,
            post_map_observed,
            for_of_map_observed,
            known_entry_release(value),
            moved_field_entry_release(value),
            replacement_entry_outcome(value, !even),
            branch_entry_release(value, even),
            mixed_branch_entry_release(value, even),
            paired_branch_entry_release(value, even),
            post_entry_loop_release(value, limit),
            counted_entry_loop_release(value, limit),
            for_of_entry_loop_release(value, limit),
            loop_entry_refill(value, limit),
            inner_branch_entry_release(value, even),
            branch_entry_refill(value, even),
            iteration_entry_mutation(value, limit),
            post_iteration_entry_mutation(value, limit),
            abrupt_iteration_entry_mutation(value, even),
            counted_update_entry_mutation(value, even),
            finally_entry_mutation(value),
            cleanup_iteration_mutation(value, even),
            return_cleanup_entry_mutation(value, 0),
            return_cleanup_entry_mutation(value, 1),
            return_cleanup_entry_mutation(value, 2),
            return_cleanup_entry_mutation(value, 3),
            return_cleanup_entry_mutation(value, 4),
            nested_cleanup_entry_mutation(value, even, false),
            nested_cleanup_entry_mutation(value, even, true),
            direct_nested_entry_mutation(value, even),
            labelled_block_entry_mutation(value, even),
            labelled_loop_block_entry_mutation(value, even),
            ordinary_entry_edges(value, limit, false),
            ordinary_entry_edges(value, 2, true),
            narrowed_ordinary_entry_edges(value, 2, false),
            no_backedge_entry_break(value),
            no_backedge_entry_return(value),
            phase_joined_entry(value, false),
            phase_joined_entry(value, true),
            finally_normal_joined_entry(value, !even),
            error_phase_entry(value, !even),
            finally_error_entry(value, !even),
            finally_pending_error(value, !even),
            handler_rethrow_entry(value),
            cleanup_outgoing_entry(value, !even),
            catch_finally_entry(value, 0),
            catch_finally_entry(value, 1),
            catch_finally_entry(value, 2),
            catch_finally_entry(value, 3),
            return_carrier_entry(value, 0),
            return_carrier_entry(value, 1),
            return_carrier_entry(value, 2),
            catch_return_carrier_entry(value, 0),
            catch_return_carrier_entry(value, 1),
            catch_return_carrier_entry(value, 2),
            control_carrier_entry(value, 0),
            control_carrier_entry(value, 1),
            control_carrier_entry(value, 2),
            handler_control_carrier_entry(value, 0),
            handler_control_carrier_entry(value, 1),
            handler_control_carrier_entry(value, 2),
            cleanup_override_entry(value, 0),
            cleanup_override_entry(value, 1),
            cleanup_override_entry(value, 2),
            cleanup_override_entry(value, 3),
            handler_cleanup_override_entry(value, 0),
            handler_cleanup_override_entry(value, 1),
            handler_cleanup_override_entry(value, 2),
            handler_cleanup_override_entry(value, 3),
            indexed_replacement_entry(value),
            indexed_field_replacement_entry(value),
            indexed_moved_replacement_entry(value),
            indexed_pop_entry(value),
            indexed_shift_entry(value),
            indexed_empty_entry(value),
            indexed_retained_entry(value),
            indexed_independent_result(value),
            indexed_mutable_result(value),
            indexed_replaced_result(value),
            indexed_comma_removal(value),
            indexed_nested_comma_removal(value),
            indexed_retained_comma_result(value),
            indexed_comma_reassignment(value),
            nullable_record_replacement(value),
            nullable_record_null_result(value),
            nullable_field_replacement(value),
            nullable_field_array(value),
            nullable_control_stores(value),
            nullable_value_joins(value),
            nullable_parameter_read(value),
            nullable_owned_join(value),
            nullable_element_slots(value),
            container_boundaries(value),
            borrowed_nullable_views(value),
            nullable_parameter_stores(value),
            independent_nullable_parameters(value),
        ]));
        i = add(i, 1);
    }
    Array(output)
}
fn main() {}
