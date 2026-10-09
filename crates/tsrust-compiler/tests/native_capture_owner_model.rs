//! Executable native representation proof; compiler lowering is tested separately.
use std::marker::PhantomData;
struct Counter(i64);
impl Counter {
    fn call(&mut self) -> i64 {
        self.0 += 1;
        self.0
    }
}
struct First;
struct Second;
struct Tagged<T, O>(T, PhantomData<O>);
enum Family<A, B> {
    First(A),
    Second(B),
}
impl<'a, 'b> From<Tagged<&'a mut Counter, First>>
    for Family<Tagged<&'a mut Counter, First>, Tagged<&'b mut Counter, Second>>
{
    fn from(value: Tagged<&'a mut Counter, First>) -> Self {
        Self::First(value)
    }
}
impl<'a, 'b> From<Tagged<&'b mut Counter, Second>>
    for Family<Tagged<&'a mut Counter, First>, Tagged<&'b mut Counter, Second>>
{
    fn from(value: Tagged<&'b mut Counter, Second>) -> Self {
        Self::Second(value)
    }
}
impl Family<Tagged<&mut Counter, First>, Tagged<&mut Counter, Second>> {
    fn call(&mut self) -> i64 {
        match self {
            Self::First(x) => x.0.call(),
            Self::Second(x) => x.0.call(),
        }
    }
}
#[inline(never)]
fn install<'a, O, H: From<Tagged<&'a mut Counter, O>>>(
    holder: &mut Option<H>,
    action: &'a mut Counter,
    calls: &mut i64,
) {
    *calls += 1;
    *holder = Some(Tagged(action, PhantomData).into());
}
#[test]
fn equal_type_inputs_keep_independent_lifetimes_after_opaque_replacement() {
    let mut first = Counter(3);
    let mut second = Counter(10);
    let mut calls = 0;
    let mut holder: Option<Family<Tagged<&mut Counter, First>, Tagged<&mut Counter, Second>>> =
        None;
    install::<First, _>(&mut holder, &mut first, &mut calls);
    assert_eq!(holder.as_mut().unwrap().call(), 4);
    install::<Second, _>(&mut holder, &mut second, &mut calls);
    holder = match holder {
        Some(Family::Second(value)) => Some(Family::Second(value)),
        _ => unreachable!(),
    };
    assert_eq!(first.call(), 5);
    assert_eq!(holder.as_mut().unwrap().call(), 11);
    assert_eq!(calls, 2);
    drop(holder);
    assert_eq!(second.call(), 12);
}

#[inline(never)]
fn relay<'a, O, H: From<Tagged<&'a mut Counter, O>>>(
    holder: &mut Option<H>,
    action: &'a mut Counter,
    calls: &mut i64,
) {
    install::<O, H>(holder, action, calls);
}

#[test]
fn relay_propagates_owner_markers_without_shared_state() {
    let mut first = Counter(20);
    let mut second = Counter(40);
    let mut calls = 0;
    let mut holder: Option<Family<Tagged<&mut Counter, First>, Tagged<&mut Counter, Second>>> =
        None;
    relay::<First, _>(&mut holder, &mut first, &mut calls);
    assert_eq!(holder.as_mut().unwrap().call(), 21);
    relay::<Second, _>(&mut holder, &mut second, &mut calls);
    holder = match holder {
        Some(Family::Second(value)) => Some(Family::Second(value)),
        _ => unreachable!(),
    };
    assert_eq!(first.call(), 22);
    assert_eq!(holder.as_mut().unwrap().call(), 41);
    assert_eq!(calls, 2);
    let value = match holder.take().unwrap() {
        Family::Second(value) => value,
        _ => unreachable!(),
    };
    assert_eq!(value.0.call(), 42);
    assert_eq!(second.call(), 43);
}
