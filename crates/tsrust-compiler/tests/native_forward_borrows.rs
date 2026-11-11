mod support;

#[test]
fn native_forwarded_views_outlive_the_callable_and_keep_the_argument_owner_at_o0_and_o3() {
    let directory=support::TempDirectory::new();
    let source=r#"
import {Bytes,ByteView,ByteViewMut,fromUtf8,view,viewMut,toUtf8,set} from 'std:bytes';
import {Url,StringView,parseUrl,formatUrl} from 'std:url';
import {JsonValue,JsonView,parse as parseJson,get,asInt} from 'std:json';
import {strictEqual} from 'std:testing';
function shared(bytes:ReadOnly<Bytes>):ByteView{const observe=view;return unwrap(observe(bytes,1,4));}
function exclusive(bytes:Mutable<Bytes>):ByteViewMut{const edit=viewMut;return unwrap(edit(bytes,0,1));}
function addressText(address:ReadOnly<Url>):StringView{const text=formatUrl;return text(address);}
function child(owner:ReadOnly<JsonValue>,key:string):JsonView|undefined{const find=get;return unwrap(find(owner,key));}
function main(){
  const bytes=unwrap(fromUtf8('hello'));
  {const part=shared(bytes);strictEqual(unwrap(toUtf8(part)),'ell');}
  {const part=exclusive(bytes);unwrap(set(part,0,72));}
  strictEqual(unwrap(toUtf8(bytes)),'Hello');
  const address=unwrap(parseUrl('https://EXAMPLE.com/a'));
  const text=addressText(address);strictEqual(String(text),'https://example.com/a');
  const document=unwrap(parseJson('{"value":7}'));
  const field=child(document,'value');if(field===undefined){throw 'value missing';}else{strictEqual(unwrap(asInt(field)),7);}
  const missing=child(document,'absent');if(missing!==undefined){throw 'unexpected child';}
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    for release in [false,true] {
        let binary=directory.0.join(format!("native-forward-borrows{}",std::env::consts::EXE_SUFFIX));
        let mut command=support::cli();command.arg("build").arg(&path).arg("-o").arg(&binary);
        if release {command.arg("--release");}
        support::success(&support::run(&mut command,180));
        let output=support::run(&mut std::process::Command::new(&binary),30);support::success(&output);assert!(output.stdout.is_empty());
    }
}
