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
 // Mixed strings bypass the whole-string ASCII shortcut and straddle both table bounds.
 for code in (0..=0xA0).chain(std::iter::once(0x2FF)) {
  let character=char::from_u32(code).unwrap();
  let text=format!("{character}\u{315}\u{300}");
  let canonical=format!("{character}\u{300}\u{315}");
  let compatibility=if code==0xA0 { " \u{300}\u{315}".to_owned() } else { canonical.clone() };
  assert_eq!(tsr_string_normalize(&text,Some("NFD"),"boundary").as_ref(),canonical);
  assert_eq!(tsr_string_normalize(&text,Some("NFKD"),"boundary").as_ref(),compatibility);
 }
 // Equal combining classes must remain stable even across a long leading-mark run.
 let marks="\u{315}\u{301}\u{300}\u{323}".repeat(1024);
 let ordered=format!("{}{}{}","\u{323}".repeat(1024),"\u{301}\u{300}".repeat(1024),"\u{315}".repeat(1024));
 for form in ["NFC","NFD","NFKC","NFKD"] {
  assert_eq!(tsr_string_normalize(&marks,Some(form),"stable classes").as_ref(),ordered);
 }
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
   for index in 0..5 {
    let result=tsr_string_normalize(&columns[index],Some(form),"conformance");
    assert_eq!(result.as_ref(),columns[expected[index]],"line {} form {} column {}",line_number+1,form,index+1);
    if let String::Owned(output)=result { assert_eq!(output.capacity(),output.len(),"exact output reservation"); }
   }
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

#[test]
fn string_normalize_repeated_mixed_decomposition_sequences_keep_each_form_independent() {
 execute(r#"function main(){
 const original="Åﬁ①각à̕é😀";
 for(let i=1;i<=8;i=i+1){
  const text=original.repeat(i);
  console.log(text.normalize("NFC")==="Åﬁ①각à̕é😀".repeat(i),text.normalize("NFD")==="Åﬁ①각à̕é😀".repeat(i),text.normalize("NFKC")==="Åfi1각à̕é😀".repeat(i),text.normalize("NFKD")==="Åfi1각à̕é😀".repeat(i));
 }
 console.log(original);
 }"#,&format!("{}Åﬁ①각à̕é😀\n","true true true true\n".repeat(8)),None);
}

#[test]
fn string_normalize_compacts_compositions_without_overwriting_unread_scalars() {
 execute(r#"function main(){
 const text="́Ǻ|각|A͂̊".repeat(64);
 const composed="́Ǻ|각|A͂̊".repeat(64);
 console.log(text.normalize("NFC")===composed,text.normalize("NFKC")===composed);
 console.log(composed.normalize("NFD")===text,composed.normalize("NFKD")===text);
 console.log(text.normalize("NFC").normalize("NFC")===composed);
 }"#,"true true\ntrue true\ntrue\n",None);
}

