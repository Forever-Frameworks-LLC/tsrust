use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType,NativeBackend},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding)->String {
 let record=|id|program.records.iter().position(|r|r.native_type==Some(id)).unwrap();
 let reader=record(NativeTypeId::JsonlReader);let error=record(NativeTypeId::JsonlError);
 let NativeType::Structured(signature)=binding.result else {unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap() else {unreachable!()};
 let members=&program.unions[result.index()].members;let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let Type::Union(os)=program.records[error].fields[5].ty.unwrap() else {unreachable!()};
 let os_members=&program.unions[os.index()].members;
 let integer=os_members.iter().position(|ty|*ty==Type::Int).unwrap();let absent=os_members.iter().position(|ty|*ty==Type::Undefined).unwrap();
 let helper=format!("let failure=|code:&'static str,message:&'static str,operation:&'static str,line:usize,offset:usize,os_code:Option<i64>|Record{error}{{field0:String::Borrowed(code),field1:String::Borrowed(message),field2:String::Borrowed(operation),field3:line as i64,field4:offset as i64,field5:match os_code{{Some(code)=>Union{os}::V{integer}(code),None=>Union{os}::V{absent}(TsrUndefined)}}}};",os=os.index());
 let (parameters,call,payload)=match binding.name {
  "openJsonl"=>{
   let limits=record(NativeTypeId::JsonLimits);
   (format!("path:&String,limits:Option<&Record{limits}>,capacity:Option<i64>"),
    format!("(||{{
     let capacity=capacity.unwrap_or(8192);
     if !(1..=1048576).contains(&capacity){{return Err(failure(\"capacity-range\",\"Buffer capacity must be between 1 and 1048576 bytes\",\"open\",1,0,None));}}
     let (bytes,depth,entries,nodes)=if let Some(limits)=limits{{(limits.field0,limits.field1,limits.field2,limits.field3)}}else{{(16*1024*1024,64,1_000_000,1_000_000)}};
     if bytes<0 || !(0..=64).contains(&depth) || entries<0 || nodes<0 {{return Err(failure(\"invalid-limits\",\"JSON limits must be nonnegative and depth must not exceed 64\",\"open\",1,0,None));}}
     let convert=|value:i64|usize::try_from(value).map_err(|_|failure(\"invalid-limits\",\"JSON limits exceed this platform's range\",\"open\",1,0,None));
     let limits=tsrust_native_json::Limits{{bytes:convert(bytes)?,depth:convert(depth)?,entries:convert(entries)?,nodes:convert(nodes)?}};
     let file=tsrust_native_files::fs::open(path.as_ref()).map_err(|error|failure(error.code,\"Unable to open JSON Lines input\",\"open\",1,0,error.os_code))?;
     let input=tsrust_native_files::buffered_fs::buffer_reader(file,capacity).map_err(|error|failure(error.code,\"Unable to buffer JSON Lines input\",\"open\",1,0,error.os_code))?;
     tsrust_native_json::lines::Lines::new(input,limits).map_err(|error|failure(error.error.code,error.error.message,\"open\",error.line,error.error.offset,error.os_code.map(i64::from)))
    }})()"),format!("field0:Record{reader}{{handle:value}}"))
  },
  "readJsonl"=>{
   let value=record(NativeTypeId::JsonValue);
   let Type::Union(optional)=program.records[ok].fields[0].ty.unwrap() else{unreachable!()};
   let variants=&program.unions[optional.index()].members;
   let present=variants.iter().position(|ty|*ty==Type::Record(value)).unwrap();let absent=variants.iter().position(|ty|*ty==Type::Undefined).unwrap();
   (format!("reader:&mut Record{reader}"),"reader.handle.next().map_err(|error|failure(error.error.code,error.error.message,\"read\",error.line,error.error.offset,error.os_code.map(i64::from)))".into(),
    format!("field0:match value{{Some(value)=>Union{optional}::V{present}(Record{value}{{value}}),None=>Union{optional}::V{absent}(TsrUndefined)}}",optional=optional.index()))
  },
  "closeJsonl"=>(format!("reader:Record{reader}"),"reader.handle.close_with(|input|input.close()).map_err(|error|failure(error.code,\"Unable to close JSON Lines input\",\"close\",0,0,error.os_code))".into(),String::new()),
  _=>unreachable!(),
 };
 let NativeBackend::Bundled{symbol}=binding.backend;
 format!("fn {symbol}({parameters})->Union{result}{{{helper}match {call}{{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{{payload}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:error}})}}}}\n",result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
}
