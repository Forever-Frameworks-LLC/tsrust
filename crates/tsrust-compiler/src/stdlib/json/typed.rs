//! Direct typed decoding uses bounded validation without a dynamic value tree.
pub use serde;
use super::{Budget, Error, Limits, failure, located};

#[cfg(test)]
mod tests {
 use super::*;
 fn limits()->Limits{Limits{bytes:1024,depth:4,entries:3,nodes:12}}
 #[test]
 fn validation_reports_structural_depth_with_existing_limits(){
  for (input,depth) in [("null",0),("[]",1),("{}",1),(r#"{"x":[{},[0]],"s":"[\"]"}"#,3)] {
   assert_eq!(validate_depth(input.as_bytes(),limits()).unwrap(),depth);
  }
  let deep="[".repeat(64)+"0"+&"]".repeat(64);
  assert_eq!(validate_depth(deep.as_bytes(),Limits{depth:64,nodes:100,..limits()}).unwrap(),64);
  assert_eq!(validate_depth(b"[[]]",Limits{depth:1,..limits()}).unwrap_err().code,"depth-limit");
  assert_eq!(validate_depth(b"null null",limits()).unwrap_err().code,"trailing-data");
 }
 #[test]
 fn typed_borrowed_fields_retain_input_and_reject_escaped_replacements(){
  let input=String::from(r#"{"name":"hello"}"#);
  let value:std::collections::BTreeMap<&str,&str>=decode_borrowed(input.as_bytes(),limits()).unwrap();
  assert_eq!(value["name"],"hello");
  assert_eq!(value["name"].as_ptr(),input[9..].as_ptr());
  assert_eq!(decode_borrowed::<std::collections::BTreeMap<&str,&str>>(br#"{"name":"he\u006clo"}"#,limits()).unwrap_err().code,"shape-mismatch");
  assert_eq!(decode_borrowed::<&str>(br#""hello""#,Limits{bytes:2,..limits()}).unwrap_err().code,"byte-limit");
 }
 #[test]
 fn typed_jsonc_preserves_offsets_limits_and_strict_policy(){
  let input=b"/* header */ {\"items\":[1,2,],}";
  let value:Record=decode_jsonc(input,limits(),true).unwrap();
  assert_eq!(value,Record{items:vec![1,2]});
  assert!(decode_jsonc::<Record>(input,limits(),false).is_err());
  assert!(decode::<Record>(input,limits()).is_err());
  let bad=b"/* header */ {\"items\":[1,\"secret\"]}";
  let normalized=b"             {\"items\":[1,\"secret\"]}";
  let error=decode_jsonc::<Record>(bad,limits(),false).unwrap_err();
  assert_eq!(error,decode::<Record>(normalized,limits()).unwrap_err());
  assert_eq!(error.code,"shape-mismatch");assert!(!error.message.contains("secret"));
  assert_eq!(decode_jsonc::<Record>(input,Limits{bytes:4,..limits()},true).unwrap_err().code,"byte-limit");
  assert_eq!(decode_jsonc::<Record>(b"{\"items\":[1,2,3,4]}",limits(),false).unwrap_err().code,"entry-limit");
  assert_eq!(decode_jsonc::<Record>(b"/* open",limits(),false).unwrap_err().code,"unterminated-comment");
 }
 #[test]
 fn typed_arrays_and_maps_decode_without_dynamic_values(){
  let values:Vec<(String,Vec<i64>)>=decode(br#"[["hello",[42,-1]]]"#,limits()).unwrap();
  assert_eq!(values,vec![("hello".into(),vec![42,-1])]);
  let record:std::collections::BTreeMap<String,Vec<i64>>=decode(br#"{"items":[1,2]}"#,limits()).unwrap();
  assert_eq!(record["items"],vec![1,2]);
 }
 #[test]
 fn typed_decoding_enforces_limits_even_for_ignored_values(){
  assert_eq!(decode::<String>(br#""\ud83d\ude00""#,limits()).unwrap(),"😀");
  assert_eq!(decode::<serde::de::IgnoredAny>(b"[",limits()).unwrap_err().code,"unexpected-end");
  assert_eq!(decode::<serde::de::IgnoredAny>(b"1e400",limits()).unwrap_err().code,"invalid-json");
  assert_eq!(decode::<serde::de::IgnoredAny>(b"[0,1,2,3]",limits()).unwrap_err().code,"entry-limit");
  assert_eq!(decode::<serde::de::IgnoredAny>(b"[[[]]]",Limits{depth:2,..limits()}).unwrap_err().code,"depth-limit");
  assert_eq!(decode::<Vec<i64>>(b"[1,2]",Limits{nodes:2,..limits()}).unwrap_err().code,"node-limit");
  assert_eq!(decode::<Vec<i64>>(b"[1]",Limits{bytes:2,..limits()}).unwrap_err().code,"byte-limit");
  assert_eq!(decode::<Vec<i64>>(b"[1] null",limits()).unwrap_err().code,"trailing-data");
  assert_eq!(decode::<i64>(b"\"secret\"",limits()).unwrap_err().message,"JSON value does not match the requested type");
  assert_eq!(decode::<String>(&[b'"',255,b'"'],limits()).unwrap_err().code,"invalid-utf8");
 }
 #[derive(Debug,PartialEq)]
 struct Record{items:Vec<i64>}
 impl<'de> serde::Deserialize<'de> for Record{
  fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{
   struct Fields;
   impl<'de> serde::de::Visitor<'de> for Fields{
    type Value=Record;
    fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("an items record")}
    fn visit_map<A:serde::de::MapAccess<'de>>(self,mut map:A)->Result<Record,A::Error>{
     let mut items=None;
     while let Some(key)=map.next_key::<String>()?{if key=="items"{items=Some(map.next_value()?);}else{let _:serde::de::IgnoredAny=map.next_value()?;}}
     Ok(Record{items:items.ok_or_else(||serde::de::Error::missing_field("items"))?})
    }
   }
   decoder.deserialize_struct("Record",&["items"],Fields)
  }
 }
 #[test]
 fn typed_records_validate_ignored_fields_and_keep_last_duplicate(){
  assert_eq!(decode::<Record>(br#"{"items":[1],"items":[2]}"#,limits()).unwrap(),Record{items:vec![2]});
  assert_eq!(decode::<Record>(br#"{"ignored":[0,1,2,3],"items":[]}"#,limits()).unwrap_err().code,"entry-limit");
  assert_eq!(decode::<Record>(br#"{"ignored":null}"#,limits()).unwrap_err().code,"shape-mismatch");
 }
 #[test]
 fn partially_decoded_owned_values_are_dropped_on_failure(){
  use std::sync::atomic::{AtomicUsize,Ordering};static DROPS:AtomicUsize=AtomicUsize::new(0);
  #[derive(Debug)]struct Token;
  impl Drop for Token{fn drop(&mut self){DROPS.fetch_add(1,Ordering::SeqCst);}}
  impl<'de> serde::Deserialize<'de> for Token{
   fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{let _=i64::deserialize(decoder)?;Ok(Token)}
  }
  DROPS.store(0,Ordering::SeqCst);
  assert_eq!(decode::<Vec<Token>>(br#"[1,"wrong"]"#,limits()).unwrap_err().code,"shape-mismatch");
  assert_eq!(DROPS.load(Ordering::SeqCst),1);
 }
 #[test]
 fn typed_numbers_never_round_integer_payloads(){
  assert_eq!(decode::<ExactInt>(b"1.0",limits()).unwrap().0,1);
  assert_eq!(decode::<ExactInt>(b"-0.0",limits()).unwrap().0,0);
  assert_eq!(decode::<ExactInt>(b"1.5",limits()).unwrap_err().code,"integer-fractional");
  assert_eq!(decode::<ExactInt>(b"18446744073709551615",limits()).unwrap_err().code,"integer-range");
  assert_eq!(decode::<i64>(b"9223372036854775807",limits()).unwrap(),i64::MAX);
  assert_eq!(decode::<ExactNumber>(b"9007199254740993",limits()).unwrap_err().code,"number-precision");
  assert_eq!(decode::<ExactNumber>(b"18446744073709551615",limits()).unwrap_err().code,"number-precision");
  assert_eq!(decode::<ExactNumber>(b"9007199254740992",limits()).unwrap().0,9007199254740992.0);
  assert!(decode::<ExactNumber>(b"-0.0",limits()).unwrap().0.is_sign_negative());
 }
}

/// Generated numeric fields use this adapter instead of Serde's lossy integer-to-f64 coercion.
#[derive(Debug)]
pub struct ExactNumber(pub f64);
impl<'de> serde::Deserialize<'de> for ExactNumber {
 fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{
  struct NumberVisitor;
  impl<'de> serde::de::Visitor<'de> for NumberVisitor {
   type Value=ExactNumber;
   fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("an exactly representable JSON number")}
   fn visit_i64<E:serde::de::Error>(self,value:i64)->Result<ExactNumber,E>{self.visit_i128(i128::from(value))}
   fn visit_u64<E:serde::de::Error>(self,value:u64)->Result<ExactNumber,E>{self.visit_i128(i128::from(value))}
   fn visit_i128<E:serde::de::Error>(self,value:i128)->Result<ExactNumber,E>{
    let float=value as f64;if float as i128!=value{return Err(E::custom("tsrust:number-precision"));}Ok(ExactNumber(float))
   }
   fn visit_f64<E:serde::de::Error>(self,value:f64)->Result<ExactNumber,E>{Ok(ExactNumber(value))}
  }
  decoder.deserialize_any(NumberVisitor)
 }
}

/// Generated int fields use the same checked integral conversion as asInt.
#[derive(Debug)]
pub struct ExactInt(pub i64);
impl<'de> serde::Deserialize<'de> for ExactInt {
 fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{
  struct IntVisitor;
  impl<'de> serde::de::Visitor<'de> for IntVisitor {
   type Value=ExactInt;
   fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("a signed integer JSON value")}
   fn visit_i64<E:serde::de::Error>(self,value:i64)->Result<ExactInt,E>{Ok(ExactInt(value))}
   fn visit_u64<E:serde::de::Error>(self,value:u64)->Result<ExactInt,E>{i64::try_from(value).map(ExactInt).map_err(|_|E::custom("tsrust:integer-range"))}
   fn visit_f64<E:serde::de::Error>(self,value:f64)->Result<ExactInt,E>{
    super::read_i64(&super::Value::from(value)).map(ExactInt).map_err(|error|E::custom(format!("tsrust:{}",error.code)))
   }
  }
  decoder.deserialize_any(IntVisitor)
 }
}

/// Normalize bounded JSONC while retaining source offsets, then decode directly.
pub fn decode_jsonc<T:serde::de::DeserializeOwned>(input:&[u8],limits:Limits,trailing_comma:bool)->Result<T,Error>{
 let normalized=super::normalize_jsonc(input,limits,trailing_comma)?;
 decode(&normalized,limits)
}

/// Internal boundary for generated typed visitors. Primitive number fields must use ExactNumber.
pub fn decode<T:serde::de::DeserializeOwned>(input:&[u8],limits:Limits)->Result<T,Error>{
 decode_borrowed(input,limits)
}

/// Borrowing boundary for source-backed fields. Returned borrows cannot outlive input.
/// JSONC normalization and streaming retain the owned boundary above.
pub fn decode_borrowed<'de,T:serde::Deserialize<'de>>(input:&'de [u8],limits:Limits)->Result<T,Error>{
 decode_seed(input,limits,std::marker::PhantomData::<T>)
}

/// Stateful typed decoding for immutable reusable schema plans. The seed may
/// borrow a plan and issue collector independently of the source lifetime.
/// Validate the complete document first, including ignored/duplicate fields and
/// trailing bytes, so malformed input cannot invoke user callbacks or factories.
/// The seed then constructs typed output directly; no dynamic value tree is built.
pub fn decode_seed<'de,S:serde::de::DeserializeSeed<'de>>(
 input:&'de [u8],limits:Limits,seed:S,
)->Result<S::Value,Error>{
 // Only an integer token too large for i64/u64 reaches a visitor as f64.
 if validate_floats(input,limits)?.1{super::validate_integer_tokens(input)?;}
 decode_once(input,seed)
}

/// `JSON.parse<T>(text)` without limits: one serde decode, like hand-written
/// Rust. The linear integer-token scan keeps over-range integers from being
/// rounded into number fields. Limits, when wanted, are passed explicitly.
pub fn decode_plain<'de,T:serde::Deserialize<'de>>(input:&'de [u8])->Result<T,Error>{
 super::validate_integer_tokens(input)?;
 decode_once(input,std::marker::PhantomData::<T>)
}

fn decode_once<'de,S:serde::de::DeserializeSeed<'de>>(input:&'de [u8],seed:S)->Result<S::Value,Error>{
 let mut decoder=serde_json::Deserializer::from_slice(input);
 let value=seed.deserialize(&mut decoder).map_err(|error|{
  let text=error.to_string();
  let code=if error.is_data(){["number-precision","integer-range","integer-fractional"].into_iter().find(|code|text.starts_with(&format!("tsrust:{code}"))).unwrap_or("shape-mismatch")}else{"shape-mismatch"};
  located(input,error,Some(code))
 })?;
 decoder.end().map_err(|error|located(input,error,Some("trailing-data")))?;
 Ok(value)
}
/// Validate the document; true when any number was read as a float, the only
/// way an integer token too large for i64/u64 can appear.
pub(super) fn validate(input:&[u8],limits:Limits)->Result<bool,Error>{
 validate_floats(input,limits).map(|(_,saw_float)|saw_float)
}
pub(super) fn validate_depth(input:&[u8],limits:Limits)->Result<usize,Error>{
 validate_floats(input,limits).map(|(maximum,_)|maximum)
}
/// Validate the document and report its maximum depth and whether any number was a float.
fn validate_floats(input:&[u8],limits:Limits)->Result<(usize,bool),Error>{
 if input.len()>limits.bytes{return Err(failure("byte-limit",limits.bytes));}
 if limits.depth>64{return Err(super::invalid_limits());}
 if let Err(error)=std::str::from_utf8(input){return Err(failure("invalid-utf8",error.valid_up_to()));}
 let mut budget=Budget{limits,nodes:0,failed:None,saw_float:false};
 let mut decoder=serde_json::Deserializer::from_slice(input);
 let mut maximum=0;
 serde::de::DeserializeSeed::deserialize(ValidateSeed{budget:&mut budget,maximum:&mut maximum,depth:0,permitted:true},&mut decoder).map_err(|error|located(input,error,budget.failed))?;
 decoder.end().map_err(|error|located(input,error,Some("trailing-data")))?;
 Ok((maximum,budget.saw_float))
}
struct ValidateSeed<'a>{budget:&'a mut Budget,maximum:&'a mut usize,depth:usize,permitted:bool}
impl<'de> serde::de::DeserializeSeed<'de> for ValidateSeed<'_>{
 type Value=();
 fn deserialize<D:serde::Deserializer<'de>>(self,decoder:D)->Result<(),D::Error>{
  use serde::de::Error;
  if !self.permitted{return Err(D::Error::custom(self.budget.fail("entry-limit")));}
  if self.budget.nodes>=self.budget.limits.nodes{return Err(D::Error::custom(self.budget.fail("node-limit")));}
  self.budget.nodes+=1;decoder.deserialize_any(self)
 }
}
impl<'de> serde::de::Visitor<'de> for ValidateSeed<'_>{
 type Value=();
 fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("a JSON value")}
 fn visit_unit<E:serde::de::Error>(self)->Result<(),E>{Ok(())}
 fn visit_bool<E:serde::de::Error>(self,_:bool)->Result<(),E>{Ok(())}
 fn visit_i64<E:serde::de::Error>(self,_:i64)->Result<(),E>{Ok(())}
 fn visit_u64<E:serde::de::Error>(self,_:u64)->Result<(),E>{Ok(())}
 fn visit_f64<E:serde::de::Error>(self,_:f64)->Result<(),E>{self.budget.saw_float=true;Ok(())}
 fn visit_str<E:serde::de::Error>(self,_:&str)->Result<(),E>{Ok(())}
 fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut sequence:A)->Result<(),A::Error>{
  use serde::de::Error;
  if self.depth>=self.budget.limits.depth{return Err(A::Error::custom(self.budget.fail("depth-limit")));}
  *self.maximum=(*self.maximum).max(self.depth+1);
  let mut entries=0;
  loop{let permitted=entries<self.budget.limits.entries;match sequence.next_element_seed(ValidateSeed{budget:self.budget,maximum:self.maximum,depth:self.depth+1,permitted})?{Some(())=>entries+=1,None=>return Ok(())}}
 }
 fn visit_map<A:serde::de::MapAccess<'de>>(self,mut object:A)->Result<(),A::Error>{
  use serde::de::Error;
  if self.depth>=self.budget.limits.depth{return Err(A::Error::custom(self.budget.fail("depth-limit")));}
  *self.maximum=(*self.maximum).max(self.depth+1);
  let mut entries=0;
  loop{
   let permitted=entries<self.budget.limits.entries;
   if object.next_key_seed(ValidateKey{budget:self.budget,permitted})?.is_none(){return Ok(());}
   object.next_value_seed(ValidateSeed{budget:self.budget,maximum:self.maximum,depth:self.depth+1,permitted:true})?;entries+=1;
  }
 }
}
struct ValidateKey<'a>{budget:&'a mut Budget,permitted:bool}
impl<'de> serde::de::DeserializeSeed<'de> for ValidateKey<'_>{
 type Value=();
 fn deserialize<D:serde::Deserializer<'de>>(self,decoder:D)->Result<(),D::Error>{
  use serde::de::Error;
  if !self.permitted{return Err(D::Error::custom(self.budget.fail("entry-limit")));}
  struct Key;
  impl<'de> serde::de::Visitor<'de> for Key{
   type Value=();
   fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("a JSON object key")}
   fn visit_str<E:serde::de::Error>(self,_:&str)->Result<(),E>{Ok(())}
  }
  decoder.deserialize_str(Key)
 }
}

#[cfg(test)]
mod encoding_tests {
 use super::*;
 struct Record<'a>{name:&'a str,values:&'a [i64],active:bool}
 impl serde::Serialize for Record<'_>{
  fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
   use serde::ser::SerializeMap;
   let mut map=serializer.serialize_map(Some(3))?;
   map.serialize_entry("active",&self.active)?;
   map.serialize_entry("name",self.name)?;
   map.serialize_entry("values",self.values)?;
   map.end()
  }
 }
 #[test]
 fn direct_typed_encoding_borrows_records_and_preserves_integer_bits(){
  let name=String::from("hi\n😀");let values=vec![i64::MIN,i64::MAX];
  let record=Record{name:&name,values:&values,active:true};
  let expected="{\"active\":true,\"name\":\"hi\\n😀\",\"values\":[-9223372036854775808,9223372036854775807]}";
  assert_eq!(encode(&record,expected.len()).unwrap(),expected);
  assert_eq!(name,"hi\n😀");assert_eq!(values,vec![i64::MIN,i64::MAX]);
  let error=encode(&record,expected.len()-1).unwrap_err();assert_eq!(error.code,"output-limit");assert!(error.offset<=expected.len()-1);
  assert_eq!(encode(&record,0).unwrap_err().code,"output-limit");
 }
 #[test]
 fn direct_typed_encoding_stops_at_the_generated_container_bound(){
  struct Nested{remaining:usize,depth:usize}
  impl serde::Serialize for Nested{
   fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
    use serde::ser::SerializeSeq;
    if self.remaining==0{return serializer.serialize_i64(7);}
    let depth=encoding_container::<S::Error>(self.depth)?;
    let mut sequence=serializer.serialize_seq(Some(1))?;
    sequence.serialize_element(&Nested{remaining:self.remaining-1,depth})?;
    sequence.end()
   }
  }
  let expected=format!("{}7{}","[".repeat(64),"]".repeat(64));
  assert_eq!(encode(&Nested{remaining:64,depth:0},expected.len()).unwrap(),expected);
  let error=encode(&Nested{remaining:65,depth:0},1024).unwrap_err();
  assert_eq!(error.code,"depth-limit");assert_eq!(error.offset,64);
  assert_eq!(encoding_container::<serde_json::Error>(usize::MAX).unwrap_err().to_string(),"tsrust:depth-limit");
 }
 #[test]
 fn dynamic_values_share_the_typed_parent_depth_budget(){
  let value=serde_json::json!({"items":[1,2]});
  assert_eq!(encode(&Dynamic(&value,62),100).unwrap(),"{\"items\":[1,2]}");
  assert_eq!(encode(&Dynamic(&value,63),100).unwrap_err().code,"depth-limit");
  assert_eq!(encode(&Dynamic(&serde_json::json!(7),64),100).unwrap(),"7");
 }
 #[test]
 fn direct_typed_encoding_rejects_undefined_at_root_or_in_arrays(){
  let error=encode(&Undefined,32).unwrap_err();assert_eq!(error.code,"undefined-value");assert_eq!(error.offset,0);
  let error=encode(&(1,Undefined),32).unwrap_err();assert_eq!(error.code,"undefined-value");assert_eq!(error.offset,3);
 }
 #[test]
 fn direct_typed_encoding_rejects_nonfinite_and_redacts_serializer_errors(){
  for value in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY]{assert_eq!(encode(&FiniteNumber(value),32).unwrap_err().code,"number-nonfinite");}
  assert_eq!(encode(&FiniteNumber(-0.0),32).unwrap(),"-0.0");
  struct Bad;
  impl serde::Serialize for Bad{fn serialize<S:serde::Serializer>(&self,_:S)->Result<S::Ok,S::Error>{Err(serde::ser::Error::custom("secret content"))}}
  let error=encode(&Bad,32).unwrap_err();assert_eq!(error.code,"invalid-json");assert!(!error.message.contains("secret"));
 }
}