#[test]
fn normalization_pages_match_full_searches_and_identity_needs_no_pending_allocation() {
 let source=support::format_round_trip("function normal(text:string):string{return text.normalize();}function main(){}");
 let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();
 assert!(!emitted.contains("unsafe"));
 let allocator=include_str!("../../../benchmarks/runtime/allocation-driver.rs").split("fn main()").next().unwrap();
 let harness=r#"
fn main(){
 for &(key,expected) in TSR_NORMALIZATION_COMPOSE {
  let left=(key>>21) as u32;let right=(key&u64::from(TSR_NORMALIZATION_SCALAR_MASK)) as u32;
  assert!(right>=0x300,"composition bound U+{right:X}");
  assert_eq!(tsr_normalization_compose(left,right),Some(expected));
 }
 for left in [0,0x3c,0x41,0x1100,0xac00,0x10ffff,u32::MAX] {
  for right in [0,0x7f,0x2ff,0x300] {
   let key=(u64::from(left)<<21)|u64::from(right);
   let expected=TSR_NORMALIZATION_COMPOSE.binary_search_by_key(&key,|row|row.0).ok().map(|index|TSR_NORMALIZATION_COMPOSE[index].1);
   assert_eq!(tsr_normalization_compose(left,right),expected);
  }
 }
 for leading in 0..19 {for vowel in 0..21 {
  let syllable=0xac00+leading*588+vowel*28;
  assert_eq!(tsr_normalization_compose(0x1100+leading,0x1161+vowel),Some(syllable));
  for tail in 1..28 {assert_eq!(tsr_normalization_compose(syllable,0x11a7+tail),Some(syllable+tail));}
 }}
 for code in (0..=0x10ffff).chain([u32::MAX]) {
  let expected=TSR_NORMALIZATION_CCC.binary_search_by_key(&code,|row|row.0).map_or(0,|index|TSR_NORMALIZATION_CCC[index].1);
  assert_eq!(tsr_normalization_class(code),expected,"class U+{code:X}");
  let rows=&TSR_NORMALIZATION_DECOMPOSE[tsr_normalization_page(code,TSR_NORMALIZATION_DECOMPOSE_PAGES)];
  let actual=rows.binary_search_by_key(&code,|row|row.0).ok().map(|index|rows[index]);
  let expected=TSR_NORMALIZATION_DECOMPOSE.binary_search_by_key(&code,|row|row.0).ok().map(|index|TSR_NORMALIZATION_DECOMPOSE[index]);
  assert_eq!(actual,expected,"decomposition U+{code:X}");
 }
 for text in ["😀","Ω","ᄀ","क"] {
  for form in ["NFC","NFD","NFKC","NFKD"] {
   let before=ALLOCS.load(Relaxed);let live=LIVE.load(Relaxed);
   let result=tsr_string_normalize(text,Some(form),"identity");
   assert_eq!(ALLOCS.load(Relaxed)-before,2,"only scalar storage and owned output: {text} {form}");
   assert_eq!(result.as_ref(),text);assert!(matches!(result,String::Owned(_)));
   drop(result);assert_eq!(LIVE.load(Relaxed),live);
  }
 }
}
"#;
 let directory=support::TempDirectory::new();let path=directory.0.join("normalization-pages.rs");
 std::fs::write(&path,emitted.replace("fn main()","fn source_main()")+allocator+harness).unwrap();
 for level in ["0","3"] {
  let executable=directory.0.join(format!("normalization-pages-{level}"));
  support::success(&support::run(support::rustc().args(["--edition=2024","-A","warnings","-D","unused_assignments","-C",&format!("opt-level={level}"),"-C","overflow-checks=on"]).arg(&path).arg("-o").arg(&executable),60));
  let output=support::run(&mut std::process::Command::new(executable),30);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
 }
}

#[test]
fn normalization_expanded_decompositions_match_raw_unicode_mappings() {
 let source=support::format_round_trip("function normal(text:string):string{return text.normalize();}function main(){}");
 let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();assert!(!emitted.contains("unsafe"));
 let allocator=include_str!("../../../benchmarks/runtime/allocation-driver.rs").split("fn main()").next().unwrap();
 let data=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/unicode-normalization/data/UnicodeData.txt");
 let harness=r#"
fn reference_decompose(code:u32,compatibility:bool,mappings:&std::collections::BTreeMap<u32,(bool,Vec<u32>)>,output:&mut Vec<u32>){
 if (0xAC00..0xAC00+11172).contains(&code){
  let syllable=code-0xAC00;output.push(0x1100+syllable/588);output.push(0x1161+syllable%588/28);
  if syllable%28!=0{output.push(0x11A7+syllable%28);}return;
 }
 if let Some((compatible,values))=mappings.get(&code){
  if !*compatible||compatibility{for &value in values{reference_decompose(value,compatibility,mappings,output);}return;}
 }
 output.push(code);
}
fn main(){
 let mut mappings=std::collections::BTreeMap::new();
 for line in include_str!(DATA_PATH).lines(){
  let fields=line.split(';').collect::<Vec<_>>();let code=u32::from_str_radix(fields[0],16).unwrap();
  let mut values=fields[5].split_whitespace().peekable();let compatible=values.peek().is_some_and(|value|value.starts_with('<'));
  if compatible{values.next();}let values=values.map(|value|u32::from_str_radix(value,16).unwrap()).collect::<Vec<_>>();
  if !values.is_empty(){mappings.insert(code,(compatible,values));}
 }
 assert_eq!(mappings.len(),5913);assert_eq!(TSR_NORMALIZATION_COMPATIBILITY.len(),16);
 for &(code,kind,_,_) in TSR_NORMALIZATION_DECOMPOSE{
  if kind>=2{assert_eq!(TSR_NORMALIZATION_COMPATIBILITY[usize::from(kind-2)].0,code);}
 }
 let mut count=0;
 for code in 0..=0x10ffff{
  if char::from_u32(code).is_none(){continue;}
  for compatibility in [false,true]{
   let mut expected=Vec::new();reference_decompose(code,compatibility,&mappings,&mut expected);
   let mut actual=Vec::new();tsr_normalization_decompose(code,compatibility,&mut actual);
   assert_eq!(actual.len(),expected.len(),"U+{code:X} compatibility={compatibility}");
   for (packed,expected) in actual.into_iter().zip(expected){
    assert_eq!(packed&TSR_NORMALIZATION_SCALAR_MASK,expected,"U+{code:X} compatibility={compatibility}");
    assert_eq!(packed>>21,u32::from(tsr_normalization_class(expected)),"class U+{code:X} compatibility={compatibility}");
   }
   count+=1;
  }
 }
 assert_eq!(count,2224128);
 for text in ["Å","ﬁ","\u{385}","\u{1e9b}","\u{fdfa}","각"]{
  for form in ["NFC","NFD","NFKC","NFKD"]{
   let before=ALLOCS.load(Relaxed);let live=LIVE.load(Relaxed);
   let result=tsr_string_normalize(text,Some(form),"expanded mappings");
   assert_eq!(ALLOCS.load(Relaxed)-before,2,"only scalar storage and owned output: {text} {form}");
   assert!(matches!(result,String::Owned(_)));drop(result);assert_eq!(LIVE.load(Relaxed),live);
  }
 }
}
"#.replace("DATA_PATH",&format!("{:?}",data));
 let directory=support::TempDirectory::new();let path=directory.0.join("expanded-normalization.rs");
 std::fs::write(&path,emitted.replace("fn main()","fn source_main()")+allocator+&harness).unwrap();
 for level in ["0","3"]{
  let executable=directory.0.join(format!("expanded-normalization-{level}"));
  support::success(&support::run(support::rustc().args(["--edition=2024","-A","warnings","-D","unused_assignments","-C",&format!("opt-level={level}"),"-C","overflow-checks=on"]).arg(&path).arg("-o").arg(&executable),60));
  let output=support::run(&mut std::process::Command::new(executable),30);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
 }
}

