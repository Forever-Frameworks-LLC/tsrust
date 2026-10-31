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

// Stable Rust uses generic methods to retain one mutable environment across types.
struct Receipt<T> {
    value: T,
    calls: i64,
}
struct PatternBox<T> {
    value: T,
}
struct Counter {
    calls: i64,
    step: i64,
}
impl Counter {
    fn select<T>(&mut self, value: T) -> Receipt<T> {
        self.calls = add(self.calls, self.step);
        Receipt {
            value,
            calls: self.calls,
        }
    }
    fn head<T>(&mut self, values: Vec<T>) -> Receipt<T> {
        let mut iter = values.into_iter();
        let first = iter.next().unwrap();
        let _second = iter.next().unwrap();
        self.select(first)
    }
    fn record<T>(&mut self, record: PatternBox<T>) -> Receipt<T> {
        self.select(record.value)
    }
    fn invoke<T, F: FnOnce(T) -> T>(&mut self, action: F, value: T) -> Receipt<T> {
        self.calls = add(self.calls, 1);
        Receipt {
            value: action(value),
            calls: self.calls,
        }
    }
}
fn make(start: i64) -> Counter {
    Counter {
        calls: start,
        step: 1,
    }
}
fn double_factory(start: i64) -> Counter {
    Counter {
        calls: start,
        step: 2,
    }
}
fn generic_factory<T: 'static>(_seed: T, start: i64) -> Counter {
    Counter {
        calls: start,
        step: if std::any::TypeId::of::<T>() == std::any::TypeId::of::<std::string::String>() {
            2
        } else {
            1
        },
    }
}
struct Inferred {
    calls: i64,
    start: i64,
}
impl Inferred {
    fn select<T>(&mut self, left: T, right: T) -> T {
        self.calls = add(self.calls, 1);
        if self.calls == add(self.start, 1) {
            left
        } else {
            right
        }
    }
}
fn inferred_factory(start: i64) -> Inferred {
    Inferred {
        calls: start,
        start,
    }
}
struct Reducer {
    calls: i64,
}
impl Reducer {
    fn select<T>(&mut self, left: T, right: T) -> T {
        self.calls = add(self.calls, 1);
        if self.calls == 4 { right } else { left }
    }
}
struct Predicate {
    calls: i64,
}
impl Predicate {
    fn accept<T>(&mut self, _value: T) -> bool {
        self.calls = add(self.calls, 1);
        self.calls == 2
    }
}
struct Comparator {
    seen: i64,
}
impl Comparator {
    fn compare<T: 'static>(&mut self, _left: &T, _right: &T) -> i64 {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<std::string::String>() {
            self.seen
        } else {
            self.seen = 1;
            0
        }
    }
}
struct Defaults {
    calls: i64,
}
impl Defaults {
    fn select<T>(&mut self, value: T, offset: Option<i64>) -> Receipt<T> {
        let calls = offset.unwrap_or_else(|| {
            self.calls = add(self.calls, 1);
            self.calls
        });
        Receipt { value, calls }
    }
}
struct Rest {
    calls: i64,
}
impl Rest {
    fn select<T>(&mut self, fallback: T, values: Vec<T>) -> Receipt<T> {
        self.calls = add(self.calls, 1);
        Receipt {
            value: fallback,
            calls: add(self.calls, i64::try_from(values.len()).unwrap()),
        }
    }
}
struct Payload {
    calls: i64,
    start: i64,
}
impl Payload {
    fn select<T: 'static>(&mut self, value: T) -> T {
        self.calls = add(self.calls, 1);
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<bool>() {
            assert_eq!(self.calls, add(self.start, 3));
        }
        value
    }
}
fn payload_factory(start: i64) -> Payload {
    Payload {
        calls: start,
        start,
    }
}
struct Selector {
    calls: i64,
    array: bool,
}
enum ScalarResult {
    Scalar(i64),
    Array(Vec<i64>),
}
impl Selector {
    fn select(&mut self, value: i64) -> ScalarResult {
        self.calls = add(self.calls, 1);
        if self.array {
            ScalarResult::Array(vec![value])
        } else {
            ScalarResult::Scalar(value)
        }
    }
}
struct Holder {
    run: Counter,
}
struct First;
impl First {
    fn call<T>(&self, value: T) -> T {
        value
    }
}
struct Second;
impl Second {
    fn call<T>(&self, _left: T, right: T) -> T {
        right
    }
}
fn retain_polymorphic<A>(action: A) -> impl FnOnce() -> A {
    move || action
}
fn lexical_echo<T>(value: T) -> T {
    fn echo<T>(other: T) -> T {
        other
    }
    echo(value)
}
fn lexical_capture<T>(value: T) -> impl FnOnce() -> T {
    move || value
}
fn consume_scoped<F: FnOnce() -> Receipt<i64>>(action: F) -> Receipt<i64> {
    action()
}
fn retain_once<F: FnOnce() -> Receipt<i64>>(action: F) -> F {
    action
}
fn sum_payload(value: Option<i64>, tail: Vec<i64>) -> i64 {
    tail.into_iter().fold(value.unwrap_or(4), add)
}
fn increment(value: i64) -> i64 {
    add(value, 1)
}
fn post(counter: &mut i64) -> i64 {
    let old = *counter;
    *counter = add(old, 1);
    old
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(add(i % 5, value1 % 5), 1);

        let mut action = make(base);
        let first = action.select(base);
        let first_value = first.value;
        let first_calls = first.calls;
        let generic = action.select("generic".to_owned());
        let mut owned = action;
        let last = owned.select("owner".to_owned());
        let mut mapper = make(add(base, 10));
        let mapped: Vec<_> = [base, add(base, 1)]
            .into_iter()
            .map(|value| mapper.select(value))
            .collect();
        let after = mapper.select(true);
        let mut holder = Holder {
            run: make(add(base, 20)),
        };
        let record = holder.run.select(add(base, 2));
        let mut actions = vec![make(add(base, 30))];
        let stored = actions[0].select("array".to_owned());
        let mut reduce = Reducer { calls: 0 };
        let reduced = [base, add(base, 1), add(base, 2)]
            .into_iter()
            .fold(add(base, 3), |a, v| reduce.select(a, v));
        let reduced_after = reduce.select("before".to_owned(), "fourth".to_owned());
        let mut reduce_right = Reducer { calls: 0 };
        let reversed = [base, add(base, 1), add(base, 2)]
            .into_iter()
            .rev()
            .fold(add(base, 4), |a, v| reduce_right.select(a, v));
        let reversed_after = reduce_right.select("before".to_owned(), "fourth".to_owned());
        let mut accept = Predicate { calls: 0 };
        let found = i64::try_from(
            [base, add(base, 1), add(base, 2)]
                .into_iter()
                .position(|v| accept.accept(v))
                .unwrap(),
        )
        .unwrap();
        let searched_after = accept.accept("after".to_owned());
        let mut compare = Comparator { seen: 0 };
        let mut sortable = vec![add(base, 2), base, add(base, 1)];
        sortable.sort_by(|a, b| compare.compare(a, b).cmp(&0));
        let mut copied = sortable.clone();
        copied.sort_by(|a, b| compare.compare(a, b).cmp(&0));
        let sorted_after = compare.compare(&"after".to_owned(), &"unused".to_owned());
        let mut defaults = Defaults { calls: 0 };
        let omitted = defaults.select(base, None);
        let default_value = omitted.value;
        let omitted_calls = omitted.calls;
        let override_calls = defaults.select(base, Some(9)).calls;
        let undefined_calls = defaults.select(base, None).calls;
        let defaults_after = defaults.select("after".to_owned(), None);
        let mut rest = Rest { calls: 0 };
        let empty = rest.select(base, vec![]);
        let empty_value = empty.value;
        let empty_calls = empty.calls;
        let packed_calls = rest
            .select(add(base, 1), vec![add(base, 2), add(base, 3)])
            .calls;
        let rest_after = rest.select("after".to_owned(), vec!["one".to_owned()]);
        let mut direct = make(add(base, 40));
        let direct_first = direct.select(add(base, 5));
        let direct_second = direct.select(add(base, 6));
        let mut mapping = make(add(base, 50));
        let factory_mapped: Vec<_> = [add(base, 7), add(base, 8)]
            .into_iter()
            .map(|v| mapping.select(v))
            .collect();
        let factory = make;
        let mut alias = factory(add(base, 60));
        let alias_value = alias.select(add(base, 9));
        let selected_factory = if false { factory } else { factory };
        let mut selected_mapper = selected_factory(add(base, 70));
        let selected_mapped: Vec<_> = [add(base, 11), add(base, 12)]
            .into_iter()
            .map(|v| selected_mapper.select(v))
            .collect();
        let mut generic_direct = generic_factory("string".to_owned(), add(base, 80));
        let generic_first = generic_direct.select(add(base, 13));
        let generic_second = generic_direct.select(add(base, 14));
        let mut generic_mapper = generic_factory(base, add(base, 90));
        let generic_mapped: Vec<_> = [add(base, 15), add(base, 16)]
            .into_iter()
            .map(|v| generic_mapper.select(v))
            .collect();
        let mut selections = 0;
        let mut evaluations = 0;
        let selected: fn(i64) -> Counter = if post(&mut selections) == 0 {
            make
        } else {
            double_factory
        };
        let mut distinct = selected(add(add(base, 100), post(&mut evaluations)));
        let distinct_first = distinct.select(add(base, 17));
        let distinct_second = distinct.select(add(base, 18));
        let selected: fn(i64) -> Counter = if post(&mut selections) == 0 {
            make
        } else {
            double_factory
        };
        let mut distinct_mapper = selected(add(add(base, 110), post(&mut evaluations)));
        let distinct_mapped: Vec<_> = [add(base, 19), add(base, 20)]
            .into_iter()
            .map(|v| distinct_mapper.select(v))
            .collect();
        let mut nested_selections = 0;
        let mut nested_evaluations = 0;
        let selected: fn(i64) -> Counter = if post(&mut nested_selections) == 0 {
            if post(&mut nested_selections) == 1 {
                make
            } else {
                double_factory
            }
        } else {
            double_factory
        };
        let mut nested = selected(add(add(base, 120), post(&mut nested_evaluations)));
        let nested_first = nested.select(add(base, 21));
        let nested_second = nested.select(add(base, 22));
        let selected: fn(i64) -> Counter = if post(&mut nested_selections) == 0 {
            make
        } else if post(&mut nested_selections) == 3 {
            double_factory
        } else {
            make
        };
        let mut nested_mapper = selected(add(add(base, 130), post(&mut nested_evaluations)));
        let nested_mapped: Vec<_> = [add(base, 23), add(base, 24)]
            .into_iter()
            .map(|v| nested_mapper.select(v))
            .collect();
        let mut inferred = inferred_factory(add(base, 140));
        let inferred_first = inferred.select(add(base, 25), add(base, 26));
        let inferred_second = inferred.select(add(base, 27), add(base, 28));
        let mut inferred_mapper = inferred_factory(add(base, 150));
        let inferred_mapped: Vec<_> = [add(base, 29), add(base, 30)]
            .into_iter()
            .enumerate()
            .map(|(index, v)| inferred_mapper.select(v, i64::try_from(index).unwrap()))
            .collect();
        let mut fresh_holder = Holder {
            run: make(add(base, 160)),
        };
        let fresh_record = fresh_holder.run.select(add(base, 31));
        let mut fresh_actions = vec![make(add(base, 170))];
        let fresh_stored = fresh_actions[0].select("fresh".to_owned());
        let mut callback = make(add(base, 180));
        let callback_first = callback.invoke(increment, add(base, 32));
        let callback_second = callback.invoke(increment, add(base, 33));
        let mut nullable = Some(make(add(base, 190)));
        let nullable_record = nullable.as_mut().map(|f| f.select(add(base, 35))).unwrap();
        let mut absent: Option<Counter> = None;
        let mut skipped = 0;
        let absent_value = absent
            .as_mut()
            .map(|f| f.select(add(base, post(&mut skipped))));
        assert_eq!(skipped, 0);
        let mut scalar_view = Selector {
            calls: 0,
            array: false,
        };
        let mut array_view = Selector {
            calls: 0,
            array: true,
        };
        let scalar_result = match scalar_view.select(add(base, 37)) {
            ScalarResult::Scalar(v) => v,
            _ => panic!("unexpected array"),
        };
        let array_result = match array_view.select(add(base, 38)) {
            ScalarResult::Array(v) => v,
            _ => panic!("unexpected scalar"),
        };
        let mut payload = payload_factory(add(base, 200));
        let action = payload.select(|v: i64| add(v, 1));
        let payload_int = action(add(base, 39));
        let action = payload.select(|v: std::string::String| v + "!");
        let payload_string = length(&action(base.to_string()));
        let payload_after = payload.select(true);
        let mut explicit = make(add(base, 210));
        let explicit_int = explicit.select::<i64>(add(base, 41));
        let explicit_text = explicit.select::<std::string::String>("typed".to_owned());
        let explicit_after = explicit.select(true);
        let mut optional_generic = Some(make(add(base, 230)));
        let optional_result = optional_generic
            .as_mut()
            .map(|f| f.select::<i64>(add(base, 42)));
        let mut missing_generic: Option<Counter> = None;
        let mut optional_effects = 0;
        let optional_skipped = missing_generic
            .as_mut()
            .map(|f| f.select::<i64>(post(&mut optional_effects)));
        let optional_values: Option<Vec<i64>> = Some(Vec::new());
        let mut patterns = make(add(base, 260));
        let receipt = patterns.head(vec![add(base, 43), add(base, 99)]);
        let pattern_value = receipt.value;
        let pattern_calls = receipt.calls;
        let pattern_text = patterns.head(vec!["owner".to_owned(), "extra".to_owned()]);
        let mut record_patterns = make(add(base, 270));
        let pattern_record = record_patterns.record(PatternBox {
            value: add(base, 44),
        });
        let mut default_payload = payload_factory(add(base, 280));
        let sum = default_payload.select(sum_payload as fn(Option<i64>, Vec<i64>) -> i64);
        let sum_default = sum(None, vec![]);
        let sum_undefined = sum(None, vec![1, 2]);
        let sum_spread = sum(Some(7), vec![3, 4]);
        default_payload.select(sum_payload as fn(Option<i64>, Vec<i64>) -> i64);
        let default_payload_after = default_payload.select(true);
        let mut assignment = make(add(base, 300));
        let assignment_first = assignment.select::<i64>(add(base, 45));
        assignment = make(add(base, 310));
        let assignment_last = assignment.select::<std::string::String>("owner".to_owned());
        let lexical_number = lexical_echo(add(base, 46));
        let lexical_text = lexical_echo("lexical".to_owned());
        let captured_number = lexical_capture(add(base, 47));
        let captured_text = lexical_capture("captured".to_owned());
        let retained_first = retain_polymorphic(First)();
        let retained_second = retain_polymorphic(Second)();
        let common_carrier: fn(i64) -> i64 = if base % 2 == 0 {
            |v| add(v, 1)
        } else {
            |v| add(v, 2)
        };
        let common_result = common_carrier(add(base, 50));
        let captured_offset = add(base, 2);
        let captured_carrier: Box<dyn Fn(i64) -> i64> = if base % 2 == 0 {
            Box::new(move |v| add(v, captured_offset))
        } else {
            Box::new(move |v| mul(v, captured_offset))
        };
        let captured_result = captured_carrier(add(base, 50));
        let borrowed_values = vec![base, add(base, 1)];
        let read_left = || borrowed_values[0];
        let read_right = || borrowed_values[1];
        let borrowed_reader: &dyn Fn() -> i64 = if base % 2 == 0 {
            &read_left
        } else {
            &read_right
        };
        let borrowed_result = borrowed_reader();
        let mut scoped_left = base;
        let mut scoped_right = add(base, 10);
        let borrowed_calls = {
            let mut first = || {
                scoped_left = add(scoped_left, 1);
                scoped_left
            };
            let mut second = || {
                scoped_right = add(scoped_right, 2);
                scoped_right
            };
            let selected: &mut dyn FnMut() -> i64 = if base % 2 == 0 {
                &mut first
            } else {
                &mut second
            };
            add(selected(), selected())
        };
        let borrowed_mutation = add(add(borrowed_calls, scoped_left), scoped_right);
        let consumed_left = Receipt {
            value: add(base, 60),
            calls: 0,
        };
        let consumed_right = Receipt {
            value: add(base, 70),
            calls: 0,
        };
        let source = &borrowed_values;
        let consumed_carrier: Box<dyn FnOnce() -> Receipt<i64> + '_> = if base % 2 == 0 {
            Box::new(move || {
                let mut value = consumed_left;
                value.calls = i64::try_from(source.len()).unwrap();
                value
            })
        } else {
            Box::new(move || {
                let mut value = consumed_right;
                value.calls = i64::try_from(source.len()).unwrap();
                value
            })
        };
        let consumed_record = consume_scoped(consumed_carrier);
        let consumed_result = add(consumed_record.value, consumed_record.calls);
        let retained_payload = Receipt {
            value: add(base, 80),
            calls: 3,
        };
        let retained_record = retain_once(move || retained_payload)();
        let retained_once_result = add(retained_record.value, retained_record.calls);
        let map_left = |v| add(v, borrowed_values[0]);
        let map_right = |v| add(v, borrowed_values[1]);
        let traversal_mapper: &dyn Fn(i64) -> i64 =
            if base % 2 == 0 { &map_left } else { &map_right };
        let traversal_mapped: Vec<_> = borrowed_values
            .iter()
            .map(|v| traversal_mapper(*v))
            .collect();
        let scoped_optional = if base % 2 == 0 {
            Some(borrowed_reader)
        } else {
            None
        };
        let scoped_optional_result = scoped_optional.map_or(0, |f| f());
        let mut traversal_total = 0;
        {
            let mut observer: Box<dyn FnMut(i64) + '_> = if base % 2 == 0 {
                Box::new(|v| traversal_total = add(traversal_total, v))
            } else {
                Box::new(|v| traversal_total = add(traversal_total, mul(v, 2)))
            };
            for value in &borrowed_values {
                observer(*value);
            }
        }
        output.push(NumericArray(vec![
            first_value,
            first_calls,
            length(&generic.value),
            generic.calls,
            length(&last.value),
            last.calls,
            mapped[0].value,
            mapped[1].value,
            mapped[0].calls,
            mapped[1].calls,
            i64::from(after.value),
            after.calls,
            record.value,
            record.calls,
            length(&stored.value),
            stored.calls,
            length(value2),
            reduced,
            length(&reduced_after),
            reversed,
            length(&reversed_after),
            found,
            i64::from(searched_after),
            sortable[0],
            copied[0],
            sorted_after,
            default_value,
            omitted_calls,
            override_calls,
            undefined_calls,
            defaults_after.calls,
            length(&defaults_after.value),
            empty_value,
            empty_calls,
            packed_calls,
            rest_after.calls,
            length(&rest_after.value),
            direct_first.value,
            direct_first.calls,
            direct_second.value,
            direct_second.calls,
            factory_mapped[0].value,
            factory_mapped[1].value,
            factory_mapped[0].calls,
            factory_mapped[1].calls,
            alias_value.value,
            alias_value.calls,
            selected_mapped[0].value,
            selected_mapped[1].value,
            selected_mapped[0].calls,
            selected_mapped[1].calls,
            generic_first.value,
            generic_first.calls,
            generic_second.calls,
            generic_mapped[0].value,
            generic_mapped[1].value,
            generic_mapped[0].calls,
            generic_mapped[1].calls,
            distinct_first.value,
            distinct_first.calls,
            distinct_second.calls,
            distinct_mapped[0].value,
            distinct_mapped[1].value,
            distinct_mapped[0].calls,
            distinct_mapped[1].calls,
            selections,
            evaluations,
            nested_first.value,
            nested_first.calls,
            nested_second.calls,
            nested_mapped[0].value,
            nested_mapped[1].value,
            nested_mapped[0].calls,
            nested_mapped[1].calls,
            nested_selections,
            nested_evaluations,
            inferred_first,
            inferred_second,
            inferred_mapped[0],
            inferred_mapped[1],
            fresh_record.value,
            fresh_record.calls,
            length(&fresh_stored.value),
            fresh_stored.calls,
            callback_first.value,
            callback_first.calls,
            callback_second.value,
            callback_second.calls,
            nullable_record.value,
            nullable_record.calls,
            i64::from(absent_value.is_none()),
            scalar_result,
            array_result[0],
            payload_int,
            payload_string,
            i64::from(payload_after),
            explicit_int.value,
            explicit_int.calls,
            length(&explicit_text.value),
            explicit_text.calls,
            explicit_after.calls,
            optional_result.as_ref().map_or(-1, |r| r.value),
            optional_result.as_ref().map_or(-1, |r| r.calls),
            i64::from(optional_skipped.is_none()),
            optional_effects,
            optional_values
                .as_ref()
                .map_or(-1, |v| i64::try_from(v.len()).unwrap()),
            pattern_value,
            pattern_calls,
            length(&pattern_text.value),
            pattern_text.calls,
            pattern_record.value,
            pattern_record.calls,
            sum_default,
            sum_undefined,
            sum_spread,
            i64::from(default_payload_after),
            assignment_first.value,
            assignment_first.calls,
            length(&assignment_last.value),
            assignment_last.calls,
            lexical_number,
            length(&lexical_text),
            captured_number(),
            length(&captured_text()),
            retained_first.call::<i64>(add(base, 48)),
            length(&retained_first.call("poly".to_owned())),
            retained_second.call::<i64>(0, add(base, 49)),
            common_result,
            captured_result,
            borrowed_result,
            borrowed_mutation,
            consumed_result,
            retained_once_result,
            traversal_mapped[1],
            scoped_optional_result,
            traversal_total,
        ]));
    }
    Array(output)
}
fn main() {}