/// Generated visitors borrow source fields and write directly into bounded output.
pub fn encode<T:serde::Serialize + ?Sized>(value:&T,max_bytes:usize)->Result<String,Error>{
 super::serialize_bounded(value,max_bytes)
}
/// Serde's ordinary f64 serialization maps nonfinite values to null. Typed JSON
/// numbers use this adapter so a value is never silently changed into null.
pub struct FiniteNumber(pub f64);
impl serde::Serialize for FiniteNumber{
 fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
  if !self.0.is_finite(){return Err(serde::ser::Error::custom("tsrust:number-nonfinite"));}
  serializer.serialize_f64(self.0)
 }
}
/// Every generated record/array visitor enters before visiting its children.
/// Scalars retain their parent's depth; the root starts at zero.
pub fn encoding_container<E:serde::ser::Error>(depth:usize)->Result<usize,E>{
 if depth>=64{return Err(E::custom("tsrust:depth-limit"));}
 Ok(depth+1)
}

/// Object visitors omit undefined fields. All other positions reject them.
pub struct Undefined;
impl serde::Serialize for Undefined{
 fn serialize<S:serde::Serializer>(&self,_:S)->Result<S::Ok,S::Error>{
  Err(serde::ser::Error::custom("tsrust:undefined-value"))
 }
}

