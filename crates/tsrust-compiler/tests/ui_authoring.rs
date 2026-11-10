use tsrust_compiler::project::{FileSourceProvider, check_graph, load_project};
mod support;

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
    let source=std::fs::read_to_string(root.join("examples/gpui-starter/src/pages/Table.tsrx")).unwrap();
    let example=path.parent().unwrap().join("example.tsrx");
    std::fs::write(&example,std::fs::read_to_string(root.join("examples/gpui-starter/src/example.tsrx")).unwrap()).unwrap();
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
