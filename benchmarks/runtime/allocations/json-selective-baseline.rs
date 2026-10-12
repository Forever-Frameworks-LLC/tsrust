// Direct Serde selective-record baseline; ignored values are streamed and discarded.
struct Selected { items: Vec<Vec<i64>> }
impl<'de> serde::Deserialize<'de> for Selected {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> serde::de::Visitor<'de> for Fields {
            type Value = Selected;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an items record")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Selected, A::Error> {
                let mut items = None;
                while let Some(key) = map.next_key::<std::string::String>()? {
                    if key == "items" {
                        items = Some(map.next_value()?);
                    } else {
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }
                Ok(Selected { items: items.ok_or_else(|| serde::de::Error::missing_field("items"))? })
            }
        }
        decoder.deserialize_struct("Selected", &["items"], Fields)
    }
}
