// No Clone implementation: borrowed queries must not duplicate owned keys.
struct String(std::string::String);
impl AsRef<str> for String { fn as_ref(&self)->&str { &self.0 } }
struct Window<'a>(&'a [u8]);
fn main() {
    let bytes=[1,2,3];
    let mut map=TsrOrderedMap::new();
    map.insert(TsrOrdered::new(String("b".into())),Window(&bytes[1..]));
    map.insert(TsrOrdered::new(String("a".into())),Window(&bytes[..1]));
    let query=String("b".into());let address=query.0.as_ptr();
    assert!(map.contains_string_key(&query));
    assert_eq!(map.get_string_key(&query).unwrap().0.as_ptr(),bytes[1..].as_ptr());
    map.get_mut_string_key(&query).unwrap().0=&bytes[..2];
    assert_eq!(map.source_iter().map(|(key,_)|key.as_ref()).collect::<Vec<_>>(),["a","b"]);
    let moved=map.remove_string_key(&query).unwrap();assert_eq!(moved.0,&[1,2]);
    assert!(!map.contains_string_key(&query));assert_eq!(query.0.as_ptr(),address);
    let entries=map.into_source_entries().collect::<Vec<_>>();assert_eq!(entries[0].0.as_ref(),"a");assert_eq!(entries[0].1.0.as_ptr(),bytes.as_ptr());
    let mut map=TsrOrderedMap::new();map.insert(TsrOrdered::new(String("b".into())),Window(&bytes));
    assert_eq!(map.into_value_string_key(&query).unwrap().0.as_ptr(),bytes.as_ptr());
    let mut numbers=TsrOrderedMap::new();numbers.insert(TsrOrdered::new(-0.0),vec![1]);numbers.insert(TsrOrdered::new(f64::NAN),vec![2]);
    assert!(numbers.contains_copy_key(0.0));assert_eq!(numbers.get_copy_key(-0.0),Some(&vec![1]));
    numbers.get_mut_copy_key(f64::NAN).unwrap().push(3);assert_eq!(numbers.remove_copy_key(f64::from_bits(0xfff8_0000_0000_0042)),Some(vec![2,3]));
    assert_eq!(numbers.into_value_copy_key(0.0),Some(vec![1]));
    struct Probe<'a>(&'a std::cell::Cell<usize>);
    impl Drop for Probe<'_>{fn drop(&mut self){self.0.set(self.0.get()+1);}}
    let drops=std::cell::Cell::new(0);
    {let mut map=TsrOrderedMap::new();map.insert(TsrOrdered::new(1_i64),Probe(&drops));map.insert(TsrOrdered::new(2_i64),Probe(&drops));let moved=map.into_value_copy_key(1).unwrap();assert_eq!(drops.get(),1);drop(moved);}
    assert_eq!(drops.get(),2);
}
