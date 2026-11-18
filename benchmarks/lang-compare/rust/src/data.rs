//! Record, collection and recursive-structure phases.
use crate::text::next_random;
use std::collections::{HashMap, HashSet};

struct Order {
    id: i64,
    customer: String,
    amount: f64,
    tags: Vec<String>,
}

pub fn orders(count: usize) -> i64 {
    let mut all = Vec::with_capacity(count);
    let mut state = 7;
    for id in 0..count as i64 {
        state = next_random(state);
        let mut tags = Vec::new();
        if state % 3 == 0 {
            tags.push("priority".to_owned());
        }
        if state % 5 == 0 {
            tags.push("gift".to_owned());
        }
        all.push(Order {id, customer: format!("customer-{}", state % 5000), amount: (state % 100_000) as f64 / 100.0, tags});
    }
    let mut large: Vec<&Order> = all.iter().filter(|order| order.amount > 250.0).collect();
    let sum: f64 = large.iter().map(|order| order.amount * (order.tags.len() + 1) as f64).sum();
    large.sort_by(|left, right| right.amount.total_cmp(&left.amount).then(left.id.cmp(&right.id)));
    let mut by_customer: HashMap<&str, f64> = HashMap::new();
    for order in &large {
        *by_customer.entry(&order.customer).or_insert(0.0) += order.amount;
    }
    sum.floor() as i64 + large[0].id + by_customer.len() as i64
}

struct Tree {
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
}

fn build(depth: i64) -> Tree {
    if depth == 0 {Tree {left: None, right: None}} else {Tree {left: Some(Box::new(build(depth - 1))), right: Some(Box::new(build(depth - 1)))}}
}

fn check(tree: &Tree) -> i64 {
    match (&tree.left, &tree.right) {
        (Some(left), Some(right)) => 1 + check(left) + check(right),
        _ => 1,
    }
}

pub fn trees(max_depth: i64) -> i64 {
    let mut total = 0;
    let mut depth = 4;
    while depth <= max_depth {
        let iterations = 1 << (max_depth - depth + 4);
        for _ in 0..iterations {
            total += check(&build(depth));
        }
        depth += 2;
    }
    total
}

pub fn sets(count: usize) -> i64 {
    let mut seen = HashSet::new();
    let mut state = 3;
    let mut repeats = 0i64;
    for _ in 0..count {
        state = next_random(state);
        let key = state % (count as i64 * 2);
        if !seen.insert(key) {
            repeats += 1;
        }
    }
    repeats * 1_000_003 + seen.len() as i64
}
