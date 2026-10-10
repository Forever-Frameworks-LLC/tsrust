//! Concrete native ABI bridges use the same checked Result alternatives as source calls.
use crate::{analysis::CheckedProgram, native_signatures::NativeTypeId, syntax::Type};

pub(crate) fn bridges(program: &CheckedProgram, construction: bool, length: bool) -> String {
    let bytes = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::Bytes)).expect("checked Bytes identity");
    let mut output = String::new();
    if length { output.push_str(&format!("fn tsrust_std_bytes_length(bytes: &Record{bytes}) -> i64 {{ i64::try_from(bytes.bytes.len()).expect(\"checked byte length\") }}\n")); }
    if !construction { return output; }
    output.push_str("mod tsrust_native_bytes {\n");
    output.push_str(include_str!("runtime.rs"));
    output.push_str("}\n");
    let crate::native_bindings::NativeType::Structured(signature) = super::bindings::FROM_ARRAY.result else { unreachable!() };
    let Type::Union(result) = crate::native_binding_types::resolve(&program.records, signature).expect("checked native Result") else { unreachable!() };
    let members = &program.unions[result.index()].members;
    let [ok, err] = crate::result_identity::variants(&program.records, members).expect("Result identities");
    let tag = |id| members.iter().position(|ty| *ty == Type::Record(id)).unwrap();
    let error = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::ByteError)).expect("checked ByteError identity");
    let fields = &program.records[error].fields;
    let mut initializers = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let value = match field.name.text.as_str() {
            "code" => "String::Borrowed(failure.code)".into(),
            "message" => "String::Borrowed(failure.message)".into(),
            "offset" => {
                let Type::Union(union) = field.ty.unwrap() else { unreachable!() };
                let members = &program.unions[union.index()].members;
                let integer = members.iter().position(|ty| *ty == Type::Int).unwrap();
                let absent = members.iter().position(|ty| *ty == Type::Undefined).unwrap();
                format!("match failure.offset {{ Some(offset) => Union{}::V{integer}(offset), None => Union{}::V{absent}(TsrUndefined) }}", union.index(), union.index())
            }
            _ => unreachable!("public error schema"),
        };
        initializers.push(format!("field{index}: {value}"));
    }
    output.push_str(&format!("fn tsrust_std_bytes_from_array(values: &[i64]) -> Union{} {{ match tsrust_native_bytes::from_array(values) {{ Ok(bytes) => Union{}::V{}(Record{ok} {{ field0: Record{bytes} {{ bytes }} }}), Err(failure) => Union{}::V{}(Record{err} {{ field0: Record{error} {{ {} }} }}) }} }}\n", result.index(), result.index(), tag(ok), result.index(), tag(err), initializers.join(", ")));
    output
}
