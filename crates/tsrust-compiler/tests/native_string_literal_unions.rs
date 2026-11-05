mod support;
use std::fs;
const SOURCE:&str=include_str!("fixtures/string-literal-unions/observations.tsr");
#[test]
fn literal_union_string_catalog_runs_without_consuming_receivers(){
 let directory=support::TempDirectory::new();let source=directory.source(&support::format_round_trip(SOURCE));
 for release in [false,true]{
  let binary=directory.0.join(format!("observations-{}{}",if release{"release"}else{"debug"},std::env::consts::EXE_SUFFIX));
  let mut command=support::cli();command.arg("build").arg(&source).arg("-o").arg(&binary);if release{command.arg("--release");}
  support::success(&support::run(&mut command,120));let output=support::run(&mut std::process::Command::new(binary),15);support::success(&output);assert_eq!(output.stdout,b"string literal observations accepted\n");
 }
}
#[test]
fn ordinary_and_reexported_literal_aliases_keep_exact_contracts(){
 let directory=support::TempDirectory::new();fs::write(directory.0.join("origins.tsr"),"import {SeekOrigin as NativeOrigin} from 'std:fs';export type Origin=NativeOrigin;").unwrap();
 for prelude in ["import {Origin} from './origins.tsr';","type Origin='start'|'current'|'end';"]{
  let source=format!("{prelude}function observe(value:ReadOnly<Origin>):int{{if(value.startsWith('st')&&value.includes('ta')){{return value.length;}}return value.toUpperCase().length;}}function main(){{const value:Origin='start';console.log(observe(value));console.log(value.length);}}");
  let file=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(&file),30);support::success(&output);
 }
}
#[test]
fn observations_do_not_erase_nullable_mixed_literal_or_readonly_restrictions(){
 let directory=support::TempDirectory::new();
 for source in [
  "import {SeekOrigin} from 'std:fs';function inspect(value:SeekOrigin|undefined){console.log(value.length);}function main(){}",
  "import {SeekOrigin} from 'std:fs';function inspect(value:SeekOrigin|int){console.log(value.startsWith('s'));}function main(){}",
  "import {SeekOrigin} from 'std:fs';function inspect(value:ReadOnly<SeekOrigin>){value.length=1;}function main(){}",
  "import {SeekOrigin} from 'std:fs';function inspect(value:ReadOnly<SeekOrigin>){value[0]='x';}function main(){}",
  "import {SeekOrigin} from 'std:fs';function inspect(value:ReadOnly<SeekOrigin>){const invalid:SeekOrigin=value.toUpperCase();}function main(){}",
  "import {SeekOrigin} from 'std:fs';function main(){const invalid:SeekOrigin='other';}",
  "import {SeekOrigin} from 'std:fs';function inspect(value:ReadOnly<SeekOrigin>):SeekOrigin{return value;}function main(){}",
  "import {SeekOrigin} from 'std:fs';function change(values:Mutable<SeekOrigin[]>):string{values.push('current');return 't';}function main(){const values:SeekOrigin[]=['start'];console.log(values[0].includes(change(values)));}",
 ]{
  let file=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("check").arg(file),30);assert!(!output.status.success(),"accepted {source}");
 }
}
