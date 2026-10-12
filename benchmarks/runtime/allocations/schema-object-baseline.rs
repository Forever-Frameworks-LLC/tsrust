// Direct Serde baseline for the strict reusable schema object probe.
struct SchemaPayload { items: Vec<i64> }
impl<'de> serde::Deserialize<'de> for SchemaPayload {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> serde::de::Visitor<'de> for Fields {
            type Value = SchemaPayload;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an items record")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<SchemaPayload, A::Error> {
                let mut items = None;
                while let Some(key) = map.next_key::<std::string::String>()? {
                    if key == "items" {
                        items = Some(map.next_value()?);
                    } else {
                        return Err(serde::de::Error::unknown_field(&key, &["items"]));
                    }
                }
                Ok(SchemaPayload { items: items.ok_or_else(|| serde::de::Error::missing_field("items"))? })
            }
        }
        decoder.deserialize_struct("SchemaPayload", &["items"], Fields)
    }
}
