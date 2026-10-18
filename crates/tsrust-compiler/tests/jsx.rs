mod support;
use tsrust_compiler::{check as analyze, format};

#[test]
fn jsx_components_use_checked_calls_and_preserve_source() {
    let source=r#"
interface LabelProps { children: string[]; }
function Label(props: LabelProps): string { return props.children.join(""); }
interface BoxProps { children: string[]; }
function Box(props: BoxProps): string { return props.children.join("|"); }
function main() {
    const name = "café";
    const output = <Box><Label>Hello {name}</Label><Label>🦀</Label></Box>;
    console.log(output);
}
"#;
    support::execute(source,"Hello café|🦀\n",None);
    let formatted=format(source).unwrap();
    assert_eq!(format(&formatted).unwrap(),formatted);
    analyze(&formatted).unwrap();
    assert!(formatted.contains("<Label>Hello {name}</Label>"));
}

#[test]
fn jsx_attributes_callbacks_and_diagnostics_share_the_language() {
    let source=r#"
interface Props { title: string; disabled: boolean; onPress: Function<int>; }
function Button(props: Props): int { return props.onPress(); }
function main() { const result=<Button title="test" disabled onPress={() => 42} />; console.log(result); }
"#;
    analyze(source).unwrap();
    analyze(&format(source).unwrap()).unwrap();
    for (from,to) in [("title=\"test\"","title={42}"),("disabled onPress","disabled disabled onPress"),("onPress={() => 42}","onPress={() => \"wrong\"}")] {
        assert!(analyze(&source.replace(from,to)).is_err(),"{to}");
    }
    assert!(analyze("function main(){const x=<Box></Wrong>;}").err().unwrap().message.contains("closing JSX tag Box"));
}

#[test]
fn jsx_lexing_keeps_generic_calls_comparisons_and_template_expressions() {
    let source=r#"
interface Props { children: string[]; }
function Text(props: Props): string { return props.children.join(""); }
function identity<T>(value: T): T {return value;}
function main(){const a=identity<int>(1); const b=a<2; const text=<Text>{`value ${a}`}</Text>;console.log(text,b);}
"#;
    analyze(source).unwrap();
    analyze(&format(source).unwrap()).unwrap();
}

#[test]
fn jsx_entities_and_multiline_text_are_explicit() {
    let source=r#"
interface Props { children: string[]; }
function Text(props: Props): string { return props.children.join(""); }
function main(){console.log(<Text>Hello &amp; &#x1F980;
  café
</Text>);}
"#;
    support::execute(source,"Hello & 🦀 café\n",None);
    for entity in ["&unknown;","&#xD800;","&broken"] {
        assert!(analyze(&source.replace("&amp;",entity)).is_err());
    }
}

#[test]
fn tsrx_project_resolution_discovery_and_cli_execution() {
    use tsrust_compiler::project::{FileSourceProvider,SourceId,SourceProvider};
    let dir=support::TempDirectory::new();
    std::fs::write(dir.0.join("label.tsrx"),support::format_round_trip(r#"
export interface Props { children: string[]; }
export function Label(props: Props): string {return props.children.join("");}
"#)).unwrap();
    std::fs::write(dir.0.join("main.tsrx"),support::format_round_trip(r#"
import {Label} from "./label";
function main(){console.log(<Label>paired syntax</Label>);}
"#)).unwrap();
    std::fs::write(dir.0.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"jsx","build":{"entry":"main.tsrx","output":"dist/jsx"}}"#).unwrap();
    let importer=SourceId(dir.0.join("main.tsrx").to_str().unwrap().into());
    let resolved=FileSourceProvider.resolve(Some(&importer),"./label").unwrap();
    assert!(resolved.0.ends_with("label.tsrx"));
    assert_eq!(tsrust_compiler::source_discovery::discover(&[dir.0.clone()]).unwrap().len(),2);
    let output=support::run(support::cli().arg("run").current_dir(&dir.0),45);
    support::success(&output);assert_eq!(output.stdout,b"paired syntax\n");
}
