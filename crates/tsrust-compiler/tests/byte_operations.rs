mod support;
use tsrust_compiler::project::{FileSourceProvider,check_graph,load_project};
#[test]
fn byte_operations_run_vectors_views_aliases_and_all_catalog_examples_at_o0_and_o3(){
    let directory=support::TempDirectory::new();let fixture=include_str!("fixtures/byte-operations/main.tsr");
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut examples=String::new();let mut expected=String::new();
    for name in ["concat","copyInto","copyWithin","equals","compare","indexOf","readInt","writeInt","readUint","writeUint","readFloat32","writeFloat32","readFloat64","writeFloat64"]{
        let entry=catalog["std:bytes"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();let example=&entry["examples"][0];
        let body=example["source"].as_str().unwrap().split_once("function main() {").unwrap().1.trim().strip_suffix('}').unwrap();examples.push_str(&format!("{{{body}}}\n"));expected.push_str(example["output"].as_str().unwrap());
    }
    let end=fixture.rfind('}').unwrap();let source=format!("{}{examples}}}",&fixture[..end]);let path=directory.source(&support::format_round_trip(&source));
    for release in [false,true]{let binary=directory.0.join(format!("operations{}",std::env::consts::EXE_SUFFIX));let mut build=support::cli();build.arg("build").arg(&path).arg("-o").arg(&binary);if release{build.arg("--release");}support::success(&support::run(&mut build,180));let output=support::run(&mut std::process::Command::new(binary),30);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());}
}
#[test]
fn byte_operations_reject_missing_coordinates_nominal_inputs_and_aliasing(){
    for source in [
        "import {fromArray,copyInto} from 'std:bytes';function main(){const bytes=unwrap(fromArray([1]));copyInto(bytes,0,bytes);}",
        "import {fromArray,view,copyInto} from 'std:bytes';function main(){const bytes=unwrap(fromArray([1]));const part=unwrap(view(bytes,0,1));copyInto(bytes,0,part);}",
        "import {fromArray,view,copyInto} from 'std:bytes';function main(){const bytes=unwrap(fromArray([1]));const source=unwrap(fromArray([2]));const part=unwrap(view(bytes,0,1));copyInto(part,0,source);}",
        "import {fromArray,concat} from 'std:bytes';function main(){concat([1],unwrap(fromArray([1])));}",
        "import {fromArray,indexOf} from 'std:bytes';function main(){indexOf(unwrap(fromArray([1])),1,0);}",
        "import {fromArray,indexOf} from 'std:bytes';function main(){const bytes=unwrap(fromArray([1]));indexOf(bytes,bytes);}",
        "import {fromArray,readInt} from 'std:bytes';function main(){readInt(unwrap(fromArray([1])),0,1,'native');}",
        "import {fromArray,writeFloat32} from 'std:bytes';function main(){writeFloat32(unwrap(fromArray([1])),0,1.0);}",
        "import {Bytes,fromArray,set,indexOf} from 'std:bytes';function change(bytes:Mutable<Bytes>):int{unwrap(set(bytes,0,9));return 0;}function main(){const bytes=unwrap(fromArray([1]));const needle=unwrap(fromArray([1]));indexOf(bytes,needle,change(bytes));}",
    ]{let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));assert!(check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).is_err(),"accepted {source}");}
}
