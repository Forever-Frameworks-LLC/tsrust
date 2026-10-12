//! Native scalar validation preserves ownership and checked numeric conversion.
use crate::{analysis::CheckedProgram, native_bindings::NativeType, native_signatures::NativeTypeId, syntax::Type};
pub(crate) fn safe_parse(program: &CheckedProgram, schema: usize, binding: &crate::native_bindings::NativeBinding) -> String {
    let NativeType::Structured(signature) = binding.result else { unreachable!() };
    let Type::Union(result) = crate::native_binding_types::resolve(&program.records, signature).unwrap() else { unreachable!() };
    let members = &program.unions[result.index()].members;
    let [ok, err] = crate::result_identity::variants(&program.records, members).unwrap();
    let tag = |id| members.iter().position(|ty| *ty == Type::Record(id)).unwrap();
    let record = |id| program.records.iter().position(|record| record.native_type == Some(id)).unwrap();
    let crate::native_bindings::NativeBackend::Bundled { symbol } = binding.backend else { unreachable!() };
    let boolean = matches!(binding.name, "__safeParseBoolean" | "__safeParseBooleanValue");
    let integer = matches!(binding.name, "__safeParseInt" | "__safeParseIntValue");
    let number = matches!(binding.name, "__safeParseNumber" | "__safeParseNumberValue" | "__safeParseNumberInt");
    let owned = !matches!(binding.parameters[1].ty, NativeType::Structured(_));
    let native = if number { "f64" } else if integer { "i64" } else if boolean { "bool" } else { "String" };
    let expected = if number { "number" } else if integer { "integer" } else if boolean { "boolean" } else { "string" };
    let variant = if boolean { "Bool" } else { "String" };
    let payload = if boolean { "text" } else { "String::Owned(text)" };
    if owned && number {
        let error = record(NativeTypeId::SchemaError);
        let issue = record(NativeTypeId::SchemaIssue);
        let path = record(NativeTypeId::SchemaPath);
        let int_input = binding.name == "__safeParseNumberInt";
        let input_type = if int_input { "i64" } else { "f64" };
        let conversion = if int_input { "let value=input as f64;" } else { "let value=input;" };
        // Compare in i128 so saturating casts cannot accept rounded i64::MAX.
        let valid = if int_input { "value as i128 == input as i128" } else { "value.is_finite()" };
        let code = if int_input { "number-precision" } else { "number-nonfinite" };
        let observed = if int_input { "integer" } else { "number" };
        return format!("fn {symbol}(_schema:&Record{schema},input:{input_type})->Union{result} {{
            {conversion}
            if {valid} {{ return Union{result}::V{ok_tag}(Record{ok}{{field0:value}}); }}
            let issue=Record{issue}{{field0:String::Borrowed(\"{code}\"),field1:RecordArray{path}::from([]),field2:false,field3:String::Borrowed(\"number\"),field4:String::Borrowed(\"{observed}\")}};
            Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:RecordArray{issue}::from([issue]),field1:false}}}})
        }}",result=result.index(),ok_tag=tag(ok),err_tag=tag(err));
    }
    if owned {
        return format!("fn {symbol}(_schema:&Record{schema},input:{native})->Union{result} {{ Union{result}::V{ok_tag}(Record{ok}{{field0:input}}) }}\n",result=result.index(),ok_tag=tag(ok));
    }
    let value = record(NativeTypeId::JsonValue);
    let error = record(NativeTypeId::SchemaError);
    let issue = record(NativeTypeId::SchemaIssue);
    let path = record(NativeTypeId::SchemaPath);
    if integer || number {
        let reader = if integer { "read_i64" } else { "read_f64" };
        return format!("fn {symbol}(_schema:&Record{schema},input:Record{value})->Union{result} {{
            match tsrust_native_json::{reader}(&input.value) {{
                Ok(value) => Union{result}::V{ok_tag}(Record{ok}{{field0:value}}),
                Err(failure) => {{
                    let code=if failure.code==\"expected-number\" {{ \"type-mismatch\" }} else {{ failure.code }};
                    let issue=Record{issue}{{field0:String::Borrowed(code),field1:RecordArray{path}::from([]),field2:false,field3:String::Borrowed(\"{expected}\"),field4:String::Borrowed(tsrust_native_json::kind(&input.value))}};
                    Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:RecordArray{issue}::from([issue]),field1:false}}}})
                }}
            }}
        }}", result=result.index(), ok_tag=tag(ok), err_tag=tag(err));
    }
    format!("fn {symbol}(_schema:&Record{schema},input:Record{value})->Union{result} {{
        match input.value {{
            tsrust_native_json::Value::{variant}(text) => Union{result}::V{ok_tag}(Record{ok}{{field0:{payload}}}),
            other => {{
                let observed=tsrust_native_json::kind(&other);
                let issue=Record{issue}{{field0:String::Borrowed(\"type-mismatch\"),field1:RecordArray{path}::from([]),field2:false,field3:String::Borrowed(\"{expected}\"),field4:String::Borrowed(observed)}};
                Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:RecordArray{issue}::from([issue]),field1:false}}}})
            }}
        }}
    }}", result=result.index(), ok_tag=tag(ok), err_tag=tag(err))
}
