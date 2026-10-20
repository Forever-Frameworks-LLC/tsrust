mod support;
#[test]
fn semantic_content_sources_execute_natively_and_offline_install_as_editable_files(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-content");
 let output=support::run(support::cli().arg("run").arg(root.join("inspect.tsrx")),120);support::success(&output);assert_eq!(output.stdout,b"Semantic content passed\n");
 let dir=support::TempDirectory::new();let output=support::run(support::cli().args(["ui","add","kbd","separator","--target","native","--offline"]).current_dir(&dir.0),30);support::success(&output);
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/ui-catalog/semantic-content-1.0.0.json")).unwrap();
 for item in catalog.as_array().unwrap(){let name=item["name"].as_str().unwrap();let source=support::format_round_trip(item["files"][0]["content"].as_str().unwrap());let installed=std::fs::read_to_string(dir.0.join(format!("src/components/ui/{name}.tsrx"))).unwrap();assert!(installed.starts_with("// License: MIT. See ../../../attributions/shadcn-MIT.txt."));let current:serde_json::Value=serde_json::from_str(include_str!("../src/ui-catalog/kbd-2.0.0.json")).unwrap();let current=if name=="kbd"{support::format_round_trip(current[0]["files"][0]["content"].as_str().unwrap())}else{source.clone()};assert_eq!(installed.split_once("import ").unwrap().1,current.split_once("import ").unwrap().1);assert_eq!(std::fs::read_to_string(root.join(format!("src/components/ui/{name}.tsrx"))).unwrap(),source);}
 let lock=std::fs::read(dir.0.join("ui-components.lock.json")).unwrap();let output=support::run(support::cli().args(["ui","add","kbd","separator"]).current_dir(&dir.0),30);support::success(&output);assert_eq!(std::fs::read(dir.0.join("ui-components.lock.json")).unwrap(),lock);
}
#[test]
fn separator_children_unknown_orientation_and_unsupported_props_reject(){
 let dir=support::TempDirectory::new();let output=support::run(support::cli().args(["ui","add","kbd","separator","--target","native"]).current_dir(&dir.0),30);support::success(&output);
 for (body,reason) in [("const n=<Separator orientation='diagonal'/>;","Expected"),("const n=<Separator>Invalid</Separator>;","children"),("const n=<Kbd render={()=>{return <View/>;}}>Invalid</Kbd>;","render")] {
  let source=format!("import {{Separator}} from './components/ui/separator';import {{Kbd}} from './components/ui/kbd';import {{View}} from 'std:ui';function main(){{{body}}}");let path=dir.0.join("src/reject.tsrx");std::fs::write(&path,support::format_round_trip(&source)).unwrap();for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains(reason),"{}",String::from_utf8_lossy(&output.stderr));}
 }
}

#[test]
fn current_keycaps_preserve_semantics_and_owned_bindings(){
 let dir=support::TempDirectory::new();support::success(&support::run(support::cli().args(["ui","add","kbd","--target","native","--target","web","--offline"]).current_dir(&dir.0),30));
 let source=r#"import {View,Button,Svg,Path,document} from 'std:ui';
import {Kbd,KbdGroup} from './src/components/ui/kbd';
import {trackedResource,resourceId,releaseCount,strictEqual} from 'std:testing';
function main(){
 const owner=trackedResource();const id=resourceId(owner);
 {const node=<Kbd id="owned-key" data-slot="custom-key" role="group" aria-label="Command" className={{evaluate:()=>{strictEqual(releaseCount(resourceId(owner)),0);return 'h-8 px-3';}}}><Svg fill="none" stroke="currentColor"><Path d="M4 12H20"/></Svg>K</Kbd>;
 const html=document(node,'Key');strictEqual(html.includes('data-slot="custom-key"'),true,'caller slot');strictEqual(html.includes('aria-label="Command"'),true,'accessible group name');strictEqual(html.includes('height:32px'),true,'explicit height');strictEqual(html.includes('viewBox="0 0 24 24"'),true,'SVG child');strictEqual(releaseCount(id),0);}
 strictEqual(releaseCount(id),1);
 const nested=document(<Button><KbdGroup><Kbd>Ctrl</Kbd>+<Kbd>K</Kbd></KbdGroup></Button>,'Shortcut');strictEqual(nested.includes('data-slot="kbd-group"'),true);strictEqual(nested.includes('<kbd'),true);
 console.log('Current keycaps passed');
}"#;
 let path=dir.0.join("inspect.tsrx");std::fs::write(&path,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("run").arg(&path),180);support::success(&output);assert_eq!(output.stdout,b"Current keycaps passed\n");
}

