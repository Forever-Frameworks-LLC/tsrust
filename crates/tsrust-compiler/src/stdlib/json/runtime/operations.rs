pub fn stringify(value: &Value, max_bytes: usize) -> Result<std::string::String, Error> {
    serialize_bounded(value,max_bytes)
}

/// Shared bounded output for dynamic values and compiler-generated typed visitors.
fn serialize_bounded<T:serde::Serialize + ?Sized>(value:&T,max_bytes:usize)->Result<std::string::String,Error> {
    struct Output { bytes: Vec<u8>, limit: usize, exceeded: bool }
    impl std::io::Write for Output {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.write_all(bytes)?;Ok(bytes.len())
        }
        fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
            if bytes.len() > self.limit - self.bytes.len() { self.exceeded=true;return Err(std::io::Error::other("JSON output limit")); }
            self.bytes.extend_from_slice(bytes);Ok(())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    let mut output=Output{bytes:Vec::with_capacity(max_bytes.min(128)),limit:max_bytes,exceeded:false};
    if let Err(error)=serde_json::to_writer(&mut output,value) {
        let code=if output.exceeded {"output-limit"} else {
            match error.to_string().as_str() {
                "tsrust:number-nonfinite"=>"number-nonfinite",
                "tsrust:depth-limit"=>"depth-limit",
                "tsrust:undefined-value"=>"undefined-value",
                _=>"invalid-json",
            }
        };
        return Err(failure(code,output.bytes.len()));
    }
    Ok(std::string::String::from_utf8(output.bytes).expect("JSON serialization emits UTF-8"))
}
pub fn kind(value: &Value) -> &'static str {
    match value {Value::Null=>"null",Value::Bool(_)=>"boolean",Value::Number(_)=>"number",Value::String(_)=>"string",Value::Array(_)=>"array",Value::Object(_)=>"object"}
}

/// Familiar typeof categories; arrays and null share the object category.
pub fn type_of(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Null | Value::Array(_) | Value::Object(_) => "object",
    }
}

/// Scalar inspection does not coerce null, strings, or booleans into numbers.
/// Values do not retain source positions, so extraction errors use offset zero.
pub fn read_bool(value: &Value) -> Result<bool, Error> {
    value.as_bool().ok_or_else(|| failure("expected-boolean", 0))
}
pub fn read_i64(value: &Value) -> Result<i64, Error> {
    let Value::Number(number) = value else { return Err(failure("expected-number", 0)); };
    if let Some(integer) = number.as_i64() { return Ok(integer); }
    if number.as_u64().is_some() { return Err(failure("integer-range", 0)); }
    let float = number.as_f64().expect("JSON numbers are finite");
    if float < -9223372036854775808.0 || float >= 9223372036854775808.0 {
        return Err(failure("integer-range", 0));
    }
    if float.fract() != 0.0 { return Err(failure("integer-fractional", 0)); }
    // An explicit integer extraction normalizes floating negative zero to zero.
    Ok(float as i64)
}
pub fn read_f64(value: &Value) -> Result<f64, Error> {
    let Value::Number(number) = value else { return Err(failure("expected-number", 0)); };
    let float = number.as_f64().expect("JSON numbers are finite");
    let integer = number.as_i64().map(i128::from).or_else(|| number.as_u64().map(i128::from));
    // Widen the comparison so a saturating i64/u64 cast cannot falsely accept
    // rounded boundary values such as i64::MAX or u64::MAX.
    if integer.is_some_and(|integer| float as i128 != integer) {
        return Err(failure("number-precision", 0));
    }
    Ok(float)
}
pub fn read_string(value: &Value) -> Result<&str, Error> {
    value.as_str().ok_or_else(|| failure("expected-string", 0))
}
pub fn take_string(value: Value) -> Result<String, Error> {
    match value { Value::String(text) => Ok(text), _ => Err(failure("expected-string", 0)) }
}


