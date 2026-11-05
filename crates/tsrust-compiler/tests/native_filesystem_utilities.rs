mod support;

#[test]
fn filesystem_records_paths_and_mutating_operations_execute_at_both_optimization_levels() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip(r#"
import {OpenOptions,FileTime,Metadata,DirEntry,openFile,close,stat,lstat,fileStat,readDir,mkdir,mkdirAll,rename,copyFile,removeFile,removeDir,removeDirAll,canonicalize,readLink,hardLink,appendFile,readTextFile,writeTextFile,truncate,syncAll,syncData,writeAll} from "std:fs";
import {fromUtf8,view,viewMut} from "std:bytes";
import {strictEqual} from "std:testing";
function identity<T>(value:T):T{return value;}
function main(){
  const at:FileTime={unixSeconds:-1,nanoseconds:999999999};strictEqual(at.nanoseconds,999999999);
  const options:OpenOptions={read:true,write:true,createNew:true};
  unwrap(mkdir("tree"));unwrap(mkdirAll("tree/nested/empty"));
  const file=unwrap(openFile("tree/é-🦀.txt",options));
  unwrap(writeAll(file,unwrap(fromUtf8("hé🦀"))));
  const before:Metadata=unwrap(fileStat(file));strictEqual(before.size,7);strictEqual(before.isFile,true);
  strictEqual(Object.keys(before).join(","),"isFile,isDirectory,isSymlink,size,readOnly,modified,accessed,created");
  unwrap(truncate(file,2));unwrap(syncData(file));unwrap(syncAll(file));unwrap(close(file));
  strictEqual(unwrap(stat("tree/é-🦀.txt")).size,2);strictEqual(before.size,7);
  unwrap(writeTextFile("tree/é-🦀.txt","A"));const bytes=unwrap(fromUtf8("xBCy"));
  unwrap(appendFile("tree/é-🦀.txt",unwrap(view(bytes,1,3))));
  unwrap(appendFile("tree/é-🦀.txt",unwrap(viewMut(bytes,0,1))));
  strictEqual(unwrap(readTextFile("tree/é-🦀.txt",4)),"ABCx");
  const copied=identity(copyFile);strictEqual(unwrap(copied("tree/é-🦀.txt","tree/copy")),4);
  unwrap(rename("tree/copy","tree/moved"));unwrap(hardLink("tree/moved","tree/hard"));unwrap(removeFile("tree/moved"));
  strictEqual(unwrap(readTextFile("tree/hard")),"ABCx");
  strictEqual(unwrap(lstat("tree/hard")).isSymlink,false);
  strictEqual(unwrap(stat(unwrap(canonicalize("tree/hard")))).size,4);
  match(readLink("tree/hard")){Ok(_)=>{throw "regular file link";},Err(error)=>{strictEqual(error.operation,"readLink");}}
  const snapshot:DirEntry[]=unwrap(readDir("tree",3));strictEqual(snapshot.length,3);
  match(readDir("tree",2)){Ok(_)=>{throw "entry limit";},Err(error)=>{strictEqual(error.code,"length-range");}}
  match(readDir("missing",-1)){Ok(_)=>{throw "negative limit";},Err(error)=>{strictEqual(error.code,"length-range");}}
  unwrap(removeDir("tree/nested/empty"));unwrap(removeDirAll("tree"));strictEqual(snapshot.length,3);
  console.log("utilities ok");
}
"#);
    let path=directory.source(&source);
    for release in [false,true] {
        let binary=directory.0.join(format!("utilities{}",std::env::consts::EXE_SUFFIX));
        let mut command=support::cli();command.arg("build").arg(&path).arg("-o").arg(&binary);if release {command.arg("--release");}
        support::success(&support::run(&mut command,120));
        let output=support::run(std::process::Command::new(&binary).current_dir(&directory.0),30);support::success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(),"utilities ok\n");
        assert!(!directory.0.join("tree").exists());
    }
}

#[test]
fn new_file_observers_do_not_relax_mutable_or_consumed_owner_checks() {
    for source in [
        "import {File,truncate} from 'std:fs';function action(file:ReadOnly<File>){truncate(file,0);}",
        "import {open,close,fileStat} from 'std:fs';function main(){const file=unwrap(open('input'));unwrap(close(file));fileStat(file);}",
        "import {create,bufferWriter,truncate} from 'std:fs';function main(){const writer=unwrap(bufferWriter(unwrap(create('input'))));truncate(writer,0);}",
        "import {File,seek} from 'std:fs';function action(file:ReadOnly<File>){seek(file,0,'start');}",
        "import {File,position} from 'std:fs';function action(file:ReadOnly<File>){position(file);}",
        "import {open,close,position} from 'std:fs';function main(){const file=unwrap(open('input'));unwrap(close(file));position(file);}",
        "import {File,seek} from 'std:fs';function action(file:Mutable<File>,origin:string){seek(file,0,origin);}",
        "import {File,seek} from 'std:fs';function action(file:Mutable<File>){seek(file,0,'middle');}",
        "import {open,bufferReader,position} from 'std:fs';function main(){const reader=unwrap(bufferReader(unwrap(open('input'))));position(reader);}",
    ] {
        let source=support::format_round_trip(source);
        assert!(tsrust_compiler::check(&source).is_err(),"{source}");
    }
}

