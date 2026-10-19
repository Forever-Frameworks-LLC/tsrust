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

#[test]
fn parenthesized_multiline_jsx_keeps_return_layout() {
    let source=include_str!("../../../examples/ui-counter/screen.tsrx");
    let formatted=support::format_round_trip(source);
    assert!(formatted.contains("return (\n    <View"));
    assert!(formatted.contains("\n  );"));
}

#[test]
fn quoted_properties_and_hyphenated_jsx_attributes_preserve_identity() {
    let source=r#"
interface Identity { 'aria-label': string; 'data-slot'?: string; }
interface Props extends Identity { children: string[]; }
function Label(props: Props): string { return props['aria-label'] + ':' + props.children.join(''); }
type Values = { 'with space': int; 'a.b': string; };
function main() {
    const props: Identity = {'aria-label': 'Accessible', 'data-slot': 'label'};
    console.log(<Label {...props} aria-label="Override">Body</Label>);
    const values: Values = {'a.b':'exact', 'with space':7};
    console.log(values['with space'], values['a.b'], JSON.stringify(values));
}
"#;
    let dir=support::TempDirectory::new();let path=dir.0.join("quoted-properties.tsrx");
    std::fs::write(&path,support::format_round_trip(source)).unwrap();
    let output=support::run(support::cli().arg("run").arg(&path),90);support::success(&output);
    assert_eq!(output.stdout,b"Override:Body\n7 exact {\"a.b\":\"exact\",\"with space\":7}\n");
    for (from,to) in [("aria-label=\"Override\"","aria-label={7}"),("aria-label=\"Override\"","aria-label=\"one\" aria-label=\"two\""),("'with space':7","'with space':'wrong'")] {
        assert!(analyze(&source.replace(from,to)).is_err(),"{to}");
    }
    assert!(analyze("type Bad = { 'bad\\0key': int; }; function main(){}").err().unwrap().message.contains("NUL"));
    assert!(analyze("function main(){const invalid={'quoted'};}").is_err());
}

#[test]
fn quoted_owned_fields_keep_distinct_move_paths() {
    let source="type Inner={b:string;};type Row={'a.b':string; a:Inner;};function main(){const row:Row={'a.b':'flat',a:{b:'nested'}};const flat=row['a.b'];const nested=row.a.b;console.log(flat,nested);}";
    support::execute(source,"flat nested\n",None);
    let source=source.replace("console.log(flat,nested);","console.log(flat,nested);console.log(row['a.b']);");
    assert!(analyze(&source).is_err(),"moving a quoted field must invalidate that exact owner");
}

#[test]
fn jsx_spread_projects_declared_props_and_defaults_missing_optional_fields() {
    support::execute(r#"
interface Base {value:int; note?:string;}
interface Extended extends Base {extra:string;}
function Label(props:Base):string{return String(props.value)+':'+(props.note??'none');}
function source():Extended{console.log('source');return {value:9007199254740993,extra:'extra'};}
function override():int{console.log('override');return 7;}
function main(){console.log(<Label {...source()}/>);console.log(<Label {...source()} value={override()}/>);console.log(<Label {...{value:3,extra:'ignored'}}/>);}
"#, "source\n9007199254740993:none\nsource\noverride\n7:none\n3:none\n",None);
    for (source,expected) in [
        ("interface Base{value:int;}interface Extra extends Base{extra:string;}function Label(p:Base):int{return p.value;}function main(){const source:Extra={value:1,extra:'x'};const n=<Label {...source} extra='explicit'/>;}","Unknown JSX attribute"),
        ("interface Base{value:int;}interface Extra{extra:string;}function Label(p:Base):int{return p.value;}function main(){const source:Extra={extra:'x'};const n=<Label {...source}/>;}","Missing required JSX attribute"),
        ("interface Base{value:int;}interface Wrong{value:string;}function Label(p:Base):int{return p.value;}function main(){const source:Wrong={value:'wrong'};const n=<Label {...source}/>;}","Expected int"),
        ("interface Base{value:int;}function Label(p:Base):int{return p.value;}function main(){const source:Base={value:1};const n=<Label {...source}/>;console.log(source.value);}","transferred"),
        ("interface Base{value:int;}interface Extra extends Base{extra:int;}function main(){const source:Extra={value:1,extra:2};const narrowed:Base={...source};}","Spread record fields must match"),
    ] {let formatted=support::format_round_trip(source);let error=analyze(&formatted).expect_err(source);assert!(error.message.contains(expected),"{expected}: {error:?}");}
}

#[test]
fn explicit_jsx_children_forward_once_and_reject_duplicate_child_sources(){
 let source=r#"
interface Props { children?: string[]; }
function Box(props:Props):string{return (props.children??[]).join('|');}
function values():string[]{console.log('children evaluated');return ['a','🦀'];}
function main(){console.log(<Box children={values()}/>);console.log(<Box children={['empty body']}> 
 </Box>);}
"#;
 let formatted=support::format_round_trip(source);support::execute(&formatted,"children evaluated\na|🦀\nempty body\n",None);
 for invalid in ["<Box children={['a']}>body</Box>","<Box children={['a']} children={['b']}/>"]{
  let broken=format!("interface Props{{children:string[];}}function Box(props:Props):string{{return props.children.join('');}}function main(){{console.log({invalid});}}");
  assert!(analyze(&broken).is_err());assert!(format(&broken).is_err());
 }
}
