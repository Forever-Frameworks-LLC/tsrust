struct Item { active: bool, items: Vec<Vec<i64>>, name: String }
impl serde::Serialize for Item {
    fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
        use serde::ser::SerializeStruct;
        let mut record=serializer.serialize_struct("Item",3)?;
        record.serialize_field("active",&self.active)?;
        record.serialize_field("items",&self.items)?;
        record.serialize_field("name",&self.name)?;
        record.end()
    }
}
// Preparation is outside the counters; only the typed record remains afterward.
fn prepare_object(text:&str)->Item {
    let raw:serde_json::Value=serde_json::from_str(text).unwrap();
    let item=Item {
        active:raw["active"].as_bool().unwrap(),
        items:raw["items"].as_array().unwrap().iter().map(|row|row.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect()).collect(),
        name:raw["name"].as_str().unwrap().to_owned(),
    };
    drop(raw);
    item
}