/// Borrow dynamic subtrees while sharing the containing typed visitor's depth.
pub struct Dynamic<'a>(pub &'a serde_json::Value, pub usize);
impl serde::Serialize for Dynamic<'_> {
 fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
  match self.0 {
   serde_json::Value::Array(values)=>{
    use serde::ser::SerializeSeq;
    let depth=encoding_container::<S::Error>(self.1)?;
    let mut output=serializer.serialize_seq(Some(values.len()))?;
    for value in values {output.serialize_element(&Dynamic(value,depth))?;}
    output.end()
   },
   serde_json::Value::Object(values)=>{
    use serde::ser::SerializeMap;
    let depth=encoding_container::<S::Error>(self.1)?;
    let mut output=serializer.serialize_map(Some(values.len()))?;
    for (key,value) in values {output.serialize_entry(key,&Dynamic(value,depth))?;}
    output.end()
   },
   value=>value.serialize(serializer),
  }
 }
}

#[cfg(test)]
mod seeded_decode_tests {
    use super::*;
    use std::cell::Cell;

    struct StringPlan { minimum: usize, invocations: Cell<usize> }
    struct StringSeed<'a>(&'a StringPlan);
    impl<'de> serde::de::DeserializeSeed<'de> for StringSeed<'_> {
        type Value = Option<String>;
        fn deserialize<D:serde::Deserializer<'de>>(self,decoder:D)->Result<Self::Value,D::Error> {
            let value=<String as serde::Deserialize>::deserialize(decoder)?;
            self.0.invocations.set(self.0.invocations.get()+1);
            Ok((value.chars().count()>=self.0.minimum).then_some(value))
        }
    }
    fn limits()->Limits { Limits {bytes:128,depth:4,entries:4,nodes:16} }

    #[test]
    fn reusable_seed_retains_runtime_plan_and_builds_owned_typed_output() {
        let plan=StringPlan {minimum:2,invocations:Cell::new(0)};
        assert_eq!(decode_seed(br#""hello""#,limits(),StringSeed(&plan)).unwrap(),Some("hello".into()));
        assert_eq!(decode_seed(br#""x""#,limits(),StringSeed(&plan)).unwrap(),None);
        assert_eq!(decode_seed(br#""\u00e9\ud83d\ude00""#,limits(),StringSeed(&plan)).unwrap(),Some("é😀".into()));
        assert_eq!(plan.invocations.get(),3);
    }

    #[test]
    fn syntax_and_resource_failures_never_invoke_seed_callbacks() {
        let plan=StringPlan {minimum:0,invocations:Cell::new(0)};
        for (input,budget,code) in [
            (&b"\"ok\" null"[..],limits(),"trailing-data"),
            (&b"\"unterminated"[..],limits(),"unexpected-end"),
            (&b"\"ok\""[..],Limits{bytes:2,..limits()},"byte-limit"),
            (&b"[0,1]"[..],Limits{nodes:1,..limits()},"node-limit"),
            (&b"{\"unused\":\"\\ud800\"}"[..],limits(),"invalid-json"),
        ] {
            assert_eq!(decode_seed(input,budget,StringSeed(&plan)).unwrap_err().code,code);
        }
        assert_eq!(plan.invocations.get(),0);
    }

    #[test]
    fn borrowed_seed_keeps_real_source_lifetime() {
        struct Borrow;
        impl<'de> serde::de::DeserializeSeed<'de> for Borrow {
            type Value=&'de str;
            fn deserialize<D:serde::Deserializer<'de>>(self,decoder:D)->Result<Self::Value,D::Error> {
                <&'de str as serde::Deserialize>::deserialize(decoder)
            }
        }
        let owner=String::from("\"source-backed\"");
        let value=decode_seed(owner.as_bytes(),limits(),Borrow).unwrap();
        assert_eq!(value,"source-backed");
        assert_eq!(value.as_ptr(),owner[1..].as_ptr());
    }
}
