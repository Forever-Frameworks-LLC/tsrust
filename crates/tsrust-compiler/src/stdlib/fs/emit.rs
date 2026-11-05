use crate::{analysis::CheckedProgram, syntax::Type, native_bindings::{NativeBackend, NativeBinding, NativeType}, native_signatures::NativeTypeId};

pub(crate) fn bridges(program: &CheckedProgram, bindings: &[&NativeBinding], ordered:&std::collections::HashSet<usize>) -> String {
    let order=|id|crate::records::runtime::declaration_order_suffix(program,ordered,id);
    let record_id=|native|program.records.iter().position(|record|record.native_type==Some(native));
    let file = program.records.iter().position(|record| record.native_type == Some(NativeTypeId::File));
    if file.is_none() && !program.records.iter().any(|record|matches!(record.native_type,Some(NativeTypeId::BufferedReader | NativeTypeId::BufferedWriter | NativeTypeId::JsonlReader | NativeTypeId::JsonlReaderView | NativeTypeId::JsonlWriter | NativeTypeId::JsonlWriterView))) && bindings.is_empty() { return String::new(); }
    let file = file.map(|id|format!("Record{id}")).unwrap_or_default();
    let mut output = crate::native_runtime::files_module();
    if bindings.iter().any(|binding|binding.name=="createTempDir") {output.push_str(&crate::native_runtime::tempfiles_module());}
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
    output.push_str(&format!("fn tsrust_std_fs_error(error: tsrust_native_files::fs::FsFailure)->Record{error} {{ Record{error} {{ field0:String::Borrowed(error.code), field1:String::Borrowed(error.operation), field2:String::Owned(error.message), field3:match error.os_code {{ Some(code)=>Union{}::V{integer}(code),None=>Union{}::V{absent}(TsrUndefined) }}{} }} }}\n", union.index(), union.index(),order(error)));
    if let Some(id)=record_id(NativeTypeId::FsFileTime) {
        output.push_str(&format!("fn tsrust_std_fs_time(value:tsrust_native_files::fs::FileTime)->Record{id} {{ Record{id} {{ field0:value.unix_seconds,field1:value.nanoseconds{} }} }}\n",order(id)));
    }
    if let Some(id)=record_id(NativeTypeId::FsMetadata) {
        let mut fields="field0:value.is_file,field1:value.is_directory,field2:value.is_symlink,field3:value.size,field4:value.read_only".to_owned();
        let time=record_id(NativeTypeId::FsFileTime).expect("metadata time");
        for (index,name) in [(5,"modified"),(6,"accessed"),(7,"created")] {
            let Type::Union(union)=program.records[id].fields[index].ty.unwrap() else {unreachable!()};
            let members=&program.unions[union.index()].members;
            let present=members.iter().position(|ty|*ty==Type::Record(time)).unwrap();
            let absent=members.iter().position(|ty|*ty==Type::Undefined).unwrap();
            fields.push_str(&format!(",field{index}:match value.{name} {{Some(time)=>Union{}::V{present}(tsrust_std_fs_time(time)),None=>Union{}::V{absent}(TsrUndefined)}}",union.index(),union.index()));
        }
        output.push_str(&format!("fn tsrust_std_fs_metadata(value:tsrust_native_files::fs::Metadata)->Record{id} {{ Record{id} {{ {fields}{} }} }}\n",order(id)));
    }
    if let Some(id)=record_id(NativeTypeId::FsDirEntry) {
        output.push_str(&format!("fn tsrust_std_fs_entry(value:tsrust_native_files::fs::DirEntry)->Record{id} {{ Record{id} {{field0:String::Owned(value.name),field1:String::Owned(value.path),field2:value.is_file,field3:value.is_directory,field4:value.is_symlink{} }} }}\n",order(id)));
    }
    for binding in bindings {
        let (parameters, call, value): (String, String, String) = match binding.name {
            "createTempDir" => ("parent:Option<&String>,prefix:Option<&String>".into(),"tsrust_native_tempfiles::create_temp_dir(parent.map(|value|value.as_ref()),prefix.map(|value|value.as_ref()))".into(),"String::Owned(success)".into()),
            "openFile" => {
                let options=record_id(NativeTypeId::FsOpenOptions).expect("file options");
                let mut fields=Vec::new();
                for (index,name) in ["read","write","append","create","create_new","truncate"].iter().enumerate() {
                    let Type::Union(union)=program.records[options].fields[index].ty.unwrap() else {unreachable!()};
                    let present=program.unions[union.index()].members.iter().position(|ty|*ty==Type::Boolean).unwrap();
                    fields.push(format!("{name}:matches!(&options.field{index},Union{}::V{present}(true))",union.index()));
                }
                (format!("path:&String,options:&Record{options}"),format!("tsrust_native_files::fs::open_file(path.as_ref(),&tsrust_native_files::fs::OpenOptions {{{}}})",fields.join(",")),format!("{file} {{handle:success}}"))
            }
            "stat" | "lstat" => ("path:&String".into(),format!("tsrust_native_files::fs::{}(path.as_ref())",binding.name),"tsrust_std_fs_metadata(success)".into()),
            "fileStat" => (format!("file:&{file}"),"tsrust_native_files::fs::file_stat(&file.handle)".into(),"tsrust_std_fs_metadata(success)".into()),
            "readDir" => {
                let entry=record_id(NativeTypeId::FsDirEntry).expect("directory entry");
                ("path:&String,limit:Option<i64>".into(),"tsrust_native_files::fs::read_dir(path.as_ref(),limit)".into(),format!("RecordArray{entry}::from(success.into_iter().map(tsrust_std_fs_entry).collect::<Vec<_>>())"))
            }
            "mkdir" | "mkdirAll" | "removeFile" | "removeDir" | "removeDirAll" => {
                let operation=match binding.name {"mkdir"=>"mkdir","mkdirAll"=>"mkdir_all","removeFile"=>"remove_file","removeDir"=>"remove_dir",_=>"remove_dir_all"};
                ("path:&String".into(),format!("tsrust_native_files::fs::{operation}(path.as_ref())"),"success".into())
            }
            "canonicalize" | "readLink" => {
                let operation=if binding.name=="canonicalize" {"canonicalize"} else {"read_link"};
                ("path:&String".into(),format!("tsrust_native_files::fs::{operation}(path.as_ref())"),"String::Owned(success)".into())
            }
            "rename" | "copyFile" | "hardLink" => {
                let operation=match binding.name {"rename"=>"rename","copyFile"=>"copy_file",_=>"hard_link"};
                ("path:&String,destination:&String".into(),format!("tsrust_native_files::fs::{operation}(path.as_ref(),destination.as_ref())"),"success".into())
            }
            "appendFile" => ("path:&String,bytes:&impl TsrReadableBytes".into(),"tsrust_native_files::fs::append_file(path.as_ref(),bytes.readable_bytes())".into(),"success".into()),
            "readTextFile" => ("path:&String,limit:Option<i64>".into(),"tsrust_native_files::fs::read_text_file(path.as_ref(),limit)".into(),"String::Owned(success)".into()),
            "writeTextFile" => ("path:&String,text:&String".into(),"tsrust_native_files::fs::write_text_file(path.as_ref(),text.as_ref())".into(),"success".into()),
            "truncate" => (format!("file:&mut {file},length:i64"),"tsrust_native_files::fs::truncate(&mut file.handle,length)".into(),"success".into()),
            "syncAll" | "syncData" => {
                let operation=if binding.name=="syncAll" {"sync_all"} else {"sync_data"};
                (format!("file:&{file}"),format!("tsrust_native_files::fs::{operation}(&file.handle)"),"success".into())
            }
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
                ("path: &String, limit: Option<i64>".into(),"tsrust_native_files::fs::read_file(path.as_ref(),limit)".into(),format!("Record{bytes} {{ bytes: success }}"))
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
        let fields=format!("{fields}{}",order(ok));
        let fields=fields.trim_start_matches(',');
        let err_order=order(err);
        let NativeBackend::Bundled { symbol } = binding.backend;
        output.push_str(&format!("fn {symbol}({parameters})->Union{} {{ match {call} {{ Ok(success)=>Union{}::V{}(Record{ok}{{{fields}}}),Err(error)=>Union{}::V{}(Record{err}{{field0:tsrust_std_fs_error(error){err_order}}}) }} }}\n", result.index(),result.index(),tag(ok),result.index(),tag(err)));
    }
    output
}
