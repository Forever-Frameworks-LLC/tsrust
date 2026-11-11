pub struct Row { pub label: String, pub value: i64 }
pub fn scalar(value: i64) -> i64 { value * 3 + 1 }
pub fn words(value: Vec<String>) -> Vec<String> { value }
pub fn rows(value: Vec<Row>) -> Vec<Row> { value }
pub fn nested(value: i64) -> Option<Option<Vec<String>>> {
    match value % 3 { 0 => None, 1 => Some(None), _ => Some(Some(vec!["alpha".into(), "beta".into()])) }
}
