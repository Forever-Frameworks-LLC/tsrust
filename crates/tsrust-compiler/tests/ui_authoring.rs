use tsrust_compiler::project::{FileSourceProvider, check_graph, load_project};
mod support;

#[test]
fn terminal_facade_has_checked_commands_and_opt_in_native_dependencies() {
 let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
 for (body,selected) in [("const view=<Terminal id='shell' directory='/tmp' fontSize={14}/>;controlTerminal('shell',{action:'interrupt'},(result)=>{console.log(result.ok);});readTerminal('shell',(snapshot)=>{console.log(snapshot.found);});",true),("controlTerminal('absent',{action:'focus'},(result)=>{console.log(result.ok);});",false)] {
  std::fs::write(&path,support::format_round_trip(&format!("import {{Terminal,controlTerminal,readTerminal}} from 'std:ui';function main(){{{body}}}"))).unwrap();
  let rust=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap().emit_rust();
  assert_eq!(tsrust_compiler::native_runtime::separate(&rust).runtime.unwrap_or_default().contains("// tsrust-ui-terminal-selected"),selected);
 }
 std::fs::write(&path,"import {controlTerminal} from 'std:ui';function main(){controlTerminal('shell',{action:'format-disk'},(result)=>{});}").unwrap();
 assert!(check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).is_err());
}

#[test]
fn data_signals_accept_snapshots_but_reject_nested_resources_and_callbacks() {
 let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
 let check=|source:&str| {std::fs::write(&path,source).unwrap();check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap())};
 check("import {signal,Signal} from 'std:ui';interface Row{text:string;next?:Row;}function main(){const rows:Row[]=[{text:'one'}];const state=signal(rows);const next=state.get();next[0].text='two';state.set(next);const maybe:string|null=null;const optional=signal(maybe);optional.set('ready');}").unwrap().emit_rust();
 for value in ["<View/>","inner","{nested:[<View/>]}","task"] {
  let source=format!("import {{signal,View}} from 'std:ui';interface HandlerData{{run:Owned<Function<int>>;}}function main(){{const inner=signal(1);const task:HandlerData={{run:()=>1}};const state=signal({value});}}");
  let error=check(&source).err().expect("state cannot retain resource owners or callbacks");
  assert_eq!(error.code,"ui-state-type","{source}: {}",error.message);
 }
}

#[test]
fn known_control_nesting_rejects_before_routes_run_and_respects_component_identity() {
    let directory=tempfile::tempdir().unwrap();
    let path=directory.path().join("main.tsrx");
    let check=|source:&str| {
        std::fs::write(&path,source).unwrap();
        check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap())
    };
    for child in ["<Kbd>secondary-k</Kbd>","<Button>Bad</Button>","<Input />","<View />"] {
        let source=format!("import {{Node,Text as Paragraph,Button,Input,View,Kbd,Route}} from 'std:ui';function Page():Node{{return <Paragraph>{child}</Paragraph>;}}function main(){{const page=<Route path=\"/never\" render={{()=>Page()}}/>;}}");
        let error=check(&source).err().expect("invalid nesting must reject");
        assert_eq!(error.code,"ui-children","{}",error.message);
        assert!(error.message.contains("child of Text"),"{}",error.message);
        assert_eq!(&source[error.location.span.start..error.location.span.end],child);
    }
    for child in ["<Button>Bad</Button>","<Link href=\"/\">Bad</Link>","<Input />"] {
        let source=format!("import {{Button,Link,Input}} from 'std:ui';function main(){{const button=<Button>{child}</Button>;}}");
        assert_eq!(check(&source).err().unwrap().code,"ui-children");
    }
    check("import {Button,Text,Link} from 'std:ui';function main(){const button=<Button><Text>Valid</Text></Button>;const paragraph=<Text><Link href=\"/\">Valid</Link></Text>;}").unwrap();
    check("import {Node,View} from 'std:ui';interface Props{children?:Node[];}function Text(props:Props):Node{return <View children={props.children??[]}/>;}function main(){const custom=<Text><View/></Text>;}").unwrap();
}

#[test]
fn signal_parameters_keep_reactive_reads_when_a_component_receives_shared_state() {
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
    std::fs::write(&path,"import {Signal as Shared,Node,Text,signal} from 'std:ui';function Display(value:Shared<int>):Node{return <Text>Count: {value.get()+1}</Text>;}function main(){const value=signal(2);const node=Display(value);}").unwrap();
    let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();
    assert!(checked.emit_rust().contains("TextBinding::computed"),"received signals need a retained getter, not a one-time scalar snapshot");
}

