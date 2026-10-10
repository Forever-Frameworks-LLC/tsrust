//! Concrete native ABI bridges reuse checked Result alternatives, including void success.
use crate::{analysis::CheckedProgram, native_bindings::{NativeBinding, NativeBackend, NativeType}, native_signatures::NativeTypeId, syntax::Type};

pub(crate) fn bridges(program: &CheckedProgram, bindings: &[&NativeBinding]) -> String {
    let bytes = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::Bytes)).expect("checked Bytes identity");
    let mut output = String::new();
    if bindings.iter().any(|binding| binding.name != "length") {
        output.push_str("mod tsrust_native_bytes {\n");
        output.push_str(include_str!("runtime.rs"));
        if bindings.iter().any(|binding| binding.returned_borrow.is_some()) {
            output.push_str(include_str!("views_runtime.rs"));
        }
        if bindings.iter().any(|binding| matches!(binding.name, "fromUtf8" | "toUtf8")) {
            output.push_str(include_str!("utf8_runtime.rs"));
        }
        output.push_str("}\n");
        output.push_str(&error_bridge(program));
    }
    // Observers and mutators accept every readable or writable byte storage that the
    // program names: the owner and its views share these access traits.
    output.push_str("trait TsrReadableBytes { fn readable_bytes(&self) -> &[u8]; }\ntrait TsrWritableBytes: TsrReadableBytes { fn writable_bytes(&mut self) -> &mut [u8]; }\n");
    output.push_str(&format!("impl TsrReadableBytes for Record{bytes} {{ fn readable_bytes(&self) -> &[u8] {{ &self.bytes }} }}\nimpl TsrWritableBytes for Record{bytes} {{ fn writable_bytes(&mut self) -> &mut [u8] {{ &mut self.bytes }} }}\n"));
    for (id,mutable) in [(NativeTypeId::ByteView,false),(NativeTypeId::ByteViewMut,true)] {
        if let Some(view)=program.records.iter().position(|record|record.native_type==Some(id)) {
            output.push_str(&format!("impl TsrReadableBytes for Record{view}<'_> {{ fn readable_bytes(&self) -> &[u8] {{ self.bytes }} }}\n"));
            if mutable {output.push_str(&format!("impl TsrWritableBytes for Record{view}<'_> {{ fn writable_bytes(&mut self) -> &mut [u8] {{ self.bytes }} }}\n"));}
        }
    }
    for binding in bindings {
        match binding.name {
            "view" | "viewMut" => {
                let contract=binding.returned_borrow.expect("view source contract");
                let view=program.records.iter().position(|record|record.native_type==Some(contract.payload)).expect("checked view identity");
                let mutable=binding.name=="viewMut";
                let access=if mutable {"mut "} else {""};
                let operation=if mutable {"view_mut"} else {"view"};
                let storage=if mutable {"bytes.writable_bytes()"} else {"bytes.readable_bytes()"};
                output.push_str(&result_bridge(program,binding,
                    &format!("bytes: &'a {access}B, start: i64, end: i64"),
                    &format!("tsrust_native_bytes::{operation}({storage}, start, end)"),
                    &format!("Record{view} {{ bytes: success }}")));
            }
            "length" => output.push_str("fn tsrust_std_bytes_length<B: TsrReadableBytes>(bytes: &B) -> i64 { i64::try_from(bytes.readable_bytes().len()).expect(\"checked byte length\") }\n"),
            "fromArray" => output.push_str(&result_bridge(program, binding, "values: &[i64]", "tsrust_native_bytes::from_array(values)", &format!("Record{bytes} {{ bytes: success }}"))),
            "at" => output.push_str(&result_bridge(program, binding, "bytes: &B, index: i64", "tsrust_native_bytes::at(bytes.readable_bytes(), index)", "success")),
            "set" => output.push_str(&result_bridge(program, binding, "bytes: &mut B, index: i64, value: i64", "tsrust_native_bytes::set(bytes.writable_bytes(), index, value)", "success")),
            "alloc" => output.push_str(&result_bridge(program, binding, "length: i64, fill: Option<i64>", "tsrust_native_bytes::alloc(length, fill)", &format!("Record{bytes} {{ bytes: success }}"))),
            "resize" => output.push_str(&result_bridge(program, binding, &format!("bytes: &mut Record{bytes}, length: i64, fill: Option<i64>"), "tsrust_native_bytes::resize(&mut bytes.bytes, length, fill)", "success")),
            "append" => output.push_str(&result_bridge(program, binding, &format!("bytes: &mut Record{bytes}, values: &[i64]"), "tsrust_native_bytes::append(&mut bytes.bytes, values)", "success")),
            "copy" => output.push_str(&result_bridge(program, binding, "bytes: &B, start: i64, end: i64", "tsrust_native_bytes::copy(bytes.readable_bytes(), start, end)", &format!("Record{bytes} {{ bytes: success }}"))),
            "fromUtf8" => output.push_str(&result_bridge(program, binding, "text: &String", "tsrust_native_bytes::from_utf8(text.as_ref())", &format!("Record{bytes} {{ bytes: success }}"))),
            "toUtf8" => output.push_str(&result_bridge(program, binding, "bytes: &B", "tsrust_native_bytes::to_utf8(bytes.readable_bytes())", "String::Owned(success)")),
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
    let lifetime=if binding.returned_borrow.is_some() {if parameters.contains("mut B") {"<'a, B: TsrWritableBytes>"} else {"<'a, B: TsrReadableBytes>"}} else if parameters.contains("&mut B") {"<B: TsrWritableBytes>"} else if parameters.contains("&B") {"<B: TsrReadableBytes>"} else {""};
    let result_type=if let Some(contract)=binding.returned_borrow {
        let view=program.records.iter().position(|record|record.native_type==Some(contract.payload)).expect("checked view payload");
        let arguments=members.iter().map(|member|if *member==Type::Record(ok) {format!("Record{ok}<Record{view}<'a>>")} else {format!("Record{err}")}).collect::<Vec<_>>();
        format!("Union{}<{}>",result.index(),arguments.join(", "))
    } else {format!("Union{}",result.index())};
    format!("fn {symbol}{lifetime}({parameters}) -> {result_type} {{ match {call} {{ Ok(success) => Union{}::V{}(Record{ok} {{ {fields} }}), Err(failure) => Union{}::V{}(Record{err} {{ field0: tsrust_std_bytes_error(failure) }}) }} }}\n", result.index(), tag(ok), result.index(), tag(err))
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

#[cfg(test)]
#[path = "views_emit_tests.rs"]
mod view_tests;
#[cfg(test)]
#[path = "views_borrowed_emit_tests.rs"]
mod borrowed_view_tests;

#[cfg(test)]
#[path = "views_stored_emit_tests.rs"]
mod stored_view_tests;
