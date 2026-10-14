mod support;

#[test]
fn nested_set_and_map_sources_evaluate_once_in_order(){
    for set in [true,false]{
        let mut value=if set{"new Set<int>([mark(7)])".to_owned()}else{"new Map<int,int>([[mark(7),0]])".to_owned()};
        for index in 1..=24{
            value=if set{format!("new Set<int>([...new Set<int>({value}),mark({})])",100+index)}else{format!("new Map<int,int>([...new Map<int,int>({value}),[mark({}),{index}]])",100+index)};
        }
        let source=format!("function mark(value:int):int{{console.log(value);return value;}}function main(){{const values={value};console.log(values.size,values.has(7),values.has(124));}}");
        let source=support::format_round_trip(&source);let directory=support::TempDirectory::new();let path=directory.source(&source);
        let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
        let expected="7\n".to_owned()+&(101..=124).map(|value|format!("{value}\n")).collect::<String>()+"25 true true\n";
        assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
    }
}

#[test]
fn constructor_sources_stay_readable_and_copies_are_independent(){
    support::execute("function main(){const keys=new Set<int>([1,2]);const copy=new Set<int>(new Set<int>(keys));copy.add(3);const letters=new Set(new Set('abca'));const empty=new Set<int>(null);console.log(keys.size,copy.size,letters.size,empty.size);}","2 3 3 0\n",None);
    support::execute("function main(){const names=new Map<string,string>([['a','x']]);const copy=new Map<string,string>(new Map<string,string>(names));copy.set('a','y');const empty=new Map<int,int>(undefined);console.log(names.get('a'),copy.get('a'),empty.size);}","x y 0\n",None);
}
