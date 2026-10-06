mod support;
use support::{execute,reject};
#[test]
fn string_normalize_canonical_compatibility_hangul_and_combining_order() {
 execute(r#"function main(){
 const text="é Å ﬁ ① 가";
 console.log(text.normalize(),text.normalize("NFC"),text.normalize("NFD"));
 console.log(text.normalize("NFKC"),text.normalize("NFKD"));
 console.log("à̕".normalize("NFD"),"Ǻ".normalize(),"각".normalize());
 console.log("".normalize().length,"é😀".normalize(undefined),text);
 }"#,include_str!("fixtures/string-normalize/basic.stdout"),None);
}
#[test]
fn string_normalize_evaluates_receiver_and_form_once_and_keeps_source() {
 execute(r#"function text(log:int[]):string{log.push(1);return "é";}function form(log:int[]):string{log.push(2);return "NFC";}
 function main(){const log:int[]=[];console.log(text(log).normalize(form(log)),log.join(","));const original="é".repeat(2);const normal=original.normalize();console.log(normal,original);}"#,"é 1,2\néé éé\n",None);
}
#[test]
fn string_normalize_checks_arity_types_and_invalid_form() {
 reject("function main(){\"x\".normalize(\"NFC\",\"NFD\");}","arity");
 reject("function main(){\"x\".normalize(1);}","type");
 reject("function main(){const n=1;n.normalize();}","type");
}

#[test]
fn string_normalize_accepts_optional_forms_and_rejects_invalid_runtime_names() {
 execute(r#"function apply(text:ReadOnly<string>,form?:ReadOnly<string>):string{return text.normalize(form);}function main(){console.log(apply("é"),apply("é","NFD"));}"#, "é é\n",None);
 for form in ["", "nfc", "NFCC"] {
  let source=format!("function main(){{console.log(\"\".normalize({form:?}));}}");
  let analysis=tsrust_compiler::check(&source).unwrap();
  let directory=support::TempDirectory::new();let input=directory.0.join("invalid.rs");let binary=directory.0.join("invalid");
  std::fs::write(&input,analysis.emit_rust()).unwrap();
  support::success(&support::run(support::rustc().arg("--edition=2024").arg(&input).arg("-o").arg(&binary),60));
  let output=support::run(&mut std::process::Command::new(binary),10);assert!(!output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("normalize form must be NFC, NFD, NFKC or NFKD at source bytes"));
 }
}

#[test]
fn string_normalize_passes_official_unicode_conformance_rows() {
 let analysis=tsrust_compiler::check("function main(){console.log(\"x\".normalize());}").unwrap();
 let directory=support::TempDirectory::new();let input=directory.0.join("conformance.rs");let binary=directory.0.join("conformance");
 let data=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/unicode-normalization/data/NormalizationTest.txt").canonicalize().unwrap();
 let harness=r#"
fn main() {
 let data=include_str!(DATA_PATH);
 let mut rows=0;let mut part1=false;let mut listed=std::collections::HashSet::new();
 for (line_number,line) in data.lines().enumerate() {
  let line=line.split('#').next().unwrap().trim();
  if line.is_empty(){continue;}
  if line.starts_with('@'){part1=line.starts_with("@Part1");continue;}
  let columns:Vec<std::string::String>=line.split(';').take(5).map(|field|field.split_whitespace().map(|code|char::from_u32(u32::from_str_radix(code,16).unwrap()).unwrap()).collect()).collect();
  assert_eq!(columns.len(),5);
  if part1 { assert_eq!(columns[0].chars().count(),1); if let Some(character)=columns[0].chars().next() { listed.insert(character as u32); } }
  for (form,expected) in [("NFC",[1,1,1,3,3]),("NFD",[2,2,2,4,4]),("NFKC",[3,3,3,3,3]),("NFKD",[4,4,4,4,4])] {
   for index in 0..5 { assert_eq!(tsr_string_normalize(&columns[index],Some(form),"conformance").as_ref(),columns[expected[index]],"line {} form {} column {}",line_number+1,form,index+1); }
  }
  rows+=1;
 }
 let mut identities=0;
 for &(start,end) in &ASSIGNED_RANGES {
  for code in start..=end {
   if listed.contains(&code){continue;}
   let text=char::from_u32(code).unwrap().to_string();
   for form in ["NFC","NFD","NFKC","NFKD"] { assert_eq!(tsr_string_normalize(&text,Some(form),"identity").as_ref(),text,"identity U+{:X} {}",code,form);identities+=1; }
  }
 }
 assert_eq!(identities,1101784);
 assert_eq!(rows,19965);
 println!("{} rows, {} normalization checks, {} identity checks",rows,rows*20,identities);
}
"#.replace("DATA_PATH",&format!("{:?}",data)).replace("ASSIGNED_RANGES",include_str!("../../../tools/unicode-normalization/assigned-ranges.rs"));
 std::fs::write(&input,analysis.emit_rust().replace("fn main()","fn source_main()")+&harness).unwrap();
 support::success(&support::run(support::rustc().args(["--edition=2024","-O"]).arg(&input).arg("-o").arg(&binary),60));
 let output=support::run(&mut std::process::Command::new(binary),30);support::success(&output);
 assert_eq!(String::from_utf8(output.stdout).unwrap(),"19965 rows, 399300 normalization checks, 1101784 identity checks\n");
}

#[test]
fn string_normalize_handles_owned_and_temporary_optional_form_values() {
 execute(r#"function choose(flag:boolean):string|undefined{return flag?"NFD":undefined;}
function missing(log:int[]):undefined{log.push(2);return undefined;}
function text(log:int[]):string{log.push(1);return "é";}
function main(){const form=choose(true);console.log("é".normalize(form),String(form));
console.log("é".normalize(choose(false)),"é".normalize(choose(true)));
const log:int[]=[];console.log(text(log).normalize(missing(log)),log.join(","));}"#,
 "é NFD\né é\né 1,2\n",None);
}

#[test]
fn string_normalize_emits_pinned_tables_and_license_only_when_used() {
 let plain=tsrust_compiler::check("function main(){console.log(\"x\");}").unwrap().emit_rust();
 assert!(!plain.contains("TSR_NORMALIZATION_CCC"));
 let normalized=tsrust_compiler::check("function main(){console.log(\"x\".normalize());}").unwrap().emit_rust();
 assert!(normalized.contains("TSR_NORMALIZATION_CCC"));
 assert!(normalized.contains("COPYRIGHT AND PERMISSION NOTICE"));
 assert!(normalized.contains("Unicode-3.0"));
}
