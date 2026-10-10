//! Concrete native ABI bridges reuse checked Result alternatives, including void success.
use crate::{analysis::CheckedProgram, native_bindings::{NativeBinding, NativeBackend, NativeType}, native_signatures::NativeTypeId, syntax::Type};

pub(crate) fn bridges(program: &CheckedProgram, bindings: &[&NativeBinding]) -> String {
    let bytes = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::Bytes)).expect("checked Bytes identity");
    let mut output = String::new();
    if bindings.iter().any(|binding| binding.name != "length") {
        output.push_str("mod tsrust_native_bytes {\n");
        output.push_str(include_str!("runtime.rs"));
        output.push_str("}\n");
        output.push_str(&error_bridge(program));
    }
    for binding in bindings {
        match binding.name {
            "length" => output.push_str(&format!("fn tsrust_std_bytes_length(bytes: &Record{bytes}) -> i64 {{ i64::try_from(bytes.bytes.len()).expect(\"checked byte length\") }}\n")),
            "fromArray" => output.push_str(&result_bridge(program, binding, "values: &[i64]", "tsrust_native_bytes::from_array(values)", &format!("Record{bytes} {{ bytes: success }}"))),
            "at" => output.push_str(&result_bridge(program, binding, &format!("bytes: &Record{bytes}, index: i64"), "tsrust_native_bytes::at(&bytes.bytes, index)", "success")),
            "set" => output.push_str(&result_bridge(program, binding, &format!("bytes: &mut Record{bytes}, index: i64, value: i64"), "tsrust_native_bytes::set(&mut bytes.bytes, index, value)", "success")),
            _ => unreachable!("registered byte bridge"),
        }
    }
    output
}

fn result_bridge(program: &CheckedProgram, binding: &NativeBinding, parameters: &str, call: &str, success: &str) -> String {
    let NativeType::Structured(signature) = binding.result else { unreachable!() };
    let Type::Union(result) = crate::native_binding_types::resolve(&program.records, signature).expect("checked native Result") else { unreachable!() };
    let members = &program.unions[result.index()].members;
    let [ok, err] = crate::result_identity::variants(&program.records, members).expect("Result identities");
    let tag = |id| members.iter().position(|ty| *ty == Type::Record(id)).unwrap();
    let fields = if program.records[ok].fields.is_empty() { String::new() } else { format!("field0: {success}") };
    let NativeBackend::Bundled { symbol } = binding.backend;
    format!("fn {symbol}({parameters}) -> Union{} {{ match {call} {{ Ok(success) => Union{}::V{}(Record{ok} {{ {fields} }}), Err(failure) => Union{}::V{}(Record{err} {{ field0: tsrust_std_bytes_error(failure) }}) }} }}\n", result.index(), result.index(), tag(ok), result.index(), tag(err))
}

fn error_bridge(program: &CheckedProgram) -> String {
    let error = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::ByteError)).expect("checked ByteError identity");
    let mut initializers = Vec::new();
    for (index, field) in program.records[error].fields.iter().enumerate() {
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
    format!("fn tsrust_std_bytes_error(failure: tsrust_native_bytes::ByteFailure) -> Record{error} {{ Record{error} {{ {} }} }}\n", initializers.join(", "))
}