/// Missing children are None; a present JSON null remains Some(Value::Null).
/// The borrow is tied only to the container, not to the lookup key.
pub fn get_key<'a>(value: &'a Value, key: &str) -> Result<Option<&'a Value>, Error> {
    let Value::Object(object) = value else { return Err(failure("expected-object", 0)); };
    Ok(object.get(key))
}
pub fn get_index(value: &Value, index: i64) -> Result<Option<&Value>, Error> {
    let Value::Array(array) = value else { return Err(failure("expected-array", 0)); };
    if index < 0 { return Err(failure("index-range", 0)); }
    // An index beyond the host address space is absent, not a truncated index.
    Ok(usize::try_from(index).ok().and_then(|index| array.get(index)))
}
/// Consumes the entire container; only the selected child survives. No clone.
pub fn take_key(value: Value, key: &str) -> Result<Option<Value>, Error> {
    let Value::Object(mut object) = value else { return Err(failure("expected-object", 0)); };
    Ok(object.remove(key))
}
pub fn take_index(value: Value, index: i64) -> Result<Option<Value>, Error> {
    let Value::Array(mut array) = value else { return Err(failure("expected-array", 0)); };
    if index < 0 { return Err(failure("index-range", 0)); }
    let Some(index) = usize::try_from(index).ok().filter(|index| *index < array.len()) else { return Ok(None); };
    // Order is unobservable after consuming the parent. Avoid shifting children
    // that are about to be dropped; the selected child's allocation is retained.
    Ok(Some(array.swap_remove(index)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> Limits { Limits { bytes: 1024, depth: 4, entries: 3, nodes: 20 } }
    #[test]
    fn jsonc_comments_trailing_commas_and_offsets() {
        let input=br#"{/* note */"url":"https://host/*literal*/",// line
"values":[1,2,],}"#;
        assert!(parse(input,limits()).is_err());
        assert!(parse_jsonc(input,limits(),false).is_err());
        let value=parse_jsonc(input,limits(),true).unwrap();
        assert_eq!(value["url"],"https://host/*literal*/");assert_eq!(value["values"],serde_json::json!([1,2]));
        assert_eq!(parse_jsonc(b"// comment\rnull",limits(),false).unwrap(),Value::Null);
        let bad="/*é*/\n[0,secret]";
        assert_eq!(parse_jsonc(bad.as_bytes(),limits(),false).unwrap_err().offset,bad.find("secret").unwrap());
        let error=parse_jsonc(b"null /* secret",limits(),false).unwrap_err();
        assert_eq!(error.code,"unterminated-comment");assert_eq!(error.offset,5);assert!(!error.to_string().contains("secret"));
        for bad in ["[,]","{,}","[1,,]","{\"a\":,}","1/*x*/2","/*x*/","[1,] garbage"] {assert!(parse_jsonc(bad.as_bytes(),limits(),true).is_err(),"{bad}");}
        assert_eq!(parse_jsonc(b"/*long*/null",Limits{bytes:4,..limits()},false).unwrap_err().code,"byte-limit");
        assert_eq!(parse_jsonc(&[b'/',b'/',0xff,b'\n',b'0'],limits(),false).unwrap_err().code,"invalid-utf8");
    }
    #[test]
    fn encoding_limits_count_utf8_and_escaped_output_bytes() {
        for (input,expected) in [("\"é\"","\"é\""),("\"\\u0000\"","\"\\u0000\""),("-0.0","-0.0"),("18446744073709551615","18446744073709551615")] {
            let value=parse(input.as_bytes(),limits()).unwrap();
            assert_eq!(stringify(&value,expected.len()).unwrap(),expected);
            let error=stringify(&value,expected.len()-1).unwrap_err();
            assert_eq!(error.code,"output-limit");assert!(error.offset<=expected.len()-1);
            assert_eq!(stringify(&value,0).unwrap_err().code,"output-limit");
        }
    }
    #[test]
    fn errors_have_codes_byte_offsets_and_no_input_contents() {
        for (input,code,offset) in [
            (b"null secret".as_slice(),"trailing-data",5),
            (b"[1,".as_slice(),"unexpected-end",3),
            (b"".as_slice(),"unexpected-end",0),
            (b" \r\n".as_slice(),"unexpected-end",3),
            ("\"é".as_bytes(),"unexpected-end",3),
            (b"[1,\r\n]".as_slice(),"invalid-json",5),
            ("[\"é\",\nsecret]".as_bytes(),"invalid-json",7),
            (&[b'"',0xff,b'"'],"invalid-utf8",1),
        ] {
            let error=parse(input,limits()).unwrap_err();
            assert_eq!(error.code,code);assert_eq!(error.offset,offset);
            assert!(!error.to_string().contains("secret"));assert!(!error.message.contains("secret"));assert!(!format!("{error:?}").contains("secret"));
        }
        assert_eq!(parse(b"null",Limits{bytes:3,..limits()}).unwrap_err().code,"byte-limit");
        assert_eq!(parse(b"[]",Limits{depth:0,..limits()}).unwrap_err().code,"depth-limit");
        assert_eq!(parse(b"[0,1]",Limits{entries:1,..limits()}).unwrap_err().code,"entry-limit");
        assert_eq!(parse(b"[0]",Limits{nodes:1,..limits()}).unwrap_err().code,"node-limit");
        assert_eq!(parse(b"null",Limits{depth:65,..limits()}).unwrap_err().code,"invalid-limits");
    }
    #[test]
    fn exhausted_budgets_stop_before_decoding_the_next_value_or_key() {
        let one = Limits { entries: 1, ..limits() };
        assert!(parse(b"[0,invalid]", one).unwrap_err().message.contains("entry limit"));
        assert!(parse(br#"{"a":0,"unterminated"#, one).unwrap_err().message.contains("entry limit"));
        assert!(parse(b"[invalid]", Limits { nodes: 1, ..limits() }).unwrap_err().message.contains("value count"));
        assert!(parse(b"null", Limits { nodes: 0, ..limits() }).is_err());
        assert!(parse(b"null", Limits { depth: 0, ..limits() }).is_ok());
        assert!(parse(b"{}", Limits { depth: 0, ..limits() }).is_err());
    }
    #[test]
    fn bounded_depth_and_reference_value_semantics_match() {
        let nested = format!("{}0{}", "[".repeat(64), "]".repeat(64));
        assert!(parse(nested.as_bytes(), Limits { depth: 64, nodes: 65, ..limits() }).is_ok());
        assert!(parse(nested.as_bytes(), Limits { depth: 63, nodes: 65, ..limits() }).is_err());
        assert!(parse(b"null", Limits { depth: 65, ..limits() }).is_err());
        for input in ["null", "false", "0", "-0", "-0.0", "1e-300", "[]", "{}", " [true, null] ", r#"{"z":1,"a":2,"z":3}"#, r#""a\n\u0000é""#] {
            assert_eq!(parse(input.as_bytes(), limits()).unwrap(), serde_json::from_str::<Value>(input).unwrap());
        }
    }
    #[test]
    fn limits_reject_before_building_unbounded_values() {
        assert!(parse(b"null", Limits { bytes: 3, ..limits() }).is_err());
        assert!(parse(b"[[[[[]]]]]", limits()).is_err());
        assert!(parse(b"[0,1,2,3]", limits()).is_err());
        assert!(parse(br#"{"a":0,"a":1,"a":2,"a":3}"#, limits()).is_err());
        assert!(parse(b"[0,1]", Limits { nodes: 2, ..limits() }).is_err());
        assert!(parse(b"[]", Limits { entries: 0, nodes: 1, ..limits() }).is_ok());
        assert!(parse(b"[[]]", Limits { depth: 1, ..limits() }).is_err());
        assert!(parse(b"[0]", Limits { depth: 1, ..limits() }).is_ok());
    }
    #[test]
    fn strict_owned_values_preserve_scalar_boundaries() {
        let value = parse(br#"{"a":null,"a":true,"b":[-0.0,9223372036854775807,18446744073709551615,"\uD83E\uDD80"]}"#, Limits { entries: 4, ..limits() }).unwrap();
        assert_eq!(value["a"], true);
        assert!(value["b"][0].as_f64().unwrap().is_sign_negative());
        assert_eq!(value["b"][1].as_i64(), Some(i64::MAX));
        assert_eq!(value["b"][2].as_u64(), Some(u64::MAX));
        assert_eq!(value["b"][3], "🦀");
        for input in [b"null true".as_slice(), b"01", b"[1,]", b"NaN", b"1e999", br#""\uD800""#, &[b'"', 0xff, b'"']] {
            assert!(parse(input, limits()).is_err(), "{input:?}");
        }
    }
}

/// Native adapters expose real shared references, never cloned JSON storage.
pub trait Source { fn json_value(&self) -> &Value; }
pub trait ViewSource<'a> { fn json_view(self) -> &'a Value; }

/// Constructs a finite JSON number without changing its floating-point value.
pub fn from_f64(value:f64)->Result<Value,Error>{
 serde_json::Number::from_f64(value).map(Value::Number).ok_or_else(||failure("number-nonfinite",0))
}

/// Budgets for already-owned dynamic values. No input byte stream is parsed.
#[derive(Clone, Copy, Debug)]
pub struct BuildLimits {pub depth:usize,pub entries:usize,pub nodes:usize}
struct BuildBudget {limits:BuildLimits,nodes:usize}
impl BuildBudget {
 fn root(limits:BuildLimits,entries:usize)->Result<Self,Error>{
  if limits.depth>64{return Err(invalid_limits());}
  if limits.nodes==0{return Err(failure("node-limit",0));}
  if limits.depth==0{return Err(failure("depth-limit",0));}
  if entries>limits.entries{return Err(failure("entry-limit",0));}
  if entries>limits.nodes-1{return Err(failure("node-limit",0));}
  Ok(Self{limits,nodes:1})
 }
 fn visit(&mut self,value:&Value,depth:usize)->Result<(),Error>{
  if self.nodes>=self.limits.nodes{return Err(failure("node-limit",0));}
  self.nodes+=1;
  let entries=match value{Value::Array(values)=>values.len(),Value::Object(values)=>values.len(),_=>return Ok(())};
  if depth>=self.limits.depth{return Err(failure("depth-limit",0));}
  if entries>self.limits.entries{return Err(failure("entry-limit",0));}
  match value {
   Value::Array(values)=>for value in values{self.visit(value,depth+1)?;},
   Value::Object(values)=>for value in values.values(){self.visit(value,depth+1)?;},
   _=>unreachable!(),
  }
  Ok(())
 }
}
/// Reject impossible roots before callers allocate converted wrapper storage.
pub fn validate_build_root(limits:BuildLimits,entries:usize)->Result<(),Error>{
 BuildBudget::root(limits,entries).map(|_|())
}
/// Validates the new root and all descendants, then transfers the input allocation.
pub fn build_array(values:Vec<Value>,limits:BuildLimits)->Result<Value,Error>{
 let mut budget=BuildBudget::root(limits,values.len())?;
 for value in &values{budget.visit(value,1)?;}
 Ok(Value::Array(values))
}
/// Counts all supplied entries before duplicate replacement; the last key wins.
pub fn build_object(entries:Vec<(String,Value)>,limits:BuildLimits)->Result<Value,Error>{
 let mut budget=BuildBudget::root(limits,entries.len())?;
 for (_,value) in &entries{budget.visit(value,1)?;}
 let mut object=serde_json::Map::new();
 for (key,value) in entries{object.insert(key,value);}
 Ok(Value::Object(object))
}

#[cfg(test)]
mod construction_tests {
 use super::*;
 fn limits()->BuildLimits{BuildLimits{depth:4,entries:3,nodes:20}}
 #[test]
 fn owned_containers_preserve_payloads_and_duplicate_key_rules(){
  let text=String::from("owned payload");let allocation=text.as_ptr();
  let array=build_array(vec![Value::String(text),Value::from(i64::MAX)],limits()).unwrap();
  assert_eq!(array[0].as_str().unwrap().as_ptr(),allocation);
  assert_eq!(array[1].as_i64(),Some(i64::MAX));
  let object=build_object(vec![("z".into(),Value::Null),("a".into(),array),("z".into(),Value::Bool(true))],limits()).unwrap();
  assert_eq!(object["z"],Value::Bool(true));
  assert_eq!(object["a"][0].as_str().unwrap().as_ptr(),allocation);
  assert_eq!(object.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(),vec!["a","z"]);
 }
 #[test]
 fn construction_budgets_count_root_nested_values_and_replaced_entries(){
  assert_eq!(build_array(vec![],BuildLimits{depth:0,..limits()}).unwrap_err().code,"depth-limit");
  assert_eq!(build_array(vec![],BuildLimits{nodes:0,..limits()}).unwrap_err().code,"node-limit");
  assert_eq!(build_array(vec![],BuildLimits{depth:65,..limits()}).unwrap_err().code,"invalid-limits");
  assert!(build_array(vec![],BuildLimits{depth:1,entries:0,nodes:1}).is_ok());
  assert_eq!(build_array(vec![Value::Array(vec![])],BuildLimits{depth:1,..limits()}).unwrap_err().code,"depth-limit");
  assert_eq!(build_array(vec![Value::Null;4],limits()).unwrap_err().code,"entry-limit");
  assert_eq!(build_array(vec![Value::Array(vec![Value::Null;4])],limits()).unwrap_err().code,"entry-limit");
  assert_eq!(build_object(vec![("same".into(),Value::Null);4],limits()).unwrap_err().code,"entry-limit");
  assert_eq!(build_object(vec![("same".into(),Value::Null);3],BuildLimits{nodes:3,..limits()}).unwrap_err().code,"node-limit");
  assert!(build_object(vec![("same".into(),Value::Null);3],BuildLimits{nodes:4,..limits()}).is_ok());
 }
}

#[cfg(test)]
mod adapter_tests {
    use super::*;
    #[test]
    fn scalar_and_container_access_preserve_types_missing_values_and_allocations() {
        for (value, expected, category) in [
            (Value::Null,"null","object"), (Value::Bool(true),"boolean","boolean"),
            (Value::from(7),"number","number"), (Value::from("é"),"string","string"),
            (serde_json::json!([]),"array","object"), (serde_json::json!({}),"object","object"),
        ] { assert_eq!(kind(&value),expected); assert_eq!(type_of(&value),category); }
        assert!(read_bool(&Value::Bool(true)).unwrap());
        assert_eq!(read_bool(&Value::from(1)).unwrap_err().code,"expected-boolean");
        let text=Value::String("owned é".into()); let pointer=read_string(&text).unwrap().as_ptr();
        let moved=take_string(text).unwrap();assert_eq!(moved.as_ptr(),pointer);
        assert_eq!(read_string(&Value::Null).unwrap_err().code,"expected-string");
        assert_eq!(take_string(Value::Null).unwrap_err().code,"expected-string");
        let object=serde_json::json!({"present":null});
        assert_eq!(get_key(&object,"missing").unwrap(),None);
        assert_eq!(get_key(&object,"present").unwrap(),Some(&Value::Null));
        assert_eq!(take_key(object,"present").unwrap(),Some(Value::Null));
        let text=String::from("retained allocation");let pointer=text.as_ptr();
        let array=Value::Array(vec![Value::Null,Value::String(text)]);
        assert_eq!(get_index(&array,-1).unwrap_err().code,"index-range");
        assert_eq!(get_index(&array,2).unwrap(),None);
        assert_eq!(get_index(&array,0).unwrap(),Some(&Value::Null));
        let selected=take_string(take_index(array,1).unwrap().unwrap()).unwrap();
        assert_eq!(selected.as_ptr(),pointer);
        assert_eq!(get_key(&Value::Null,"x").unwrap_err().code,"expected-object");
        assert_eq!(take_key(Value::Null,"x").unwrap_err().code,"expected-object");
        assert_eq!(take_index(Value::Null,0).unwrap_err().code,"expected-array");
    }
    #[test]
    fn generated_source_adapters_borrow_the_original_value() {
        struct Owner(Value);
        impl Source for Owner { fn json_value(&self)->&Value { &self.0 } }
        impl<'a> ViewSource<'a> for &'a Owner { fn json_view(self)->&'a Value { &self.0 } }
        fn owned(source:&impl Source)->&Value { source.json_value() }
        fn view<'a>(source:impl ViewSource<'a>)->&'a Value { source.json_view() }
        let owner=Owner(serde_json::json!(["payload"]));
        assert!(std::ptr::eq(owned(&owner),&owner.0));
        assert!(std::ptr::eq(view(&owner),&owner.0));
    }
    #[test]
    fn finite_numbers_and_root_budgets_reject_without_coercion() {
        assert!(from_f64(-0.0).unwrap().as_f64().unwrap().is_sign_negative());
        for value in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY] {
            assert_eq!(from_f64(value).unwrap_err().code,"number-nonfinite");
        }
        let limits=BuildLimits{depth:1,entries:2,nodes:3};
        assert!(validate_build_root(limits,2).is_ok());
        assert_eq!(validate_build_root(limits,3).unwrap_err().code,"entry-limit");
        assert_eq!(validate_build_root(BuildLimits{nodes:2,..limits},2).unwrap_err().code,"node-limit");
    }
}
