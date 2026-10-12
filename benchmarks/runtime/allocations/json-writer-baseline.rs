struct Item { active: bool, name: &'static str, values: [i64;2] }
impl serde::Serialize for Item {
    fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
        use serde::ser::SerializeStruct;
        let mut record=serializer.serialize_struct("Item",3)?;
        record.serialize_field("active",&self.active)?;
        record.serialize_field("name",&self.name)?;
        record.serialize_field("values",&self.values)?;
        record.end()
    }
}
fn stream_write(n:i64,path:&str)->i64 {
    use std::io::Write;
    let mut writer=std::io::BufWriter::with_capacity(8192,std::fs::File::create(path).unwrap());
    let item=Item{active:true,name:"hello",values:[20,17]};
    for _ in 0..n {
        let record=serde_json::to_string(&item).unwrap();
        assert!(record.len()<=16777216);
        writer.write_all(record.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }
    writer.flush().unwrap();
    7+n
}
