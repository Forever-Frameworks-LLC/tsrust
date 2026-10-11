use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};

fn deque_clone(n: i64, seed: i64) -> Vec<Vec<i64>> {
    let mut output = Vec::new();
    for i in 0..n {
        let value = (i + seed) % 256;
        let mut original = VecDeque::new();
        original.push_back(vec![value, 255]);
        let observed = original.front().unwrap()[0];
        let mut copied = original.clone();
        let mut changed = copied.pop_front().unwrap();
        changed.push(value + 1);
        let retained = original.pop_front().unwrap();
        output.push(vec![retained.len() as i64, retained[0], changed.len() as i64,
            changed[2], original.len() as i64, copied.len() as i64, observed]);
    }
    output
}

fn heap(n: i64, seed: i64) -> Vec<Vec<i64>> {
    let mut output = Vec::new();
    for i in 0..n {
        let mut values = BinaryHeap::new();
        for j in 0..9 { values.push((i + seed + j) % 256); }
        let traversed: i64 = values.iter().copied().sum();
        // Materialize the borrowed rest/spread inputs, as the source workload does.
        let spread_items: Vec<i64> = values.iter().copied().collect();
        let spread: i64 = spread_items.iter().copied().sum();
        let flattened: Vec<i64> = values.iter().copied().collect();
        let top = *values.peek().unwrap();
        let mut copied = values.clone();
        let mut sum = 0;
        while !copied.is_empty() { sum += copied.pop().unwrap(); }
        output.push(vec![values.len() as i64, top, sum, copied.len() as i64,
            traversed, spread, flattened.len() as i64]);
    }
    output
}

fn ordered_set(n: i64, seed: i64) -> Vec<Vec<i64>> {
    let mut output = Vec::new();
    for i in 0..n {
        let mut values = BTreeSet::new();
        for j in 0..9 { values.insert((i + seed + j) % 256); }
        values.insert((i + seed) % 256);
        let (mut count, mut weighted, mut first, mut last) = (0, 0, -1, -1);
        for &value in &values {
            if count == 0 { first = value; }
            count += 1; weighted += value * count; last = value;
        }
        let copied = values.clone();
        let key = (i + seed + 4) % 256;
        let found = i64::from(values.contains(&key));
        let removed = i64::from(values.remove(&key));
        output.push(vec![count, first, last, weighted, found, removed,
            values.len() as i64, copied.len() as i64]);
    }
    output
}

fn ordered_map(n: i64, seed: i64) -> Vec<Vec<i64>> {
    let mut output = Vec::new();
    for i in 0..n {
        let mut values = BTreeMap::new();
        for j in 0..9 { let key = (i + seed + j) % 256; values.insert(key, key * 3 + 1); }
        values.insert((i + seed) % 256, 999);
        let (mut count, mut weighted, mut sum, mut first, mut last) = (0, 0, 0, -1, -1);
        for (&key, &value) in &values {
            if count == 0 { first = key; }
            count += 1; weighted += key * count; sum += value; last = key;
        }
        let copied = values.clone();
        let key = (i + seed + 4) % 256;
        let selected = *values.get(&key).unwrap();
        let removed = i64::from(values.remove(&key).is_some());
        output.push(vec![count, first, last, weighted, sum, selected, removed,
            values.len() as i64, copied.len() as i64]);
    }
    output
}
