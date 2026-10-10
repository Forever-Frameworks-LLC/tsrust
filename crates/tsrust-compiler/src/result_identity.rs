//! Recognition of the compiler's private Result alternatives. Source spelling
//! alone never establishes a Result identity.
use crate::syntax::{Record, Type};

pub(crate) fn variants(records: &[Record], members: &[Type]) -> Option<[usize; 2]> {
    if members.len() != 2 { return None; }
    let mut variants = [None, None];
    for member in members {
        let Type::Record(id) = member else { return None; };
        let application = records[*id].application.as_ref()?;
        let index = match application.declaration.text.as_str() {
            "Result#Ok" => 0,
            "Result#Err" => 1,
            _ => return None,
        };
        if variants[index].replace(*id).is_some() { return None; }
    }
    Some([variants[0]?, variants[1]?])
}

pub(crate) fn payload(record: &Record) -> Option<Type> {
    record.fields.first().map(|field| field.ty).unwrap_or(Some(Type::Void))
}
