mod support;
use std::{fs,path::Path};

#[test]
fn reactive_props_and_replacement_composition_execute_after_formatting(){
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-reactive-props");
 for relative in ["src/app/page.tsrx","src/main.tsr","src/inspect.tsr","src/order.tsrx"]{let source=fs::read_to_string(root.join(relative)).unwrap();assert_eq!(source,support::format_round_trip(&source));}
 let result=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),180);support::success(&result);assert_eq!(result.stdout,b"p-2\ndefault\nchild\nReactive props passed\n");
}

#[test]
fn computed_values_reject_wrong_results_containers_and_resource_aliases(){
 let directory=support::TempDirectory::new();
 for (source,expected) in [
  ("import {computed} from 'std:ui';function main(){const value=computed<int>(()=>'wrong');}","type"),
  ("import {computed} from 'std:ui';function main(){const value=computed<int[]>(()=>[1]);}","ui-computed-type"),
  ("import {computed} from 'std:ui';function main(){const value=computed<int>(()=>42);const other=clone(value);}","clone-type"),
  ("import {computed} from 'std:ui';function main(){const value=computed<int>(()=>42);value.set(1);}","Unsupported"),
  ("import {computed} from 'std:ui';function main(){const alias=computed;const value=alias<int>(()=>1);}","ui-state-owner"),
  ("import {View,signal,Node} from 'std:ui';function Child():Node{let value=signal(1);return (<View/>);}function main(){const node=Child();}","ui-state-owner")
 ]{let path=directory.0.join("failure.tsrx");fs::write(&path,source).unwrap();let result=support::run(support::cli().arg("check").arg(path),30);assert!(!result.status.success(),"{source}");assert!(String::from_utf8_lossy(&result.stderr).contains(expected),"{source}\n{}",String::from_utf8_lossy(&result.stderr));}
}

#[test]
fn old_string_props_and_generic_alias_inheritance_stay_compatible(){
 let directory=support::TempDirectory::new();let source="import {View,ViewProps,Node} from 'std:ui';interface Props extends ViewProps{caption:string;}function Panel(props:Props):Node{const classes='p-2 '+(props.className??'');const caption=props.caption;props.caption='';return (<View {...props} className={classes}>{caption}</View>);}function main(){const node=<Panel caption=\"Still valid\"/>;}";let path=directory.0.join("main.tsrx");fs::write(&path,support::format_round_trip(source)).unwrap();support::success(&support::run(support::cli().arg("check").arg(path),30));
}
