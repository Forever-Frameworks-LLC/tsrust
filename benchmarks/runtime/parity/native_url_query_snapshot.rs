// Handwritten ordered-pair snapshot; no shared Tsrust facade or live URL view.
type String = std::borrow::Cow<'static, str>;
fn encode(rows: &[(std::string::String, std::string::String)]) -> std::string::String {
    url::form_urlencoded::Serializer::new(std::string::String::new()).extend_pairs(rows.iter().map(|(name,value)|(name,value))).finish()
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let mut value = url::Url::parse(value2.as_ref()).unwrap();
        let mut params: Vec<_> = value.query_pairs().into_owned().collect();
        let rows = params.clone();
        let first = params.iter().position(|(name,_)|name=="tag").unwrap();
        params[first].1 = "next".to_owned();
        let mut index = 0;
        params.retain(|(name,_)|{let keep = name!="tag" || index==first;index+=1;keep});
        params.push(("q".to_owned(), "a b+".to_owned()));
        params.sort_by(|left,right|left.0.cmp(&right.0));
        let encoded = encode(&params);
        assert_eq!(encoded, "q=a+b%2B&tag=next&z=last");
        assert_eq!(rows, [("tag".to_owned(),"one".to_owned()),("tag".to_owned(),"two".to_owned()),("z".to_owned(),"last".to_owned())]);
        value.set_query(Some(&encode(&params)));
        params.retain(|(name,_)|name!="tag");
        let result = &value[url::Position::AfterPath..url::Position::AfterQuery];
        assert_eq!(result, "?q=a+b%2B&tag=next&z=last");
        let observed = encoded.len() + result.len() + rows[0].1.len() + rows[1].1.len() + rows[2].1.len();
        checksum = checksum.checked_add(i64::try_from(observed).unwrap()).unwrap();
    }
    checksum
}
fn main() {}
