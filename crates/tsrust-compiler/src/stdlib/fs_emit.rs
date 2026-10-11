use crate::{analysis::CheckedProgram, syntax::Type, native_bindings::{NativeBackend, NativeBinding, NativeType}, native_signatures::NativeTypeId};

pub(crate) fn bridges(program: &CheckedProgram, bindings: &[&NativeBinding]) -> String {
    let Some(file) = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::File)) else { return String::new(); };
    let mut output = format!("mod tsrust_native_files {{ mod resource {{\n{}\n}} pub mod fs {{\n{}\n}} }}\n", include_str!("resource.rs"), include_str!("fs.rs"));
    if bindings.is_empty() { return output; }
    let error = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::FsError)).expect("checked FsError");
    let field = &program.records[error].fields[3];
    let Type::Union(union) = field.ty.unwrap() else { unreachable!() };
    let members = &program.unions[union.index()].members;
    let integer = members.iter().position(|ty| *ty == Type::Int).unwrap();
    let absent = members.iter().position(|ty| *ty == Type::Undefined).unwrap();
    output.push_str(&format!("fn tsrust_std_fs_error(error: tsrust_native_files::fs::FsFailure)->Record{error} {{ Record{error} {{ field0:String::Borrowed(error.code), field1:String::Borrowed(error.operation), field2:String::Owned(error.message), field3:match error.os_code {{ Some(code)=>Union{}::V{integer}(code),None=>Union{}::V{absent}(TsrUndefined) }} }} }}\n", union.index(), union.index()));
    for binding in bindings {
        let (parameters, call, value): (String, String, String) = match binding.name {
            "open" => ("path: &String".into(), "tsrust_native_files::fs::open(path.as_ref())".into(), format!("Record{file} {{ handle: success }}")),
            "create" => ("path: &String".into(), "tsrust_native_files::fs::create(path.as_ref())".into(), format!("Record{file} {{ handle: success }}")),
            "write" | "writeAll" => (format!("file: &mut Record{file}, bytes: &impl TsrReadableBytes"), format!("tsrust_native_files::fs::{}(&mut file.handle, bytes.readable_bytes())", if binding.name == "write" {"write"} else {"write_all"}), "success".into()),
            "read" => {
                let bytes = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::Bytes)).expect("read Bytes");
                (format!("file: &mut Record{file}, length: i64"), "tsrust_native_files::fs::read(&mut file.handle,length)".into(), format!("Record{bytes} {{ bytes: success }}"))
            }
            "close" => (format!("file: Record{file}"), "file.handle.close()".into(), String::new()),
            _ => unreachable!("registered file binding"),
        };
        let NativeType::Structured(signature) = binding.result else { unreachable!() };
        let Type::Union(result) = crate::native_binding_types::resolve(&program.records, signature).expect("checked file result") else { unreachable!() };
        let members = &program.unions[result.index()].members;
        let [ok, err] = crate::result_identity::variants(&program.records, members).expect("file result identities");
        let tag = |id| members.iter().position(|ty| *ty == Type::Record(id)).unwrap();
        let fields = if program.records[ok].fields.is_empty() { String::new() } else { format!("field0: {value}") };
        let NativeBackend::Bundled { symbol } = binding.backend;
        output.push_str(&format!("fn {symbol}({parameters})->Union{} {{ match {call} {{ Ok(success)=>Union{}::V{}(Record{ok}{{{fields}}}),Err(error)=>Union{}::V{}(Record{err}{{field0:tsrust_std_fs_error(error)}}) }} }}\n", result.index(),result.index(),tag(ok),result.index(),tag(err)));
    }
    output
}
