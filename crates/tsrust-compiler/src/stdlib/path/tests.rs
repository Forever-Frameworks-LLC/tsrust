use super::{runtime,relative};
#[test]
fn parsed_components_and_exact_capacity_format_follow_the_fixed_corpus() {
    let mut count=0;
    for line in include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../benchmarks/runtime/admission/path-env-os/corpus.tsv")).lines().filter(|line|!line.is_empty()&&!line.starts_with('#')) {
        let row=line.split('\t').collect::<Vec<_>>();let windows=row[1]=="win32";
        if row[0]=="parse" {
            let actual=runtime::parse_parts(row[2],windows);
            assert_eq!([actual.root.as_str(),actual.dir.as_str(),actual.base.as_str(),actual.ext.as_str(),actual.name.as_str()],row[3..8],"{line}");count+=1;
        }else if row[0]=="format" {
            assert_eq!(runtime::format_parts(row[3],row[4],row[5],row[6],row[7],windows),row[2],"{line}");count+=1;
        }
    }
    assert_eq!(count,25);
    for path in ["\\\\server\\share.txt","\\\\server\\share.txt\\","/","C:"] {
        let parts=runtime::parse_parts(path,true);assert!(parts.base.is_empty()&&parts.ext.is_empty()&&parts.name.is_empty(),"{parts:?}");
    }
    assert_eq!(runtime::format_parts("/ignored","/dir","base","ignored","ignored",false),"/dir/base");
    assert_eq!(runtime::format_parts("","","",".txt","é😊",false),"é😊.txt");
}
#[test]
fn multipart_resolution_keeps_windows_context_demand_and_errors() {
    let mut count=0;
    for line in include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../benchmarks/runtime/admission/path-env-os/windows-resolve.tsv")).lines().filter(|line|!line.is_empty()&&!line.starts_with('#')) {
        let row=line.split('\t').collect::<Vec<_>>();let mut cwd_reads=0;let mut drive_reads=0;
        let actual=relative::resolve_with(&row[5..],true,||{cwd_reads+=1;row[0].strip_prefix("@error:").map_or_else(||Ok(row[0].to_owned()),|error|Err(error.to_owned()))},|drive|{
            drive_reads+=1;
            if let Some(error)=row[1].strip_prefix("@error:"){Err(error.to_owned())}else{Ok(row[1].get(..2).filter(|prefix|prefix.eq_ignore_ascii_case(drive)).map(|_|row[1].to_owned()))}
        }).unwrap_or_else(|error|format!("@error:{error}"));
        assert_eq!(actual,row[2],"{line}");assert_eq!(cwd_reads,row[3].parse::<usize>().unwrap(),"{line}");assert_eq!(drive_reads,row[4].parse::<usize>().unwrap(),"{line}");count+=1;
    }
    assert_eq!(count,22);
    let mut reads=0;
    assert_eq!(relative::resolve_with::<&str>(&["relative","/absolute","..","x"],false,||{reads+=1;Err("Io")},|_|Ok(None)),Ok("/x".into()));
    assert_eq!(reads,0);
    assert_eq!(relative::resolve_with::<&str>(&[],false,||Err("Io"),|_|Ok(None)),Err("Io"));
}
#[test]
fn existing_relative_reports_bad_drive_context_without_cwd_fallback() {
    let mut cwd_reads=0;
    let error=relative::relative_checked("C:first","C:second",true,||{cwd_reads+=1;Ok("C:\\cwd".into())},|_|Err("Drive working directory is not valid UTF-8.")).unwrap_err();
    assert_eq!(error,"Drive working directory is not valid UTF-8.");assert_eq!(cwd_reads,0);
    assert_eq!(relative::relative_checked::<&str>("C:\\first","C:\\second",true,||Err("cwd"),|_|Err("drive")),Ok("..\\second".into()));
    let error=relative::resolve_with(&["relative"],false,||Err(relative::PathError{code:"Io",message:"controlled cwd failure".into()}),|_|Ok(None)).unwrap_err();
    assert_eq!(error.code,"Io");assert_eq!(error.message,"controlled cwd failure");
}
