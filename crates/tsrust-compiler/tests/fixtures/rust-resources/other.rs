#[tsrust_resource]
pub struct Session {
    value: i64,
}

pub fn create(value: i64) -> Session { Session { value } }
pub fn read(session: &Session) -> i64 { session.value }
