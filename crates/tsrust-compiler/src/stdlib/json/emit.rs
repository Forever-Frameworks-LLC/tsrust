use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding)->String {
 if matches!(binding.name,"get"|"at"|"asString"){return super::views_emit::bridge(program,binding);}
 if matches!(binding.name,"openJsonl"|"readJsonl"|"closeJsonl"){return super::lines_emit::bridge(program,binding);}
 let record=|id|program.records.iter().position(|r|r.native_type==Some(id)).unwrap();
 let value=record(NativeTypeId::JsonValue);
 if matches!(binding.name,"nullValue"|"fromBoolean"|"fromInt"|"fromString"){
  let (parameters,payload)=match binding.name {
   "nullValue"=>("","tsrust_native_json::Value::Null"),
   "fromBoolean"=>("value:bool","tsrust_native_json::Value::Bool(value)"),
   "fromInt"=>("value:i64","tsrust_native_json::Value::Number(value.into())"),
   "fromString"=>("value:String","tsrust_native_json::Value::String(value.into_owned())"),
   _=>unreachable!(),
  };
  return format!("fn tsrust_std_json_{}({parameters})->Record{value}{{Record{value}{{value:{payload}}}}}\n",binding.name);
 }
 if binding.name=="kind" {return format!("fn tsrust_std_json_kind<B:tsrust_native_json::Source>(value:&B)->String {{String::Borrowed(tsrust_native_json::kind(value.json_value()))}}\n");}
 let NativeType::Structured(signature)=binding.result else {unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap() else {unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let error=record(NativeTypeId::JsonError);
 let (parameters,call,payload)=if matches!(binding.name,"parse"|"parseJsonc") {
  let limits=record(NativeTypeId::JsonLimits);
  let (parser,option)=if binding.name=="parseJsonc" {("parse_jsonc",",allow_trailing_comma.unwrap_or(false)")}else{("parse","")};
  (format!("text:&String,{}limits:Option<&Record{limits}>",if binding.name=="parseJsonc" {"allow_trailing_comma:Option<bool>,"} else {""}),
   format!("(|| {{let (bytes,depth,entries,nodes)=if let Some(limits)=limits {{(limits.field0,limits.field1,limits.field2,limits.field3)}} else {{(16*1024*1024,64,1_000_000,1_000_000)}}; let convert=|value:i64|usize::try_from(value).map_err(|_|tsrust_native_json::invalid_limits());tsrust_native_json::{parser}(text.as_ref().as_bytes(),tsrust_native_json::Limits{{bytes:convert(bytes)?,depth:convert(depth)?,entries:convert(entries)?,nodes:convert(nodes)?}}{option})}})()"),
   format!("Record{value}{{value}}"))
 }else if matches!(binding.name,"fromArray"|"fromEntries") {
  let limits=record(NativeTypeId::JsonBuildLimits);
  let element=if binding.name=="fromArray"{value}else{record(NativeTypeId::JsonEntry)};
  let (builder,convert)=if binding.name=="fromArray"{("build_array","item.value")}else{("build_object","(item.field0.into_owned(),item.field1.value)")};
  (format!("values:RecordArray{element},limits:Option<&Record{limits}>"),format!("(||{{let (depth,entries,nodes)=if let Some(limits)=limits{{(limits.field0,limits.field1,limits.field2)}}else{{(64,1_000_000,1_000_000)}};let convert=|value:i64|usize::try_from(value).map_err(|_|tsrust_native_json::invalid_limits());let limits=tsrust_native_json::BuildLimits{{depth:convert(depth)?,entries:convert(entries)?,nodes:convert(nodes)?}};tsrust_native_json::validate_build_root(limits,values.len())?;tsrust_native_json::{builder}(values.tsr_owned_items().map(|item|{convert}).collect(),limits)}})()"),format!("Record{value}{{value}}"))
 }else if binding.name=="fromNumber" {
  ("value:f64".into(),"tsrust_native_json::from_f64(value)".into(),format!("Record{value}{{value}}"))
 }else if matches!(binding.name,"asBoolean"|"asInt"|"asNumber"|"takeString") {
  let (reader,borrow,payload)=match binding.name {
   "asBoolean"=>("read_bool","&","value"),
   "asInt"=>("read_i64","&","value"),
   "asNumber"=>("read_f64","&","value"),
   "takeString"=>("take_string","","String::Owned(value)"),
   _=>unreachable!(),
  };
  (if borrow.is_empty(){format!("value:Record{value}")}else{"value:&B".into()},if borrow.is_empty(){format!("tsrust_native_json::{reader}(value.value)")}else{format!("tsrust_native_json::{reader}(value.json_value())")},payload.to_string())
 }else{
  ("value:&B,max_bytes:Option<i64>".to_string(),"usize::try_from(max_bytes.unwrap_or(16*1024*1024)).map_err(|_|tsrust_native_json::invalid_limits()).and_then(|limit|tsrust_native_json::stringify(value.json_value(),limit))".to_string(),"String::Owned(value)".to_string())
 };
 let generic=if matches!(binding.name,"asBoolean"|"asInt"|"asNumber"|"stringify"){"<B:tsrust_native_json::Source>"}else{""};
 format!("fn tsrust_std_json_{name}{generic}({parameters})->Union{result} {{match {call} {{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:{payload}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64}}}})}}}}\n",name=if binding.name=="parseJsonc" {"parse_jsonc"}else{binding.name},result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
}