#[test]
fn table_ownership_errors_do_not_replay_every_bundled_function_or_hide_other_errors() {
    let directory=support::TempDirectory::new();
    let path=directory.0.join("main.tsrx");
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let source=std::fs::read_to_string(root.join("apps/gpui-starter/src/pages/Table.tsrx")).unwrap();
    let example=path.parent().unwrap().join("example.tsrx");
    std::fs::write(&example,std::fs::read_to_string(root.join("apps/gpui-starter/src/example.tsrx")).unwrap()).unwrap();
    let source=source.replace("'../example.tsrx'","'./example.tsrx'").replace("clone(person.name)","person.name").replace("clone(person.email)","person.email");
    let source=format!("{source}\nfunction independent():boolean{{return 42;}}function main(){{const page=TablePage();}}\n");
    std::fs::write(&path,&source).unwrap();
    let output=support::run(support::cli().arg("check").arg(&path).arg("--json"),20);
    assert!(!output.status.success());
    let result:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics=result["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|error|error["message"].as_str().is_some_and(|message|message.contains("boolean"))),"{result}");
    assert!(diagnostics.iter().any(|error|error["location"]["start"].as_u64().is_some_and(|start|start < source.find("function independent").unwrap() as u64)),"the original table ownership error must remain visible: {result}");
}

#[test]
fn live_array_props_allow_scalar_arrays_without_weakening_record_or_computed_ownership() {
 let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
 let check=|source:&str| {std::fs::write(&path,source).unwrap();check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap())};
 let checked=check("import {Node,Plot,PlotLine,signal} from 'std:ui';function Chart():Node{const count=signal(1);return <Plot labels={['A',`${count.get()}`]}><PlotLine values={[Number(count.get()),2]}/></Plot>;}function main(){const node=Chart();}").unwrap();
 assert!(!checked.emit_rust().is_empty());
 check("interface Other{value:int;}function accept(values:number[]|Other){}function main(){accept([1,2]);accept([]);accept(Array.of());accept(Array.of(3,4));const values:number[]=[1,2];accept([...values]);}").unwrap();
 assert!(check("interface Other{value:int;}function accept(values:number[]|Other){}function main(){const values:int[]=[1,2];accept(values);}").is_err());
 let error=check("import {Node,View,ReactiveValue,signal} from 'std:ui';interface Row{value:int;}interface Props{rows:ReactiveValue<Row[]>;}function Custom(props:Props):Node{return <View/>;}function main(){const count=signal(1);const node=<Custom rows={[{value:count.get()}]}/>;}").err().unwrap();
 assert_eq!(error.code,"ui-reactive-prop","{}",error.message);
 let error=check("import {computed,Computed} from 'std:ui';function main(){const values:Computed<int[]> = computed(()=>[1,2]);}").err().unwrap();
 assert_eq!(error.code,"ui-computed-type","{}",error.message);
}

#[test]
fn diff_selects_highlighting_only_when_used_and_input_predicates_keep_typed_returns() {
 let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
 let check=|source:&str| {std::fs::write(&path,source).unwrap();check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap())};
 for (element,highlight) in [("<Diff patch=\"\"/>",true),("<View/>",false)] {
  let source=format!("import {{Diff,View}} from 'std:ui';function main(){{const node={element};}}");
  let rust=check(&source).unwrap().emit_rust();let units=tsrust_compiler::native_runtime::separate(&rust);
  assert_eq!(units.runtime.unwrap_or_default().contains("// tsrust-ui-highlight-selected"),highlight);
 }
 check("import {Input,NumberInput} from 'std:ui';function main(){const field=<Input onValidate={(value)=>value.length<4} onPaste={(value)=>false}/>;const number=<NumberInput stepBy={(value,increment)=>increment?0.1:0.5}/>;}").unwrap();
 assert!(check("import {Input} from 'std:ui';function main(){const field=<Input onValidate={(value)=>value}/>;}").is_err());
 assert!(check("import {NumberInput} from 'std:ui';function main(){const field=<NumberInput stepBy={(value,increment)=>true}/>;}").is_err());
}

#[test]
fn malformed_nested_tsrx_reports_diagnostics_without_panicking_after_unicode_text() {
    let directory=support::TempDirectory::new();
    let path=directory.0.join("main.tsrx");
    for markup in ["<Text><Text>café</Text>.{1 + }</Text>","<Text><Text>café</Text>.<Other/></Text>","<Text><Text>café</Text>.</Wrong>"] {
        let source=format!("interface Props{{children:string[];}}function Text(props:Props):string{{return props.children.join('');}}function main(){{const text={markup};}}");
        let source=support::format_round_trip(&source);
        std::fs::write(&path,&source).unwrap();
        let errors=tsrust_compiler::project::check_with_diagnostics(&FileSourceProvider,path.to_str().unwrap()).err().expect("malformed or unresolved markup must reject");
        assert!(errors.iter().any(|error|if markup.contains("Other") {error.message.contains("Other")}else{error.code=="syntax"}),"{errors:?}");
        for error in errors {assert!(source.is_char_boundary(error.location.span.start));assert!(source.is_char_boundary(error.location.span.end));}
    }
}