#[test]
fn keyboard_reference_keeps_nested_markdown_and_paired_examples_executable(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
 for relative in ["src/acceptance/keyboard-documentation.tsr","src/examples/keyboard-shortcuts.tsrx","src/components/ui/kbd-v2.tsrx","src/components/docs/markdown-content.tsrx"]{let source=std::fs::read_to_string(root.join(relative)).unwrap();assert_eq!(source,support::format_round_trip(&source));}
 let output=support::run(support::cli().arg("run").arg(root.join("src/acceptance/keyboard-documentation.tsr")),180);support::success(&output);assert_eq!(output.stdout,b"Keyboard documentation passed\n");
}

#[test]
fn current_table_exports_keep_spans_scope_and_owned_children(){
 let dir=support::TempDirectory::new();support::success(&support::run(support::cli().args(["ui","add","table","--target","native","--target","web","--offline"]).current_dir(&dir.0),30));
 let source=r#"import {document,View,Text} from 'std:ui';
import {Table,TableHeader,TableBody,TableFooter,TableRow,TableHead,TableCell,TableCaption} from './src/components/ui/table';
import {trackedResource,resourceId,releaseCount,strictEqual} from 'std:testing';
function main(){
 const owner=trackedResource();const id=resourceId(owner);
 {const table=<Table id="owned-table" className={{evaluate:()=>{strictEqual(releaseCount(resourceId(owner)),0);return 'text-base';}}}><TableCaption>Invoices</TableCaption><TableHeader><TableRow><TableHead scope="col">Invoice</TableHead><TableHead scope="col">Amount</TableHead></TableRow></TableHeader><TableBody><TableRow data-state="selected"><TableHead scope="row">One</TableHead><TableCell>$10</TableCell></TableRow></TableBody><TableFooter><TableRow><TableCell colSpan={2}>Total $10</TableCell></TableRow></TableFooter></Table>;
 const html=document(table,'Table');for(const text of ['<table','<thead','<tbody','<tfoot','<caption','colspan="2"','scope="row"','data-state="selected"','data-slot="table-container"']){strictEqual(html.includes(text),true,text);}strictEqual(releaseCount(id),0);}
 strictEqual(releaseCount(id),1);
 let rejected=false;try{const bad=<TableCell colSpan={0}>Bad</TableCell>;}catch{rejected=true;}strictEqual(rejected,true);
 let nested=false;try{const bad=<Table><TableBody><TableRow><TableCell><Table/></TableCell></TableRow></TableBody></Table>;}catch{nested=true;}strictEqual(nested,true);
 console.log('Current table passed');
}"#;
 let path=dir.0.join("inspect.tsrx");std::fs::write(&path,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("run").arg(&path),180);support::success(&output);assert_eq!(output.stdout,b"Current table passed\n");
 let installed=std::fs::read_to_string(dir.0.join("src/components/ui/table.tsrx")).unwrap();for export in ["TableHeaderProps","TableBodyProps","TableFooterProps","TableRowProps","TableHeadProps","TableCellProps","TableCaptionProps"]{assert!(installed.contains(export));}assert!(installed.starts_with("// License: MIT. See "));
}

#[test]
fn table_documentation_keeps_formatted_paired_examples_executable(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
 for relative in ["src/acceptance/table-documentation.tsr","src/examples/table-layouts.tsrx","src/components/ui/table.tsrx"]{let source=std::fs::read_to_string(root.join(relative)).unwrap();assert_eq!(source,support::format_round_trip(&source));}
 let output=support::run(support::cli().arg("run").arg(root.join("src/acceptance/table-documentation.tsr")),180);support::success(&output);assert_eq!(output.stdout,b"Table documentation passed\n");
}