#[test]
fn normalization_reserves_exact_utf8_output_without_growth() {
 let source=support::format_round_trip("function normal(text:string):string{return text.normalize();}function main(){}");
 let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();assert!(!emitted.contains("unsafe"));
 let allocator=include_str!("../../../benchmarks/runtime/allocation-driver.rs").split("fn main()").next().unwrap();
 let harness=r#"
fn main(){
 for code in [0,0x7f,0x80,0x7ff,0x800,0xd7ff,0xe000,0xffff,0x10000,0x10ffff]{
  let text=char::from_u32(code).unwrap().to_string();
  for form in ["NFC","NFD","NFKC","NFKD"]{
   let before=REALLOCS.load(Relaxed);let live=LIVE.load(Relaxed);
   let result=tsr_string_normalize(&text,Some(form),"UTF-8 boundary");
   assert_eq!(REALLOCS.load(Relaxed),before,"no output or scalar growth for U+{code:X}");
   assert_eq!(result.as_ref(),text);
   if let String::Owned(ref output)=result{assert_eq!(output.capacity(),output.len());}else{panic!("normalization must own its output");}
   drop(result);assert_eq!(LIVE.load(Relaxed),live);
  }
 }
 for text in ["".to_owned(),"ascii".to_owned(),"e\u{301}Åﬁ①é😀".repeat(128),"\u{fdfa}".repeat(128),"각".repeat(128),"\u{315}\u{301}\u{300}\u{323}".repeat(1024)]{
  for form in ["NFC","NFD","NFKC","NFKD"]{
   let live=LIVE.load(Relaxed);let result=tsr_string_normalize(&text,Some(form),"output reservation");
   if let String::Owned(ref output)=result{assert_eq!(output.capacity(),output.len());}else{panic!("normalization must own its output");}
   drop(result);assert_eq!(LIVE.load(Relaxed),live);
  }
 }
}
"#;
 let directory=support::TempDirectory::new();let path=directory.0.join("normalization-output.rs");
 std::fs::write(&path,emitted.replace("fn main()","fn source_main()")+allocator+harness).unwrap();
 for level in ["0","3"]{
  let executable=directory.0.join(format!("normalization-output-{level}"));
  support::success(&support::run(support::rustc().args(["--edition=2024","-A","warnings","-C",&format!("opt-level={level}"),"-C","overflow-checks=on"]).arg(&path).arg("-o").arg(&executable),60));
  let output=support::run(&mut std::process::Command::new(executable),30);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
 }
}
