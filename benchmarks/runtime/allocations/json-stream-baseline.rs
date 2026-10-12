// Typed Serde baseline with a reusable line buffer and bounded file buffer.
struct Item { name: String, values: Vec<i64>, active: bool }
impl<'de> serde::Deserialize<'de> for Item {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> serde::de::Visitor<'de> for Fields {
            type Value = Item;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("an item") }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Item,A::Error> {
                let (mut name,mut values,mut active)=(None,None,None);
                while let Some(key)=map.next_key::<String>()? {
                    match key.as_str() {
                        "name"=>name=Some(map.next_value()?),
                        "values"=>values=Some(map.next_value()?),
                        "active"=>active=Some(map.next_value()?),
                        _=>{let _:serde::de::IgnoredAny=map.next_value()?;}
                    }
                }
                Ok(Item {
                    name:name.ok_or_else(||serde::de::Error::missing_field("name"))?,
                    values:values.ok_or_else(||serde::de::Error::missing_field("values"))?,
                    active:active.ok_or_else(||serde::de::Error::missing_field("active"))?,
                })
            }
        }
        decoder.deserialize_struct("Item",&["name","values","active"],Fields)
    }
}
fn stream(path:&str)->i64 {
    use std::io::BufRead;
    let mut reader=std::io::BufReader::with_capacity(8192,std::fs::File::open(path).unwrap());
    let mut line=String::new();
    let mut checksum=7;
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap()==0 {break;}
        let item:Item=serde_json::from_str(&line).unwrap();
        assert!(item.active);
        checksum+=item.name.len() as i64+item.values[0]+item.values[1];
    }
    checksum
}
