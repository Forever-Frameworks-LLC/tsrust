use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
const REFERENCES:&[(&str,&str,&str,usize,&str,&str)]=&[
 ("std:crypto","randomBytes","",0,"crypto runtime > catalog-01",include_str!("../fixtures/crypto/catalog-01.tsr")),
 ("std:crypto","randomBytes","",1,"crypto runtime > catalog-02",include_str!("../fixtures/crypto/catalog-02.tsr")),
 ("std:crypto","sha256","",0,"crypto runtime > catalog-03",include_str!("../fixtures/crypto/catalog-03.tsr")),
 ("std:crypto","timingSafeEqual","",0,"crypto runtime > catalog-04",include_str!("../fixtures/crypto/catalog-04.tsr")),
 ("std:crypto","CryptoError","",0,"crypto runtime > catalog-05",include_str!("../fixtures/crypto/catalog-05.tsr")),
 ("std:crypto","CryptoError","code",0,"crypto runtime > catalog-06",include_str!("../fixtures/crypto/catalog-06.tsr")),
 ("std:crypto","CryptoError","message",0,"crypto runtime > catalog-07",include_str!("../fixtures/crypto/catalog-07.tsr")),
 ("std:crypto","sha384","",0,"crypto runtime > catalog-08",include_str!("../fixtures/crypto/catalog-08.tsr")),
 ("std:crypto","sha512","",0,"crypto runtime > catalog-09",include_str!("../fixtures/crypto/catalog-09.tsr")),
 ("std:crypto","createHasher","",0,"crypto runtime > catalog-10",include_str!("../fixtures/crypto/catalog-10.tsr")),
 ("std:crypto","Hasher","",0,"crypto runtime > catalog-10",include_str!("../fixtures/crypto/catalog-10.tsr")),
 ("std:crypto","hmacSha256","",0,"crypto runtime > catalog-11",include_str!("../fixtures/crypto/catalog-11.tsr")),
 ("std:crypto","verifyHmacSha256","",0,"crypto runtime > catalog-11",include_str!("../fixtures/crypto/catalog-11.tsr")),
 ("std:crypto","hkdfSha256","",0,"crypto runtime > catalog-12",include_str!("../fixtures/crypto/catalog-12.tsr")),
 ("std:crypto","randomInt","",0,"crypto runtime > catalog-05",include_str!("../fixtures/crypto/catalog-05.tsr")),
 ("std:crypto","hashPassword","",0,"crypto runtime > catalog-13",include_str!("../fixtures/crypto/catalog-13.tsr")),
 ("std:crypto","verifyPassword","",0,"crypto runtime > catalog-13",include_str!("../fixtures/crypto/catalog-13.tsr")),
 ("std:crypto","verifyPasswordBounded","",0,"crypto runtime > catalog-13",include_str!("../fixtures/crypto/catalog-13.tsr")),
 ("std:crypto","PasswordHashOptions","",0,"crypto runtime > catalog-13",include_str!("../fixtures/crypto/catalog-13.tsr")),
 ("std:crypto","PasswordHashOptions","memoryKiB",0,"crypto runtime > catalog-14",include_str!("../fixtures/crypto/catalog-14.tsr")),
 ("std:crypto","PasswordHashOptions","iterations",0,"crypto runtime > catalog-15",include_str!("../fixtures/crypto/catalog-15.tsr")),
 ("std:crypto","PasswordHashOptions","parallelism",0,"crypto runtime > catalog-16",include_str!("../fixtures/crypto/catalog-16.tsr")),
 ("std:crypto","PasswordHashOptions","outputBytes",0,"crypto runtime > catalog-17",include_str!("../fixtures/crypto/catalog-17.tsr")),
 ("std:crypto","PasswordVerifyOptions","",0,"crypto runtime > catalog-13",include_str!("../fixtures/crypto/catalog-13.tsr")),
 ("std:crypto","PasswordVerifyOptions","maxMemoryKiB",0,"crypto runtime > catalog-18",include_str!("../fixtures/crypto/catalog-18.tsr")),
 ("std:crypto","PasswordVerifyOptions","maxIterations",0,"crypto runtime > catalog-19",include_str!("../fixtures/crypto/catalog-19.tsr")),
 ("std:crypto","PasswordVerifyOptions","maxParallelism",0,"crypto runtime > catalog-20",include_str!("../fixtures/crypto/catalog-20.tsr")),
 ("Hasher","update","",0,"crypto runtime > catalog-10",include_str!("../fixtures/crypto/catalog-10.tsr")),
 ("Hasher","finalize","",0,"crypto runtime > catalog-10",include_str!("../fixtures/crypto/catalog-10.tsr")),
];
const REGRESSIONS:&[(&str,&str,&str)]=&[
 ("crypto runtime > primitives","fixed digests, streaming, MAC and HKDF\n",include_str!("../fixtures/crypto/primitives.tsr")),
 ("crypto runtime > passwords","explicit bounded password work\n",include_str!("../fixtures/crypto/passwords.tsr")),
 ("crypto runtime > ranges","6\n",include_str!("../fixtures/crypto/ranges.tsr")),
 ("crypto runtime > aliases","true\ntrue\n",include_str!("../fixtures/crypto/aliases.tsr")),
];
fn catalog()->Value{serde_json::from_str(include_str!("../../../tsrust-lsp/src/builtins.json")).unwrap()}
fn example<'a>(catalog:&'a Value,family:&str,name:&str,field:&str,index:usize)->&'a Value{
 let entry=catalog[family].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();
 if field.is_empty(){&entry["examples"][index]}else{&entry["fields"][field]["examples"][index]}
}
pub fn assert_sources(){
 let catalog=catalog();let mut actual=BTreeSet::new();let mut unique=BTreeSet::new();let mut scalar=BTreeSet::new();
 for family in ["std:crypto","Hasher"]{for entry in catalog[family].as_array().unwrap(){let name=entry["name"].as_str().unwrap();
 for(index,_)in entry["examples"].as_array().into_iter().flatten().enumerate(){actual.insert((family,name,"",index));}
 if let Some(fields)=entry["fields"].as_object(){for(field,value)in fields{for(index,_)in value["examples"].as_array().into_iter().flatten().enumerate(){actual.insert((family,name,field.as_str(),index));}}}
 }}
 let mapped=REFERENCES.iter().map(|&(family,name,field,index,_,_)|(family,name,field,index)).collect::<BTreeSet<_>>();assert_eq!(actual,mapped);assert_eq!(actual.len(),29);
 for &(family,name,field,index,case,source)in REFERENCES{let example=example(&catalog,family,name,field,index);let raw=example["source"].as_str().unwrap();let restored=source.replacen("export ","",1);assert_eq!(super::support::format_round_trip(&restored),super::support::format_round_trip(raw),"catalog source differs beyond added export/formatting: {case}");super::support::format_round_trip(source);unique.insert(raw);
 if matches!(name,"PasswordHashOptions"|"PasswordVerifyOptions"){scalar.insert(raw);}}
 assert_eq!(unique.len(),20);assert_eq!(scalar.len(),7);
 for &(case,_,source)in REGRESSIONS{let raw=source.replacen("export ","",1);match case{
 "crypto runtime > primitives"=>assert_eq!(super::support::format_round_trip(&raw),super::support::format_round_trip(include_str!("../../../../examples/crypto/primitives.tsr"))),
 "crypto runtime > passwords"=>assert_eq!(super::support::format_round_trip(&raw),super::support::format_round_trip(include_str!("../../../../examples/crypto/passwords.tsr"))),_=>{}}
 super::support::format_round_trip(source);}
}
pub fn assert_report(output:std::process::Output){
 super::support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));let stdout=String::from_utf8(output.stdout).unwrap();print!("{stdout}");let rows=stdout.lines().map(|line|serde_json::from_str::<Value>(line).unwrap_or_else(|error|panic!("invalid JSON {line}: {error}"))).collect::<Vec<_>>();
 let catalog=catalog();let mut expected=BTreeMap::new();for &(family,name,field,index,case,_)in REFERENCES{let text=example(&catalog,family,name,field,index)["output"].as_str().unwrap();if let Some(previous)=expected.insert(case,text){assert_eq!(previous,text);}}
 for &(case,text,_)in REGRESSIONS{assert!(expected.insert(case,text).is_none());}assert_eq!(expected.len(),24);
 let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),24);let names=tests.iter().map(|row|row["name"].as_str().unwrap()).collect::<BTreeSet<_>>();assert_eq!(names,expected.keys().copied().collect());
 for row in tests{assert_eq!(row["status"],"passed","{row}");let duration=row["durationMs"].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0);}
 for row in rows.iter().filter(|row|row["event"]=="output"){assert_eq!(row["stream"],"stdout","{row}");assert!(expected.contains_key(row["name"].as_str().unwrap()),"unexpected output: {row}");}
 for(case,text)in expected{let mut actual=String::new();for row in rows.iter().filter(|row|row["event"]=="output"&&row["name"]==case){actual.push_str(row["text"].as_str().unwrap());}assert_eq!(actual,text,"{case}");}
 let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);for field in ["durationMs","checkMs","emitMs","backendMs","cacheMs","nativeBuildMs"]{let duration=builds[0][field].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0);}
 let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);assert_eq!(rows.last().unwrap()["event"],"summary");for(field,value)in[("passed",24),("failed",0),("hookFailures",0),("skipped",0),("filesPassed",1),("filesFailed",0)]{assert_eq!(summaries[0][field],value,"{field}");}
}
