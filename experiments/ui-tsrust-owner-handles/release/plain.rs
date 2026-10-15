pub struct Plain { pub id: i64 }
pub fn plain() -> Plain { Plain { id: 7 } }
pub fn consume(value: Plain) -> i64 { value.id }
