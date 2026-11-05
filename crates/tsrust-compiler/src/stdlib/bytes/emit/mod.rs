//! Concrete native ABI bridges reuse checked Result alternatives, including void success.
use crate::{analysis::CheckedProgram, native_bindings::{NativeBinding, NativeBackend, NativeType}, native_signatures::NativeTypeId, syntax::Type};

pub(crate) fn bridges(program: &CheckedProgram, bindings: &[&NativeBinding]) -> String {
    let bytes = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::Bytes)).expect("checked Bytes identity");
    let mut output = String::new();
    if bindings.iter().any(|binding| binding.name != "length") {
        output.push_str(&crate::native_runtime::bytes_module(
            bindings.iter().any(|binding|binding.returned_borrow.is_some()),
            bindings.iter().any(|binding|matches!(binding.name,"fromUtf8"|"toUtf8")),
            bindings.iter().any(|binding|matches!(binding.name,"toHex"|"fromHex"|"toUtf8Lossy"|"fromUtf16"|"toUtf16"|"toBase64"|"fromBase64"|"toBase64Url"|"fromBase64Url")),
            bindings.iter().any(|binding|matches!(binding.name,"toBase64"|"fromBase64"|"toBase64Url"|"fromBase64Url")),
        ));
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
    if bindings.iter().any(|binding| binding.name == "view") {
        output.push_str("trait TsrSharedViewSource<'a> { fn view_bytes(self) -> &'a [u8]; }\n");
        output.push_str(&format!("impl<'a> TsrSharedViewSource<'a> for &'a Record{bytes} {{ fn view_bytes(self) -> &'a [u8] {{ &self.bytes }} }}\n"));
        if let Some(view)=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::ByteView)) {
            output.push_str(&format!("impl<'a> TsrSharedViewSource<'a> for &Record{view}<'a> {{ fn view_bytes(self) -> &'a [u8] {{ self.bytes }} }}\n"));
        }
        if let Some(view)=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::ByteViewMut)) {
            output.push_str(&format!("impl<'a> TsrSharedViewSource<'a> for &'a Record{view}<'_> {{ fn view_bytes(self) -> &'a [u8] {{ self.bytes }} }}\n"));
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
                let storage=if mutable {"bytes.writable_bytes()"} else {"bytes.view_bytes()"};
                output.push_str(&result_bridge(program,binding,
                    &if mutable {format!("bytes: &'a {access}B, start: i64, end: i64")} else {"bytes: B, start: i64, end: i64".into()},
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
            "toHex" | "toUtf8Lossy" | "toBase64" | "toBase64Url" => {
                let call = match binding.name { "toHex" => "tsrust_native_bytes::to_hex(bytes.readable_bytes())", "toUtf8Lossy" => "tsrust_native_bytes::to_utf8_lossy(bytes.readable_bytes())", "toBase64" => "tsrust_native_bytes::to_base64(bytes.readable_bytes(), false)", _ => "tsrust_native_bytes::to_base64(bytes.readable_bytes(), true)" };
                output.push_str(&result_bridge(program, binding, "bytes: &B", call, "String::Owned(success)"));
            }
            "fromHex" | "fromBase64" | "fromBase64Url" => {
                let call = match binding.name { "fromHex" => "tsrust_native_bytes::from_hex(text.as_ref())", "fromBase64" => "tsrust_native_bytes::from_base64(text.as_ref(), false)", _ => "tsrust_native_bytes::from_base64(text.as_ref(), true)" };
                output.push_str(&result_bridge(program, binding, "text: &String", call, &format!("Record{bytes} {{ bytes: success }}")));
            }
            "fromUtf16" | "toUtf16" => {
                let (order_type, order) = byte_order_bridge(program);
                let (parameters, call, success) = if binding.name == "fromUtf16" {
                    (format!("text: &String, byte_order: &{order_type}"), format!("tsrust_native_bytes::from_utf16(text.as_ref(), {order})"), format!("Record{bytes} {{ bytes: success }}"))
                } else {
                    (format!("bytes: &B, byte_order: &{order_type}"), format!("tsrust_native_bytes::to_utf16(bytes.readable_bytes(), {order})"), "String::Owned(success)".into())
                };
                output.push_str(&result_bridge(program, binding, &parameters, &call, &success));
            }
            _ => unreachable!("registered byte bridge"),
        }
    }
    output
}

fn byte_order_bridge(program: &CheckedProgram) -> (String, String) {
    let Type::Union(id) = crate::native_binding_types::resolve(&program.records, &crate::stdlib::bytes::bindings::BYTE_ORDER).expect("checked byte order") else { unreachable!() };
    let arms = program.unions[id.index()].members.iter().enumerate().map(|(tag, member)| {
        let Type::Literal(record, crate::literal_types::LiteralKind::String) = member else { unreachable!("byte order literals") };
        let Some(crate::literal_types::LiteralValue::String(value)) = &program.records[*record].literal_definition else { unreachable!() };
        assert!(matches!(value.as_str(), "little" | "big"));
        format!("Union{}::V{tag}(_) => {value:?}", id.index())
    }).collect::<Vec<_>>().join(", ");
    (format!("Union{}", id.index()), format!("match byte_order {{ {arms} }}"))
}

fn result_bridge(program: &CheckedProgram, binding: &NativeBinding, parameters: &str, call: &str, success: &str) -> String {
    let NativeType::Structured(signature) = binding.result else { unreachable!() };
    let Type::Union(result) = crate::native_binding_types::resolve(&program.records, signature).expect("checked native Result") else { unreachable!() };
    let members = &program.unions[result.index()].members;
    let [ok, err] = crate::result_identity::variants(&program.records, members).expect("Result identities");
    let tag = |id| members.iter().position(|ty| *ty == Type::Record(id)).unwrap();
    let fields = if program.records[ok].fields.is_empty() { String::new() } else { format!("field0: {success}") };
    let NativeBackend::Bundled { symbol } = binding.backend;
    let lifetime=if binding.returned_borrow.is_some() {if parameters.contains("mut B") {"<'a, B: TsrWritableBytes>"} else {"<'a, B: TsrSharedViewSource<'a>>"}} else if parameters.contains("&mut B") {"<B: TsrWritableBytes>"} else if parameters.contains("&B") {"<B: TsrReadableBytes>"} else {""};
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
mod view_tests;
#[cfg(test)]
mod borrowed_view_tests;

#[cfg(test)]
mod stored_view_tests;
