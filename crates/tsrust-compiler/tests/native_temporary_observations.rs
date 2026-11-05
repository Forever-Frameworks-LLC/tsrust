mod support;
use tsrust_compiler::project::{FileSourceProvider, ProjectAnalysis, ProjectDiagnostic, check_graph, load_project};

fn checked(source: &str) -> Result<ProjectAnalysis, ProjectDiagnostic> {
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    check_graph(load_project(&FileSourceProvider, path.to_str().unwrap())?)
}

#[test]
fn fresh_url_owners_survive_immediate_direct_and_generic_observations() {
    let source = r#"
import {Url,StringView,parseUrl,resolveUrl,formatUrl,parseQuery} from 'std:url';
import {strictEqual} from 'std:testing';
function identity<T>(value:T):T{return value;}
type Owners={left:Url;right:Url;};
function make(calls:Mutable<int[]>):Url{
  calls[0]=calls[0]*10+1;
  return unwrap(parseUrl('https://example.com/a?x=one'));
}
function expected(calls:Mutable<int[]>):string{
  calls[0]=calls[0]*10+2;
  return 'https://example.com/a?x=one';
}
function observe(text:ReadOnly<StringView>,expected:ReadOnly<string>):void{
  strictEqual(String(text),expected);
}
function changeRight(value:Mutable<Url>):string{
  value.pathname='/changed';
  return 'https://example.com/left';
}
function main(){
  const base=unwrap(parseUrl('https://example.com/a/b'));
  console.log(unwrap(resolveUrl(base,'../c')).href);
  const calls:int[]=[0];
  observe(make(calls).href,expected(calls));
  strictEqual(calls[0],12);
  calls[0]=0;
  strictEqual(make(calls).href.length,'https://example.com/a?x=one'.length);
  strictEqual(calls[0],1);
  calls[0]=0;
  strictEqual(String(formatUrl(make(calls))),'https://example.com/a?x=one');
  strictEqual(calls[0],1);
  calls[0]=0;
  const format=identity(formatUrl);
  observe(format(make(calls)),expected(calls));
  strictEqual(calls[0],12);
  console.log(parseQuery('x=one').get('x'));
  const owner=unwrap(parseUrl('https://example.com/'));
  observe(owner.href,'https://example.com/');
  console.log(owner.href,owner.origin);
  const owners:Owners={left:unwrap(parseUrl('https://example.com/left')),right:unwrap(parseUrl('https://example.com/right'))};
  observe(owners.left.href,changeRight(owners.right));
  strictEqual(String(owners.right.pathname),'/changed');
}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    for release in [false, true] {
        let binary = directory.0.join(format!("temporary-observations{}", std::env::consts::EXE_SUFFIX));
        let mut command = support::cli();
        command.arg("build").arg(&path).arg("-o").arg(&binary);
        if release { command.arg("--release"); }
        support::success(&support::run(&mut command, 180));
        let output = support::run(&mut std::process::Command::new(&binary), 15);
        support::success(&output);
        assert_eq!(output.stdout, b"https://example.com/c\none\nhttps://example.com/ https://example.com\n");
    }
}

#[test]
fn temporary_native_views_cannot_escape_into_persistent_values() {
    let prelude = "import {Url,StringView,parseUrl,formatUrl} from 'std:url';function make():Url{return unwrap(parseUrl('https://example.com/a'));}function identity<T>(value:T):T{return value;}type Holder={text:StringView;};";
    for body in [
        "function main(){const text=make().href;console.log(text);}",
        "function escape():StringView{return make().href;}function main(){console.log(escape());}",
        "function main(){const texts=[make().href];console.log(texts);}",
        "function main(){const holder:Holder={text:make().href};console.log(holder);}",
        "function main(){const read=()=>make().href;console.log(read());}",
        "function main(){const format=identity(formatUrl);const text=format(make());console.log(text);}",
        "function main(){const text=identity(make().href);console.log(text);}",
    ] {
        let source = format!("{prelude}{body}");
        let error = checked(&source).err().unwrap_or_else(|| panic!("accepted temporary view escape: {source}"));
        assert_eq!(error.code, "native-view-source", "{source}: {}", error.message);
    }
}

