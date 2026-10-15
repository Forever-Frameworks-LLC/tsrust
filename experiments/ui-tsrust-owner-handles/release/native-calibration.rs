#[path = "tracked.rs"] mod tracked;
use tracked::{tracked, close, releases, Tracked};
fn moved(value: Tracked) -> Tracked { value }
fn identity(value: &Tracked) -> i64 { value.id }
struct State { resource: Tracked }
fn assert_log(expected: &[i64]) { assert_eq!(releases(), expected); }
fn main() {
    // Each invocation gets fresh audit storage; no reset can hide a live resource.
    let mode = std::env::args().nth(1).unwrap_or_else(|| "calibration".into());
    let first = tracked(); let first_id = first.id;
    let audit_snapshot = releases(); assert!(audit_snapshot.is_empty());
    if mode == "premature" { assert_log(&[first_id]); return; }
    let first = moved(first); assert_log(&[]);
    let state = State { resource: first }; assert_log(&[]);
    let mut values = vec![state.resource]; assert_log(&[]);
    let first = values.remove(0); assert_log(&[]);
    let callback = move || identity(&first);
    assert_eq!(callback(), first_id); assert_eq!(callback(), first_id); assert_log(&[]);
    drop(callback); assert_log(&[first_id]);
    let mut second = Some(tracked()); let second_id = second.as_ref().unwrap().id;
    close(second.take().unwrap()); assert_log(&[first_id, second_id]);
    drop(second.take()); assert_log(&[first_id, second_id]);
    let third_id;
    { let third = tracked(); third_id = third.id; assert_log(&[first_id, second_id]); }
    assert_log(&[first_id, second_id, third_id]);
    let mut replacement = tracked(); let old_id = replacement.id;
    let fresh = tracked(); let fresh_id = fresh.id;
    replacement = fresh; assert_eq!(replacement.id, fresh_id);
    assert_log(&[first_id, second_id, third_id, old_id]);
    drop(replacement); assert_log(&[first_id, second_id, third_id, old_id, fresh_id]);
    // This establishes observer ordering sensitivity, not automatic UI tree order.
    let parent = tracked(); let parent_id = parent.id;
    let child = tracked(); let child_id = child.id;
    drop(child); drop(parent);
    let expected = [first_id, second_id, third_id, old_id, fresh_id, child_id, parent_id];
    match mode.as_str() {
        "calibration" => assert_log(&expected),
        "count" => assert_eq!(releases().len(), expected.len() + 1),
        "id" => { let mut wrong = expected; wrong[0] = -1; assert_log(&wrong); },
        "missing" => assert_log(&expected[..expected.len()-1]),
        "order" => { let mut wrong = expected; wrong.swap(5,6); assert_log(&wrong); },
        _ => panic!("unknown calibration mode"),
    }
    assert!(audit_snapshot.is_empty()); // Snapshot survives without retaining resources.
    let baseline = releases().len();
    for _ in 0..1000 { let value = tracked(); let id = value.id; drop(value); assert_eq!(*releases().last().unwrap(), id); }
    let log = releases(); assert_eq!(log.len(), baseline + 1000);
    let unique: std::collections::BTreeSet<_> = log.iter().collect(); assert_eq!(unique.len(), log.len());
    println!("native observer calibration passed; checked UI resource boundary remains unproved");
}
