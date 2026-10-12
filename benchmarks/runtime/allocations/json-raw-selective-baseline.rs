struct SelectedRaw<'a> { payload: &'a serde_json::value::RawValue, tag: i64 }
impl<'de> serde::Deserialize<'de> for SelectedRaw<'de> {
 fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{
  struct Fields;
  impl<'de> serde::de::Visitor<'de> for Fields {
   type Value=SelectedRaw<'de>;
   fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("a raw payload envelope")}
   fn visit_map<A:serde::de::MapAccess<'de>>(self,mut map:A)->Result<Self::Value,A::Error>{
    let mut payload=None;let mut tag=None;
    while let Some(key)=map.next_key::<std::string::String>()? {
     match key.as_str(){"payload"=>payload=Some(map.next_value()?),"tag"=>tag=Some(map.next_value()?),_=>{let _:serde::de::IgnoredAny=map.next_value()?;}}
    }
    Ok(SelectedRaw{payload:payload.ok_or_else(||serde::de::Error::missing_field("payload"))?,tag:tag.ok_or_else(||serde::de::Error::missing_field("tag"))?})
   }
  }
  decoder.deserialize_map(Fields)
 }
}
