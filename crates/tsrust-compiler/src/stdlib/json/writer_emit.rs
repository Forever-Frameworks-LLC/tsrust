//! Native resource bridges for bounded JSON Lines output.
use crate::{
    analysis::CheckedProgram,
    native_bindings::{NativeBackend, NativeBinding, NativeType},
    native_signatures::NativeTypeId,
    syntax::Type,
};
pub(crate) fn bridge(program: &CheckedProgram, binding: &NativeBinding) -> String {
    let record = |id| {
        program
            .records
            .iter()
            .position(|r| r.native_type == Some(id))
            .unwrap()
    };
    let writer = record(NativeTypeId::JsonlWriter);
    let error = record(NativeTypeId::JsonlError);
    let NativeType::Structured(signature) = binding.result else {
        unreachable!()
    };
    let Type::Union(result) =
        crate::native_binding_types::resolve(&program.records, signature).unwrap()
    else {
        unreachable!()
    };
    let members = &program.unions[result.index()].members;
    let [ok, err] = crate::result_identity::variants(&program.records, members).unwrap();
    let tag = |id| {
        members
            .iter()
            .position(|ty| *ty == Type::Record(id))
            .unwrap()
    };
    let os = program.records[error].fields[5]
        .ty
        .unwrap()
        .union()
        .unwrap();
    let variants = &program.unions[os.index()].members;
    let present = variants.iter().position(|ty| *ty == Type::Int).unwrap();
    let absent = variants
        .iter()
        .position(|ty| *ty == Type::Undefined)
        .unwrap();
    let helper = format!(
        "let failure=|code:&'static str,message:&'static str,operation:&'static str,line:usize,os_code:Option<i64>|Record{error}{{field0:String::Borrowed(code),field1:String::Borrowed(message),field2:String::Borrowed(operation),field3:line as i64,field4:0,field5:match os_code{{Some(code)=>Union{os}::V{present}(code),None=>Union{os}::V{absent}(TsrUndefined)}}}};",
        os = os.index()
    );
    let (parameters,call,payload)=match binding.name {
  "createJsonl"=>("path:&String,max_bytes:Option<i64>,capacity:Option<i64>".into(),format!("(||{{
   let max_bytes=max_bytes.unwrap_or(16777216);let capacity=capacity.unwrap_or(8192);
   if !(1..=1048576).contains(&capacity){{return Err(failure(\"capacity-range\",\"Buffer capacity must be between 1 and 1048576 bytes\",\"create\",1,None));}}
   let max_bytes=usize::try_from(max_bytes).map_err(|_|failure(\"invalid-limits\",\"Output byte limit must be nonnegative and fit this platform\",\"create\",1,None))?;
   let file=tsrust_native_files::fs::create(path.as_ref()).map_err(|error|failure(error.code,\"Unable to create JSON Lines output\",\"create\",1,error.os_code))?;
   let output=tsrust_native_files::buffered_fs::buffer_writer(file,capacity).map_err(|error|failure(error.code,\"Unable to buffer JSON Lines output\",\"create\",1,error.os_code))?;
   Ok(tsrust_native_json::writer::Writer::new(output,max_bytes))
  }})()"),format!("field0:Record{writer}{{handle:value}}")),
  "writeJsonl"=>{let value=record(NativeTypeId::JsonValue);(format!("writer:&mut Record{writer},value:&Record{value}"),"writer.handle.write(&value.value).map_err(|error|failure(error.code,error.message,error.operation,error.line,error.os_code))".into(),String::new())},
  "flushJsonl"=>(format!("writer:&mut Record{writer}"),"writer.handle.flush().map_err(|error|failure(error.code,error.message,error.operation,error.line,error.os_code))".into(),String::new()),
  "closeJsonlWriter"=>(format!("writer:Record{writer}"),"writer.handle.close().map_err(|error|failure(error.code,error.message,error.operation,error.line,error.os_code))".into(),String::new()),
  _=>unreachable!(),
 };
    let NativeBackend::Bundled { symbol } = binding.backend;
    format!(
        "fn {symbol}({parameters})->Union{result}{{{helper}match {call}{{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{{payload}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:error}})}}}}",
        result = result.index(),
        ok_tag = tag(ok),
        err_tag = tag(err)
    )
}
