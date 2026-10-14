mod support;
#[test]
fn nested_spreads_evaluate_once_preserve_key_order_and_snapshot_sources(){
    let mut value="{value:mark(7),text:'retained'}".to_owned();
    for index in 1..=40{value=format!("{{...{value},value:mark({})}}",100+index);}
    let source=format!("function mark(value:int):int{{console.log(value);return value;}}function main(){{const source={value};let copy={{...source}};copy.value=999;console.log(Object.keys(copy),source.value,copy.value,source.text);}}");
    let source=support::format_round_trip(&source);let directory=support::TempDirectory::new();let path=directory.source(&source);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    let expected="7\n".to_owned()+&(101..=140).map(|value|format!("{value}\n")).collect::<String>()+"[ 'value', 'text' ] 140 999 retained\n";
    assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
}
#[test]
fn annotated_overwritten_field_and_parameter_snapshot_execute(){
    support::execute("type Item={value:int;text:string;};function copy(source:Item):Item{return {value:'overwritten',...{...source}};}function main(){const original:Item={text:'retained',value:7};let result=copy(original);result.value=9;console.log(Object.keys(result),original.value,result.value,result.text);}","[ 'value', 'text' ] 7 9 retained\n",None);
}