#[test]
fn public_filesystem_example_has_exact_output_and_removes_its_tree() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip(include_str!("../../../examples/filesystem-utilities/main.tsr"));
    let path=directory.source(&source);
    let output=support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),120);support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),"5\nhe\n7\n3\n");assert!(!std::fs::read_dir(&directory.0).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with("tsrust-example-")));
}

#[cfg(unix)]
#[test]
fn public_link_metadata_and_recursive_removal_do_not_follow_directory_links() {
    use std::os::unix::fs::symlink;
    let directory=support::TempDirectory::new();
    std::fs::create_dir(directory.0.join("outside")).unwrap();std::fs::write(directory.0.join("outside/keep"),b"safe").unwrap();
    std::fs::create_dir(directory.0.join("tree")).unwrap();std::fs::write(directory.0.join("tree/regular"),b"abc").unwrap();
    symlink("../outside",directory.0.join("tree/linked-directory")).unwrap();symlink("missing",directory.0.join("tree/dangling")).unwrap();
    let source=support::format_round_trip(r#"
import {stat,lstat,readDir,readLink,removeDirAll,readTextFile} from "std:fs";
import {strictEqual} from "std:testing";
function main(){
  strictEqual(unwrap(stat("tree/linked-directory")).isDirectory,true);
  strictEqual(unwrap(lstat("tree/linked-directory")).isSymlink,true);
  strictEqual(unwrap(lstat("tree/dangling")).isSymlink,true);
  strictEqual(unwrap(readLink("tree/dangling")),"missing");
  match(stat("tree/dangling")){Ok(_)=>{throw "followed missing target";},Err(error)=>{strictEqual(error.code,"not-found");strictEqual(error.operation,"stat");}}
  const entries=unwrap(readDir("tree"));let links=0;let files=0;
  for(const entry of entries){if(entry.isSymlink){links++;strictEqual(entry.isDirectory,false);strictEqual(entry.isFile,false);}if(entry.isFile){files++;}}
  strictEqual(links,2);strictEqual(files,1);unwrap(removeDirAll("tree"));
  strictEqual(unwrap(readTextFile("outside/keep")),"safe");strictEqual(entries.length,3);
}
"#);
    let path=directory.source(&source);support::success(&support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),120));
    assert_eq!(std::fs::read(directory.0.join("outside/keep")).unwrap(),b"safe");assert!(!directory.0.join("tree").exists());
}

#[test]
fn plain_file_cursor_calls_preserve_aliases_offsets_eof_and_unique_handles() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip(r#"
import {SeekOrigin as Origin,writeTextFile,openFile,seek,position,read,writeAll,fileStat,close} from "std:fs";
import {length,toUtf8,fromUtf8} from "std:bytes";
import {strictEqual} from "std:testing";
function identity<T>(value:T):T{return value;}
function main(){
  unwrap(writeTextFile("cursor.bin","abcd"));const file=unwrap(openFile("cursor.bin",{read:true,write:true}));
  strictEqual(unwrap(position(file)),0);const move=identity(seek);const query=identity(position);
  const origin:Origin="start";strictEqual(unwrap(move(file,2,origin)),2);strictEqual(unwrap(toUtf8(unwrap(read(file,1)))),"c");strictEqual(unwrap(query(file)),3);
  strictEqual(unwrap(seek(file,-2,"current")),1);strictEqual(unwrap(toUtf8(unwrap(read(file,2)))),"bc");
  strictEqual(unwrap(seek(file,-1,"end")),3);strictEqual(unwrap(toUtf8(unwrap(read(file,1)))),"d");
  strictEqual(unwrap(seek(file,8,"start")),8);strictEqual(length(unwrap(read(file,64))),0);strictEqual(unwrap(position(file)),8);
  match(seek(file,-1,"start")){Ok(_)=>{throw "negative start";},Err(error)=>{strictEqual(error.code,"offset-range");strictEqual(error.osCode,undefined);}}
  strictEqual(unwrap(position(file)),8);unwrap(writeAll(file,unwrap(fromUtf8("Z"))));strictEqual(unwrap(fileStat(file)).size,9);strictEqual(unwrap(position(file)),9);
  unwrap(close(file));
}
"#);
    let path=directory.source(&source);support::success(&support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),120));
    assert_eq!(std::fs::read(directory.0.join("cursor.bin")).unwrap(),b"abcd\0\0\0\0Z");
}
