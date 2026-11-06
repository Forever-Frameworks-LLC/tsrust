mod support;
use support::session::Session;
use serde_json::json;
use tsrust_lsp::project_diagnostics::source_id;

fn agree(session:&mut Session,source:&str,expected:Option<&str>,broken:bool){
    let diagnostics=session.wait_for_diagnostics("main.tsr");
    let items=diagnostics.as_array().expect("diagnostic array");
    if !broken && let Some(code)=expected{assert!(items.iter().any(|item|item["code"]==code),"expected {code}: {diagnostics}");}
    else if !broken{assert!(items.is_empty(),"{diagnostics}");}
    if broken{assert!(!items.is_empty(),"broken document must be diagnosed");}
    // didChange is an editor overlay: write the identical version for the CLI.
    let path=source_id(&session.uri("main.tsr")).unwrap().0;
    std::fs::write(&path,source).unwrap();
    let cli=std::env::var_os("TSRUST").map(std::path::PathBuf::from).unwrap_or_else(||std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join(format!("tsrust{}",std::env::consts::EXE_SUFFIX)));
    let output=std::process::Command::new(cli).args(["check",&path,"--color=never"]).output().unwrap();
    assert_eq!(output.status.success(),expected.is_none()&&!broken,"{}",String::from_utf8_lossy(&output.stderr));
    if !broken && let Some(code)=expected{assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")),"{}",String::from_utf8_lossy(&output.stderr));}
}

fn phases(raw:&str,expected:Option<&str>){
    let source=tsrust_compiler::format(raw).unwrap();
    let broken=format!("{source}\nfunction broken(){{const = ;}}\n");
    let mut session=Session::start(&[]);
    session.open("main.tsr",&source);agree(&mut session,&source,expected,false);
    session.change("main.tsr",&broken);agree(&mut session,&broken,expected,true);
    session.change("main.tsr",&source);agree(&mut session,&source,expected,false);
    session.notify("textDocument/didClose",json!({"textDocument":{"uri":session.uri("main.tsr")}}));
    session.open("main.tsr",&source);agree(&mut session,&source,expected,false);
}

#[test]
fn send_error_clone_capabilities_match_cli_through_edit_recovery(){
    for module in ["std:futures","std:threads"]{
        phases(&format!("import {{SendError}} from '{module}';function inspect(error:ReadOnly<SendError<int>>){{const copied=clone(error);console.log(copied.value);}}function main(){{}}"),None);
        phases(&format!("import {{SendError}} from '{module}';type Data={{items:string[];}};type Wrapped={{error:SendError<Data>;}};function inspect(value:ReadOnly<Wrapped>){{const copied=clone(value);console.log(copied.error.value.items.length);}}function main(){{}}"),None);
    }
    for (payload,shape) in [("TrackedResource","{error:SendError<TrackedResource>;}"),("CallablePayload","AsyncError<SendError<CallablePayload>>")]{
        phases(&format!("import {{SendError,AsyncError}} from 'std:futures';import {{TrackedResource}} from 'std:testing';type CallablePayload={{call:Function<int>;}};type Wrapped={shape};function inspect(error:ReadOnly<Wrapped>){{const copied=clone(error);}}function main(){{}}// {payload}"),Some("clone-type"));
    }
}

#[test]
fn indexed_callback_storage_and_native_view_restrictions_match_cli_after_recovery(){
    phases("type Holder={run:ReadOnly<Function<int>>;};type Box={items:Holder[];};function main(){const first=[3];const second=[7];{const boxes:Box[][]=[[{items:[{run:():int=>first[0]}]}]];boxes[0][0].items.push({run:():int=>second[0]});const removed=boxes[0][0].items.pop();if(removed!==undefined)console.log(removed.run());console.log(boxes[0][0].items[0].run());}first[0]=11;second[0]=13;}",None);
    phases("type Holder={run:ReadOnly<Function<int>>;};function main(){const first=[3];const second=[7];const groups:Holder[][]=[[{run:():int=>first[0]}]];groups[0].push({run:():int=>second[0]});second[0]=9;console.log(groups[0][1].run());}",Some("capture-conflict"));
    phases("import {ByteView,fromUtf8,view} from 'std:bytes';function main(){const bytes=unwrap(fromUtf8('abc'));const boxes:ByteView[][]=[[]];boxes[0].push(unwrap(view(bytes,0,1)));console.log(boxes[0].length);}",Some("native-view-source"));
    phases("import {ByteView,fromUtf8,view} from 'std:bytes';type Holder={run:ReadOnly<Function<int>>;part:ByteView;};type Box={items:Holder[];};function main(){const values=[3];const bytes=unwrap(fromUtf8('abc'));const boxes:Box[]=[{items:[]}];boxes[0].items.push({run:():int=>values[0],part:unwrap(view(bytes,0,1))});console.log(boxes[0].items.length);}",Some("native-view-source"));
}

#[test]
fn both_send_error_exports_explain_conditional_clone_after_edit_recovery(){
    for module in ["std:futures","std:threads"]{
        let source=tsrust_compiler::format(&format!("import {{SendError}} from '{module}';function inspect(error:ReadOnly<SendError<int>>){{console.log(error.code);}}function main(){{}}")).unwrap();
        let offset=source.rfind("SendError<int>").unwrap()+1;
        let broken=format!("{source}\nfunction broken(){{const = ;}}\n");
        let mut session=Session::start(&[]);let mut baseline=None;
        for (phase,text) in [("clean",&source),("broken",&broken),("restored",&source),("reopened",&source)]{
            if phase=="clean"{session.open("main.tsr",text);}else if phase=="reopened"{session.notify("textDocument/didClose",json!({"textDocument":{"uri":session.uri("main.tsr")}}));session.open("main.tsr",text);}else{session.change("main.tsr",text);}
            let diagnostics=session.wait_for_diagnostics("main.tsr");assert_eq!(diagnostics.as_array().unwrap().is_empty(),phase!="broken","{phase}: {diagnostics}");
            let hover=session.hover("main.tsr",text,offset).expect("SendError hover");
            for part in ["SendError","explicit clone independently copies","payload is cloneable","resources and callable environments remain non-cloneable"]{assert!(hover.contains(part),"{module}/{phase}: {hover}");}
            if let Some(prior)=&baseline{assert_eq!(&hover,prior,"{module}/{phase}");}else{baseline=Some(hover);}
        }
    }
}
