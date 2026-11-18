//! The list model, shared by both versions of the comparison app.

pub fn make_items(count: usize) -> Vec<String> {
    (0..count).map(|index| format!("Item {index}")).collect()
}

pub fn filter_items<'a>(items: &'a [String], query: &str) -> Vec<&'a str> {
    items.iter().filter(|item| query.is_empty() || item.contains(query)).map(String::as_str).collect()
}
