//! The fifteen catalog references share fourteen verbatim executable examples.
pub const REFERENCES:&[(&str,&str,usize,&str,&str)]=&[
 ("std:iterator","Iterator",0,"iterator catalog examples > from",include_str!("../fixtures/owned-iterators/catalog/from.tsr")),
 ("std:iterator","IteratorResult",0,"iterator catalog examples > result",include_str!("../fixtures/owned-iterators/catalog/result.tsr")),
 ("IteratorConstructor","from",0,"iterator catalog examples > from",include_str!("../fixtures/owned-iterators/catalog/from.tsr")),
 ("Iterator","next",0,"iterator catalog examples > next",include_str!("../fixtures/owned-iterators/catalog/next.tsr")),
 ("Iterator","map",0,"iterator catalog examples > map",include_str!("../fixtures/owned-iterators/catalog/map.tsr")),
 ("Iterator","filter",0,"iterator catalog examples > filter",include_str!("../fixtures/owned-iterators/catalog/filter.tsr")),
 ("Iterator","take",0,"iterator catalog examples > take",include_str!("../fixtures/owned-iterators/catalog/take.tsr")),
 ("Iterator","drop",0,"iterator catalog examples > drop",include_str!("../fixtures/owned-iterators/catalog/drop.tsr")),
 ("Iterator","flatMap",0,"iterator catalog examples > flat-map",include_str!("../fixtures/owned-iterators/catalog/flat-map.tsr")),
 ("Iterator","reduce",0,"iterator catalog examples > reduce",include_str!("../fixtures/owned-iterators/catalog/reduce.tsr")),
 ("Iterator","forEach",0,"iterator catalog examples > for-each",include_str!("../fixtures/owned-iterators/catalog/for-each.tsr")),
 ("Iterator","some",0,"iterator catalog examples > some",include_str!("../fixtures/owned-iterators/catalog/some.tsr")),
 ("Iterator","every",0,"iterator catalog examples > every",include_str!("../fixtures/owned-iterators/catalog/every.tsr")),
 ("Iterator","find",0,"iterator catalog examples > find",include_str!("../fixtures/owned-iterators/catalog/find.tsr")),
 ("Iterator","toArray",0,"iterator catalog examples > to-array",include_str!("../fixtures/owned-iterators/catalog/to-array.tsr")),
 ];
pub fn catalog()->serde_json::Value{serde_json::from_str(include_str!("../../../tsrust-lsp/src/builtins.json")).unwrap()}
pub fn example<'a>(catalog:&'a serde_json::Value,receiver:&str,name:&str,index:usize)->&'a serde_json::Value{&catalog[receiver].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap_or_else(||panic!("Missing {receiver}.{name}"))["examples"][index]}
pub fn case_names()->Vec<&'static str>{REFERENCES.iter().map(|row|row.3).collect::<std::collections::BTreeSet<_>>().into_iter().collect()}
pub fn assert_sources(){
 let catalog=catalog();
 let actual=["std:iterator","IteratorConstructor","Iterator"].into_iter().map(|receiver|catalog[receiver].as_array().unwrap().iter().map(|entry|entry["examples"].as_array().unwrap().len()).sum::<usize>()).sum::<usize>();assert_eq!(actual,15);assert_eq!(REFERENCES.len(),15);assert_eq!(case_names().len(),14);
 let mut keys=std::collections::BTreeSet::new();
 for &(receiver,name,index,_,fixture) in REFERENCES{
  assert!(keys.insert((receiver,name,index)),"duplicate catalog reference");let example=example(&catalog,receiver,name,index);let source=example["source"].as_str().unwrap();
  let token=if fixture.contains("export async function main"){"export async function main"}else{"export function main"};assert_eq!(fixture.matches(token).count(),1,"{receiver}.{name}");let authored=fixture.replacen(token,token.strip_prefix("export ").unwrap(),1);
  assert_eq!(crate::support::format_round_trip(source),crate::support::format_round_trip(&authored),"catalog fixture must differ only by added export: {receiver}.{name}");crate::support::format_round_trip(fixture);
 }
}