#[test]
fn dock_selection_and_close_callbacks_keep_borrowed_string_contracts() {
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
    let check=|source:&str| {
        let formatted=support::format_round_trip(source);
        std::fs::write(&path,formatted).unwrap();
        check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap())
    };
    check("import {Node,Dock,DockPanel,Text,signal} from 'std:ui';function main(){const selected=signal('a');const dirty=signal(false);const node=<Dock tabbed={true} value={selected.get()} onChange={(value)=>{selected.set(clone(value));}} onClose={(value)=>{console.log(value);}}><DockPanel value='a' title='First' closable={!dirty.get()} padded={false}><Text>Editor</Text></DockPanel></Dock>;}").unwrap().emit_rust();
    check("import {Dock,DockPanel,signal} from 'std:ui';function main(){const saved=signal('');const node=<Dock initialLayout={saved.get()} onLayoutChange={(layout)=>{saved.set(clone(layout));}} onLayoutError={(error)=>{console.log(error);}}><DockPanel value='stable'/></Dock>;}").unwrap().emit_rust();
    for property in ["onChange", "onClose", "onLayoutChange", "onLayoutError"] {
        let source=format!("import {{Dock}} from 'std:ui';function main(){{const node=<Dock {property}={{(value:int)=>{{console.log(value);}}}}/>;}}");
        assert!(check(&source).is_err(),"{property} accepted an integer callback instead of borrowed text");
    }
}


#[test]
fn resizable_reverse_accepts_live_boolean_ordering_only() {
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
    for (value,valid) in [("reverse.get()",true),("123",false)] {
        let source=format!("import {{Resizable,ResizablePanel,Text,signal}} from 'std:ui';function main(){{const reverse=signal(false);const view=<Resizable reverse={{{value}}}><ResizablePanel><Text>Main</Text></ResizablePanel><ResizablePanel><Text>Tools</Text></ResizablePanel></Resizable>;}}");
        std::fs::write(&path,support::format_round_trip(&source)).unwrap();
        let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap());
        assert_eq!(checked.is_ok(),valid,"{value}: {:?}",checked.err());
    }
}


#[test]
fn localized_tooltips_and_placeholders_accept_live_text_only() {
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
    for component in ["Button","Input","Textarea"] {
        let property=if component=="Button"{"tooltip"}else{"placeholder"};
        for (value,valid) in [("busy.get() ? 'Saving' : 'Save'",true),("t('Save')",true),("123",false)] {
            let markup=if component=="Button"{format!("<Button {property}={{{value}}}>Save</Button>")}else{format!("<{component} {property}={{{value}}}/>")};
            let source=format!("import {{Button,Input,Textarea,signal,t}} from 'std:ui';function main(){{const busy=signal(false);const field={markup};}}");
            std::fs::write(&path,support::format_round_trip(&source)).unwrap();
            let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap());
            assert_eq!(checked.is_ok(),valid,"{component} {value}: {:?}",checked.err());
        }
    }
}


#[test]
fn asynchronous_native_ui_keeps_the_process_main_thread() {
    // Inspects value-mode entry text; adapted when unwinding faults become the
    // default (docs/plans/fault-unwinding.md, 5.5).
    if std::env::var("TSRUST_FAULT_MODE").as_deref() == Ok("unwind") { return; }
    let directory=tempfile::tempdir().unwrap();
    let path=directory.path().join("main.tsrx");
    std::fs::write(&path,"import {View,run} from 'std:ui';import {delay} from 'std:futures';async function main():Task<void>{await delay(0);unwrap(run(<View/>, 'Async UI'));}").unwrap();
    let rust=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap().emit_rust();
    assert!(rust.contains("tsrust_async_block_on(async move"));
    assert!(rust.contains("fn main()->std::process::ExitCode{ tsrust_entry_main() }"));
    assert!(!rust.contains("tsrust_main_thread(tsrust_entry_main)"));
    let cli=tsrust_compiler::check("async function main():Task<void>{}").unwrap().emit_rust();
    assert!(cli.contains("tsrust_main_thread(tsrust_entry_main)"));
}

#[test]
fn paginated_tables_accept_rich_cells_manual_sorting_and_scroll_resets() {
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("main.tsrx");
    let source="import {DataTable,DataTableColumn,DataTableCell,Tag,List,ListItem,signal} from 'std:ui';function main(){const page=signal(1);const table=<DataTable rows='[[\"Done\"]]' manualSorting={true} scrollResetKey={page.get().toString()} onSort={(sort)=>{console.log(sort[0],sort[1]);}}><DataTableColumn key='0' sortable={true} fill={true} defaultSort='ascending' sort='none'>Status</DataTableColumn><DataTableCell row={0} column='0'><Tag variant='success'>Done</Tag></DataTableCell></DataTable>;const cards=<List selectable={false} minItemWidth={340} scrollResetKey={page.get().toString()}><ListItem value='one'>One</ListItem></List>;}";
    std::fs::write(&path,support::format_round_trip(source)).unwrap();
    let rust=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap().emit_rust();
    assert!(rust.contains("DataTableCell"));
    std::fs::write(&path,source.replace("sort='none'","sort='sideways'")).unwrap();
    assert!(check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).is_err(),"controlled sorting must retain its direction contract");
    std::fs::write(&path,source.replace("row={0}","row={'wrong'}")).unwrap();
    assert!(check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).is_err());
}
