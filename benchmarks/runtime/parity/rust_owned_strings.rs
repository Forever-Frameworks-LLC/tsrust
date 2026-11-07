mod backend {
pub struct Row { pub label: String, pub value: i64 }
pub fn scalar(value: i64) -> i64 { value * 3 + 1 }
pub fn words(value: Vec<String>) -> Vec<String> { value }
pub fn rows(value: Vec<Row>) -> Vec<Row> { value }
pub fn nested(value: i64) -> Option<Option<Vec<String>>> {
    match value % 3 { 0 => None, 1 => Some(None), _ => Some(Some(vec!["alpha".into(), "beta".into()])) }
}
}
fn function0(value0: i64, value1: i64) -> i64 {
let mut checksum=0i64;for i in 0..value0{let returned=backend::words(vec!["alpha".into(),"beta".into(),"gamma".into()]);let observed=value1+i+returned.iter().map(|word|word.len() as i64).sum::<i64>();checksum=(checksum*131+observed)%1000000007;}checksum
}
fn main() {}
