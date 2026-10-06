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
