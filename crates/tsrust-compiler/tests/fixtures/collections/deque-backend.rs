fn main() {
    let mut deque = TsrDeque::new();
    let mut model = Vec::new();
    for value in 0..257_i64 { deque.push_back(value); model.push(value); }
    for step in 0..8192_i64 {
        match (step * 37 + 11) % 7 {
            0 | 1 => { deque.push_back(step); model.push(step); }
            2 => { deque.push_front(step); model.insert(0, step); }
            3 | 4 => {
                let expected = if model.is_empty() { None } else { Some(model.remove(0)) };
                assert_eq!(deque.pop_front(), expected);
            }
            5 => { assert_eq!(deque.pop_back(), model.pop()); }
            _ if step % 29 == 0 => { deque.clear(); model.clear(); }
            _ => {}
        }
        assert_eq!(deque.len(), model.len());
        assert_eq!(deque.front(), model.first());
        assert_eq!(deque.back(), model.last());
        assert_eq!(deque.at(-1), None);
        assert_eq!(deque.at(i64::MAX), None);
        assert_eq!(deque.at(model.len() as i64), None);
        for (index, value) in model.iter().enumerate() { assert_eq!(deque.at(index as i64), Some(value)); }
        assert_eq!(deque.iter().copied().collect::<Vec<_>>(), model);
        assert_eq!(deque.iter().rev().copied().collect::<Vec<_>>(), model.iter().rev().copied().collect::<Vec<_>>());
    }
    deque.clear();
    assert_eq!(deque.pop_front(), None);
    assert_eq!(deque.pop_back(), None);
    assert_eq!(deque.front(), None);
    assert_eq!(deque.back(), None);

    let mut owned = TsrDeque::new();
    owned.push_back(vec![1, 2]);
    let mut copied = owned.clone();
    let mut extracted = copied.pop_front().unwrap();
    extracted[0] = 9;
    assert_eq!(owned.front().unwrap(), &[1, 2]);
    assert_eq!(extracted, [9, 2]);

    struct Probe<'a>(&'a std::cell::Cell<usize>);
    impl Drop for Probe<'_> { fn drop(&mut self) { self.0.set(self.0.get() + 1); } }
    let drops = std::cell::Cell::new(0);
    {
        let mut values = TsrDeque::new();
        values.push_front(Probe(&drops));
        values.push_back(Probe(&drops));
        values.push_front(Probe(&drops));
        let moved = values.pop_back().unwrap();
        assert_eq!(drops.get(), 0);
        values.clear();
        assert_eq!(drops.get(), 2);
        drop(moved);
        assert_eq!(drops.get(), 3);
        values.push_back(Probe(&drops));
    }
    assert_eq!(drops.get(), 4);

    struct Window<'a>(&'a [u8]);
    let owner = [3_u8, 5, 7];
    let mut views = TsrDeque::new();
    views.push_back(Window(&owner[1..]));
    assert_eq!(views.front().unwrap().0.as_ptr(), owner[1..].as_ptr());
    let moved = views.pop_front().unwrap();
    drop(views);
    assert_eq!(moved.0.as_ptr(), owner[1..].as_ptr());
    assert_eq!(moved.0, [5, 7]);
}
