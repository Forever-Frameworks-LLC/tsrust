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
    let writer = record(if binding.name=="borrowJsonlWriter"{NativeTypeId::JsonlWriterView}else{NativeTypeId::JsonlWriter});
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
    if binding.name=="borrowJsonlWriter" {
        let owner=record(NativeTypeId::BufferedWriter);
        let result_types=members.iter().map(|ty|if *ty==Type::Record(ok){format!("Record{ok}<Record{writer}<'a>>")}else{format!("Record{err}")}).collect::<Vec<_>>().join(",");
        return format!("fn tsrust_std_json_borrow_jsonl_writer<'a>(writer:&'a mut Record{owner},max_bytes:Option<i64>)->Union{result}<{result_types}>{{{helper}match max_bytes.map(usize::try_from).transpose().map(|limit|limit.unwrap_or(usize::MAX)){{Ok(limit)=>Union{result}::V{ok_tag}(Record{ok}{{field0:Record{writer}{{handle:tsrust_native_json::writer::Writer::new(&mut writer.handle,limit)}}}}),Err(_)=>Union{result}::V{err_tag}(Record{err}{{field0:failure(\"invalid-limits\",\"Output byte limit must be nonnegative and fit this platform\",\"create\",1,None)}})}}}}",result=result.index(),ok_tag=tag(ok),err_tag=tag(err));
    }
    let (parameters,call,payload)=match binding.name {
  "createJsonl"=>("path:&String,max_bytes:Option<i64>,capacity:Option<i64>".into(),format!("(||{{
   let capacity=capacity.unwrap_or(8192);
   if capacity<1 || usize::try_from(capacity).is_err(){{return Err(failure(\"capacity-range\",\"Buffer capacity must be positive and fit this platform\",\"create\",1,None));}}
   let max_bytes=max_bytes.map(usize::try_from).transpose().map(|limit|limit.unwrap_or(usize::MAX)).map_err(|_|failure(\"invalid-limits\",\"Output byte limit must be nonnegative and fit this platform\",\"create\",1,None))?;
   let output=tsrust_native_files::buffered_fs::create_buffered_writer(path.as_ref(),capacity).map_err(|error|failure(error.code,\"Unable to create buffered JSON Lines output\",\"create\",1,error.os_code))?;
   Ok(tsrust_native_json::writer::Writer::new(output,max_bytes))
  }})()"),format!("field0:Record{writer}{{handle:value}}")),
  "writeJsonl"=>{let value=record(NativeTypeId::JsonValue);(format!("writer:&mut impl tsrust_native_json::writer::Sink,value:&Record{value}"),"writer.write_jsonl(&value.value).map_err(|error|failure(error.code,error.message,error.operation,error.line,error.os_code))".into(),String::new())},
  "flushJsonl"=>("writer:&mut impl tsrust_native_json::writer::Sink".into(),"writer.flush_jsonl().map_err(|error|failure(error.code,error.message,error.operation,error.line,error.os_code))".into(),String::new()),
  "closeJsonlWriter"=>("writer:impl tsrust_native_json::writer::Sink".into(),"writer.close_jsonl().map_err(|error|failure(error.code,error.message,error.operation,error.line,error.os_code))".into(),String::new()),
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