#[test]
fn mixed_source_alternatives_keep_the_named_storage_requirement() {
    // This finite model admits a fresh checked owner call, not a conditional
    // owner carrier. An untracked branch must never erase the named branch loan.
    let prelude = "import {Url,parseUrl,formatUrl} from 'std:url';function make():Url{return unwrap(parseUrl('https://example.com/'));}";
    for body in [
        "function choose(flag:boolean){const owner=make();console.log(formatUrl(flag?owner:make()));}function main(){choose(true);}",
        "function choose(flag:boolean){const owner=make();const text=formatUrl(flag?owner:make());console.log(text);}function main(){choose(true);}",
    ] {
        let source = format!("{prelude}{body}");
        let error = checked(&source).err().unwrap_or_else(|| panic!("accepted mixed temporary source: {source}"));
        assert_eq!(error.code, "native-view-source", "{source}: {}", error.message);
    }
}

#[test]
fn transient_native_arguments_keep_whole_owner_and_exclusive_access() {
    for source in [
        "import {Url,StringView,parseUrl} from 'std:url';function mutate(value:Mutable<Url>):string{value.pathname='/changed';return 'changed';}function observe(text:ReadOnly<StringView>,other:ReadOnly<string>):void{console.log(text,other);}function main(){const owner=unwrap(parseUrl('https://example.com/'));observe(owner.href,mutate(owner));}",
        "import {Url,parseUrl} from 'std:url';function mutate(value:Mutable<Url>):string{value.pathname='/changed';return 'changed';}function main(){const owner=unwrap(parseUrl('https://example.com/'));console.log(owner.href,mutate(owner));}",
        "import {Url,parseUrl} from 'std:url';function mutate(value:Mutable<Url>):string{value.pathname='/changed';return 'changed';}function main(){const owner=unwrap(parseUrl('https://example.com/'));console.log(owner.href.includes(mutate(owner)));}",
        "import {Url,StringView,parseUrl} from 'std:url';function mutate(value:Mutable<Url>):string{value.pathname='/changed';return 'changed';}function observe(text:ReadOnly<StringView>,other:ReadOnly<string>):void{console.log(text,other);}function main(){const owner=unwrap(parseUrl('https://example.com/'));observe(...[owner.href],mutate(owner));}",
        "import {ByteViewMut,fromUtf8,viewMut,length} from 'std:bytes';function observe(part:ReadOnly<ByteViewMut>,other:int):void{console.log(length(part),other);}function main(){const bytes=unwrap(fromUtf8('hello'));observe(unwrap(viewMut(bytes,0,1)),length(bytes));}",
        "import {ByteViewMut,fromUtf8,viewMut,length} from 'std:bytes';function identity<T>(value:T):T{return value;}function observe(part:ReadOnly<ByteViewMut>,other:int):void{console.log(length(part),other);}function main(){const bytes=unwrap(fromUtf8('hello'));const exclusive=identity(viewMut);observe(unwrap(exclusive(bytes,0,1)),length(bytes));}",
        "import {ByteViewMut,fromUtf8,viewMut,length} from 'std:bytes';function observe(part:ReadOnly<ByteViewMut>,other:int):void{console.log(length(part),other);}function main(){const bytes=unwrap(fromUtf8('hello'));observe(...[unwrap(viewMut(bytes,0,1))],length(bytes));}",
    ] {
        let error = checked(source).err().unwrap_or_else(|| panic!("accepted invalidating later argument: {source}"));
        assert_eq!(error.code, "conflicting-access", "{source}: {}", error.message);
        assert!(!error.related.is_empty(), "earlier argument must be identified: {error:?}");
    }
}
