use crate::{analysis::CheckedProgram, syntax::Type, native_bindings::{NativeBackend, NativeBinding, NativeType}, native_signatures::NativeTypeId};

pub(crate) fn bridges(program: &CheckedProgram, bindings: &[&NativeBinding]) -> String {
    let file = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::File));
    if file.is_none() && !program.records.iter().any(|record|matches!(record.native_type,Some(NativeTypeId::BufferedReader | NativeTypeId::BufferedWriter | NativeTypeId::JsonlReader))) && bindings.is_empty() { return String::new(); }
    let file = file.map(|id|format!("Record{id}")).unwrap_or_default();
    let mut output = crate::native_runtime::files_module();
    output.push_str("trait TsrFileRead { fn read_bytes(&mut self,length:i64)->Result<Vec<u8>,tsrust_native_files::fs::FsFailure>; }\ntrait TsrFileWrite { fn write_bytes(&mut self,bytes:&[u8])->Result<i64,tsrust_native_files::fs::FsFailure>; fn write_all_bytes(&mut self,bytes:&[u8])->Result<(),tsrust_native_files::fs::FsFailure>; }\ntrait TsrFileClose { fn close_file(self)->Result<(),tsrust_native_files::fs::FsFailure>; }\n");
    for (index,record) in program.records.iter().enumerate() {
        let Some(id @ (NativeTypeId::File | NativeTypeId::BufferedReader | NativeTypeId::BufferedWriter))=record.native_type else {continue;};
        output.push_str(&format!("impl TsrFileClose for Record{index} {{ fn close_file(self)->Result<(),tsrust_native_files::fs::FsFailure> {{ self.handle.close() }} }}\n"));
        if matches!(id,NativeTypeId::File | NativeTypeId::BufferedReader) {
            let operation=if id==NativeTypeId::File {"fs::read"} else {"buffered_fs::buffered_read"};
            output.push_str(&format!("impl TsrFileRead for Record{index} {{ fn read_bytes(&mut self,length:i64)->Result<Vec<u8>,tsrust_native_files::fs::FsFailure> {{ tsrust_native_files::{operation}(&mut self.handle,length) }} }}\n"));
        }
        if matches!(id,NativeTypeId::File | NativeTypeId::BufferedWriter) {
            let operation=if id==NativeTypeId::File {"fs::write"} else {"buffered_fs::buffered_write"};
            output.push_str(&format!("impl TsrFileWrite for Record{index} {{ fn write_bytes(&mut self,bytes:&[u8])->Result<i64,tsrust_native_files::fs::FsFailure> {{ tsrust_native_files::{operation}(&mut self.handle,bytes) }} fn write_all_bytes(&mut self,bytes:&[u8])->Result<(),tsrust_native_files::fs::FsFailure> {{ tsrust_native_files::{operation}_all(&mut self.handle,bytes) }} }}\n"));
        }
    }
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
            "bufferReader" | "bufferWriter" => {
                let id=if binding.name=="bufferReader" {NativeTypeId::BufferedReader} else {NativeTypeId::BufferedWriter};
                let record=program.records.iter().position(|record|record.native_type==Some(id)).expect("buffered file type");
                let operation=if binding.name=="bufferReader" {"buffer_reader"} else {"buffer_writer"};
                (format!("file: {file}, capacity: Option<i64>"),format!("tsrust_native_files::buffered_fs::{operation}(file.handle,capacity.unwrap_or(tsrust_native_files::buffered_fs::DEFAULT_CAPACITY))"),format!("Record{record} {{ handle: success }}"))
            }
            "flush" => {
                let record=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::BufferedWriter)).expect("buffered writer");
                (format!("writer: &mut Record{record}"),"tsrust_native_files::buffered_fs::flush(&mut writer.handle)".into(),"success".into())
            }
            "open" => ("path: &String".into(), "tsrust_native_files::fs::open(path.as_ref())".into(), format!("{file} {{ handle: success }}")),
            "readFile" => {
                let bytes=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::Bytes)).expect("readFile Bytes");
                ("path: &String, limit: Option<i64>".into(),"tsrust_native_files::fs::read_file(path.as_ref(),limit.unwrap_or(67108864))".into(),format!("Record{bytes} {{ bytes: success }}"))
            }
            "writeFile" => ("path: &String, bytes: &impl TsrReadableBytes".into(),"tsrust_native_files::fs::write_file(path.as_ref(),bytes.readable_bytes())".into(),"success".into()),
            "create" => ("path: &String".into(), "tsrust_native_files::fs::create(path.as_ref())".into(), format!("{file} {{ handle: success }}")),
            "write" | "writeAll" => ("file: &mut impl TsrFileWrite, bytes: &impl TsrReadableBytes".into(), format!("file.{}(bytes.readable_bytes())", if binding.name == "write" {"write_bytes"} else {"write_all_bytes"}), "success".into()),
            "read" => {
                let bytes = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::Bytes)).expect("read Bytes");
                ("file: &mut impl TsrFileRead, length: i64".into(), "file.read_bytes(length)".into(), format!("Record{bytes} {{ bytes: success }}"))
            }
            "close" => ("file: impl TsrFileClose".into(), "file.close_file()".into(), String::new()),
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
