//! Native string validation moves successful input storage into the typed result.
use crate::{analysis::CheckedProgram, native_bindings::NativeType, native_signatures::NativeTypeId, syntax::Type};
pub(crate) fn safe_parse(program: &CheckedProgram, schema: usize, owned: bool) -> String {
    let NativeType::Structured(signature) = super::namespace::SAFE_PARSE.result else { unreachable!() };
    let Type::Union(result) = crate::native_binding_types::resolve(&program.records, signature).unwrap() else { unreachable!() };
    let members = &program.unions[result.index()].members;
    let [ok, err] = crate::result_identity::variants(&program.records, members).unwrap();
    let tag = |id| members.iter().position(|ty| *ty == Type::Record(id)).unwrap();
    let record = |id| program.records.iter().position(|record| record.native_type == Some(id)).unwrap();
    if owned {
        return format!("fn tsrust_schema_safe_parse_owned_string(_schema:&Record{schema},input:String)->Union{result} {{ Union{result}::V{ok_tag}(Record{ok}{{field0:input}}) }}\n",result=result.index(),ok_tag=tag(ok));
    }
    let value = record(NativeTypeId::JsonValue);
    let error = record(NativeTypeId::SchemaError);
    let issue = record(NativeTypeId::SchemaIssue);
    let path = record(NativeTypeId::SchemaPath);
    format!("fn tsrust_schema_safe_parse_string(_schema:&Record{schema},input:Record{value})->Union{result} {{
        match input.value {{
            tsrust_native_json::Value::String(text) => Union{result}::V{ok_tag}(Record{ok}{{field0:String::Owned(text)}}),
            other => {{
                let observed=tsrust_native_json::kind(&other);
                let issue=Record{issue}{{field0:String::Borrowed(\"type-mismatch\"),field1:RecordArray{path}::from([]),field2:false,field3:String::Borrowed(\"string\"),field4:String::Borrowed(observed)}};
                Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:RecordArray{issue}::from([issue]),field1:false}}}})
            }}
        }}
    }}", result=result.index(), ok_tag=tag(ok), err_tag=tag(err))
}
