mod support;
#[test]
fn string_iteration_reads_unicode_characters_and_preserves_control_flow(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/string-iteration/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn string_iteration_rejects_source_replacement_and_character_transfers(){
 for source in [
  "function main(){let text=String('abc');for(const character of text){text='changed';}}",
  "function main(){const output:string[]=[];for(const character of 'abc'){output.push(character);}}",
  "function take(text:Owned<string>):void{}function main(){for(const character of 'abc'){take(character);}}",
  "function main(){for(let character of 'abc'){character='changed';}}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe string loop accepted: {source}");}
}

#[test]
fn string_iteration_checks_callable_results(){
 tsrust_compiler::check("function main(){const count=(text:string):int=>{let size=0;for(const character of text){size+=1;}return size;};const size:int=count('abc');}").unwrap();
}

fn native_character_views(source:&str,extra:&str,expected:&str)->String {
 let analysis=tsrust_compiler::check(source).unwrap_or_else(|error|panic!("{}",error.render("character-views.tsr",source)));
 let emitted=analysis.emit_rust();assert!(!emitted.contains("unsafe"));
 let directory=support::TempDirectory::new();let path=directory.0.join("character-views.rs");
 std::fs::write(&path,emitted.replace("fn main()","fn source_main()")+extra).unwrap();
 for level in ["0","3"] {
  let binary=directory.0.join(format!("character-views-{level}"));
  support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}"),"-C","overflow-checks=on"]).arg(&path).arg("-o").arg(&binary),60));
  let result=support::run(&mut std::process::Command::new(binary),10);support::success(&result);assert_eq!(result.stdout,expected.as_bytes());assert!(result.stderr.is_empty());
 }
 emitted
}

#[test]
fn readonly_scalar_views_allocate_nothing_and_keep_utf8_byte_lengths_and_ordering(){
 let emitted=native_character_views(r#"function inspect(text:string):int{let total=0;for(const character of text){total+=character.length;if(character!=="é"){total+=1;}if(character<"z"){total+=2;}if(character>="é"){total+=4;}if(character==="🙂"){total+=8;}}return total;}function main(){}"#,
 r#"
 use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicUsize,Ordering};
 struct Observer;static CALLS:AtomicUsize=AtomicUsize::new(0);
 unsafe impl GlobalAlloc for Observer {
 unsafe fn alloc(&self,l:Layout)->*mut u8{CALLS.fetch_add(1,Ordering::SeqCst);unsafe{System.alloc(l)}}
 unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)}}
 unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{CALLS.fetch_add(1,Ordering::SeqCst);unsafe{System.realloc(p,l,n)}}
 }#[global_allocator]static ALLOCATOR:Observer=Observer;
 fn main(){source_main();for text in ["","abc","Aé🙂e\u{301}","\0\u{10ffff}"] {
 let input=String::Owned(text.to_owned());let expected=text.chars().map(|c|c.len_utf8() as i64+if c!='é'{1}else{0}+if c<'z'{2}else{0}+if c>='é'{4}else{0}+if c=='🙂'{8}else{0}).sum::<i64>();
 CALLS.store(0,Ordering::SeqCst);let actual=function0(std::hint::black_box(&input));assert_eq!(CALLS.load(Ordering::SeqCst),0);assert_eq!(actual,expected);}}
 "#,"");
 assert!(emitted.contains("let mut string_bytes_"));assert!(emitted.contains("&std::borrow::Cow<'_, str>"));
}

#[test]
fn scalar_view_concatenations_nesting_and_cleanup_preserve_owned_outputs(){
 let emitted=native_character_views(r#"
 function build(text:string):string{let output="";for(const outer of text){for(const inner of "ab"){output=output+outer+inner;}output=output+outer;}return output;}
 function stop(text:string):string{let output="";let done=0;for(const character of text){try{if(character==="a"){continue;}output=output+character;if(character==="🙂"){break;}}finally{done+=1;}}return output+":"+String(done);}
 function main(){console.log(build("é🙂"));console.log(stop("aé🙂z"));}
 "#,r#"fn main(){source_main();let input=String::Owned("é🙂".to_owned());let result=function0(&input);drop(input);assert_eq!(result.as_ref(),"éaébé🙂a🙂b🙂");}"#,"éaébé🙂a🙂b🙂\né🙂:3\n");
 assert!(emitted.contains("let mut string_bytes_"));
}

#[test]
fn scalar_copy_calls_captures_and_unproved_additions_keep_owning_fallback(){
 let emitted=native_character_views(r#"
 function bytes(text:string):int{return text.length;}
 function copy(text:string):string[]{const output:string[]=[];for(const character of text){output.push(clone(character));}return output;}
 function main(){let text=String("é🙂");const saved=copy(text);text="changed";console.log(saved.join(""));for(const character of "é"){console.log(bytes(character));}for(const character of "🙂"){console.log(character+"!");}for(const character of "é"){const read=():int=>character.length;console.log(read());}}
 "#,r#"fn main(){source_main();}"#,"é🙂\n2\n🙂!\n2\n");
 assert!(emitted.contains("=&tsr_char_text("));assert!(!emitted.contains("let mut string_bytes_"));
}

#[test]
fn character_view_proof_budget_falls_back_without_rejecting_source(){
 let source=format!("function inspect(text:string):int{{let count=0;for(const character of text){{count+=character.length;}}{}return count;}}function main(){{}}","count+=1;".repeat(10_001));
 let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();assert!(!emitted.contains("let mut string_bytes_"));assert!(emitted.contains("=&tsr_char_text("));
}
