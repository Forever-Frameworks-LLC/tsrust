//! Direct typed decoding uses bounded validation without a dynamic value tree.
use super::{Budget, Error, Limits, failure, located};

#[cfg(test)]
mod tests {
 use super::*;
 fn limits()->Limits{Limits{bytes:1024,depth:4,entries:3,nodes:12}}
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

/// Internal boundary for generated typed visitors. Primitive number fields must use ExactNumber.
pub fn decode<T:serde::de::DeserializeOwned>(input:&[u8],limits:Limits)->Result<T,Error>{
 validate(input,limits)?;
 serde_json::from_slice(input).map_err(|error|{
  let text=error.to_string();
  let code=if error.is_data(){["number-precision","integer-range","integer-fractional"].into_iter().find(|code|text.starts_with(&format!("tsrust:{code}"))).unwrap_or("shape-mismatch")}else{"shape-mismatch"};
  located(input,error,Some(code))
 })
}
fn validate(input:&[u8],limits:Limits)->Result<(),Error>{
 if input.len()>limits.bytes{return Err(failure("byte-limit",limits.bytes));}
 if limits.depth>64{return Err(super::invalid_limits());}
 if let Err(error)=std::str::from_utf8(input){return Err(failure("invalid-utf8",error.valid_up_to()));}
 let mut budget=Budget{limits,nodes:0,failed:None};
 let mut decoder=serde_json::Deserializer::from_slice(input);
 serde::de::DeserializeSeed::deserialize(ValidateSeed{budget:&mut budget,depth:0,permitted:true},&mut decoder).map_err(|error|located(input,error,budget.failed))?;
 decoder.end().map_err(|error|located(input,error,Some("trailing-data")))
}
struct ValidateSeed<'a>{budget:&'a mut Budget,depth:usize,permitted:bool}
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
 fn visit_f64<E:serde::de::Error>(self,_:f64)->Result<(),E>{Ok(())}
 fn visit_str<E:serde::de::Error>(self,_:&str)->Result<(),E>{Ok(())}
 fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut sequence:A)->Result<(),A::Error>{
  use serde::de::Error;
  if self.depth>=self.budget.limits.depth{return Err(A::Error::custom(self.budget.fail("depth-limit")));}
  let mut entries=0;
  loop{let permitted=entries<self.budget.limits.entries;match sequence.next_element_seed(ValidateSeed{budget:self.budget,depth:self.depth+1,permitted})?{Some(())=>entries+=1,None=>return Ok(())}}
 }
 fn visit_map<A:serde::de::MapAccess<'de>>(self,mut object:A)->Result<(),A::Error>{
  use serde::de::Error;
  if self.depth>=self.budget.limits.depth{return Err(A::Error::custom(self.budget.fail("depth-limit")));}
  let mut entries=0;
  loop{
   let permitted=entries<self.budget.limits.entries;
   if object.next_key_seed(ValidateKey{budget:self.budget,permitted})?.is_none(){return Ok(());}
   object.next_value_seed(ValidateSeed{budget:self.budget,depth:self.depth+1,permitted:true})?;entries+=1;
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
