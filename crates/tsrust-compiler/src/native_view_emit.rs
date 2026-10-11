//! Native byte-view storage is a real slice lifetime, including generic wrappers.
use crate::{analysis::CheckedProgram, native_return_borrows::BorrowAccess, syntax::Type};

/// Reverse reachability makes nested storage queries constant-time, including cycles.
#[derive(Default)]
pub(super) struct Shapes {
    records: Vec<bool>,
    unions: Vec<bool>,
    closures: Vec<bool>,
    families: Vec<bool>,
    recursive: Option<crate::recursive_view_shapes::Index>,
}

impl Shapes {
    pub(super) fn new(program: &CheckedProgram) -> Self {
        let count=program.records.len();
        let seeds=program.records.iter().enumerate().filter_map(|(id,record)|
            record.native_type.and_then(|native|crate::native_types::declaration(native).borrow_access()).map(|_|id)
        ).collect::<Vec<_>>();
        if seeds.is_empty() {return Self::default();}
        let unions=program.unions.len();
        let closures=program.closures.len();
        let families=program.stored_contracts.families().count();
        let mut reverse=vec![Vec::new();count+unions+closures+families];
        let mut edge=|parent,ty| {
            if let Some(child)=Self::node(ty,count,unions,closures,families)
                && let Some(parents)=reverse.get_mut(child) {parents.push(parent);}
        };
        for (id,record) in program.records.iter().enumerate() {
            for field in &record.fields {edge(id,field.ty.expect("checked field"));}
            if let Some(map)=&record.map_definition {edge(id,map.key);edge(id,map.value);}
            if let Some(set)=&record.set_definition {edge(id,set.key);}
        }
        for (id,union) in program.unions.iter().enumerate() {
            for &member in &union.members {edge(count+id,member);}
        }
        for (id,closure) in program.closures.iter().enumerate() {
            for capture in &closure.captures {edge(count+unions+id,closure.function.bindings[capture.inner].ty);}
        }
        for (id,_,origins) in program.stored_contracts.families() {
            for &origin in origins {edge(count+unions+closures+id,Type::Closure(origin));}
        }
        let mut found=vec![false;reverse.len()];
        let mut pending=seeds;
        while let Some(id)=pending.pop() {
            if std::mem::replace(&mut found[id],true) {continue;}
            pending.extend(reverse[id].iter().copied());
        }
        Self {
            records:found[..count].to_vec(),
            unions:found[count..count+unions].to_vec(),
            closures:found[count+unions..count+unions+closures].to_vec(),
            families:found[count+unions+closures..].to_vec(),
            recursive:Some(crate::recursive_view_shapes::Index::new(&program.records, &|id| program.unions[id.index()].members.clone())),
        }
    }

    fn node(ty:Type,count:usize,unions:usize,closures:usize,families:usize)->Option<usize> {
        match ty {
            Type::Record(id)|Type::Map(id)|Type::Set(id) if id<count=>Some(id),
            Type::Union(id)|Type::BorrowedUnion(id) if id.index()<unions=>Some(count+id.index()),
            Type::Closure(id) if id<closures=>Some(count+unions+id),
            Type::StoredCallable(id)|Type::BorrowedCallable(id) if id<families=>Some(count+unions+closures+id),
            Type::Array(element)=>Self::node(element.with_depth(0).ty(),count,unions,closures,families),
            _=>None,
        }
    }

    pub(super) fn contains(&self,ty:Type)->bool {
        let count=self.records.len();let unions=self.unions.len();let closures=self.closures.len();
        let Some(id)=Self::node(ty,count,unions,closures,self.families.len()) else {return false;};
        if id<count {self.records.get(id)}
        else if id<count+unions {self.unions.get(id-count)}
        else if id<count+unions+closures {self.closures.get(id-count-unions)}
        else {self.families.get(id-count-unions-closures)}.copied().unwrap_or(false)
    }

    pub(super) fn recursive(&self, record: usize) -> bool {
        self.recursive.as_ref().is_some_and(|index| index.access(record).is_some())
    }
}

pub(super) fn access(program: &CheckedProgram, id: usize) -> Option<BorrowAccess> {
    program.records[id]
        .native_type
        .and_then(|native| crate::native_types::declaration(native).borrow_access())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_storage_reachability_handles_cycles_and_array_ranks() {
        let mut parsed = crate::syntax::parse("type Window={};type Ring={next:Ring[];view:Window[][];optional:Window|null;};type Plain={value:int;};function main(){}").unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let checked = crate::analysis::analyze(parsed).unwrap();
        let shapes = Shapes::new(&checked);
        let ring = checked
            .records
            .iter()
            .position(|record| record.name.text == "Ring")
            .unwrap();
        let plain = checked
            .records
            .iter()
            .position(|record| record.name.text == "Plain")
            .unwrap();
        assert!(shapes.contains(Type::Record(ring)));
        for field in &checked.records[ring].fields {
            assert!(shapes.contains(field.ty.unwrap()));
        }
        assert!(!shapes.contains(Type::Record(plain)));
        let ordinary = crate::analysis::analyze(
            crate::syntax::parse("type Plain={value:int;};function main(){}").unwrap(),
        )
        .unwrap();
        let shapes = Shapes::new(&ordinary);
        assert!(shapes.records.is_empty() && shapes.unions.is_empty());
        assert!(!shapes.contains(Type::Record(0)));
    }

    #[test]
    fn native_view_records_and_wrappers_emit_real_slice_lifetimes() {
        let source = "type ReadWindow={};type WriteWindow={};type Shelf={item:ReadWindow;items:ReadWindow[];};type Outcome=Result<ReadWindow,int>;type Carrier={result:Outcome;};function inspect(input:ReadOnly<Outcome>):void{}function main(){}";
        let formatted = crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut parsed = crate::syntax::parse(&formatted).unwrap();
        for record in &mut parsed.records {
            record.native_type = match record.name.text.as_str() {
                "ReadWindow" => Some(crate::native_signatures::NativeTypeId::ByteView),
                "WriteWindow" => Some(crate::native_signatures::NativeTypeId::ByteViewMut),
                _ => None,
            };
        }
        let checked = crate::analysis::analyze(parsed).unwrap();
        let record = |name: &str| {
            checked
                .records
                .iter()
                .position(|record| record.name.text == name)
                .unwrap()
        };
        let read = record("ReadWindow");
        let write = record("WriteWindow");
        let shelf = record("Shelf");
        let carrier = record("Carrier");
        let outcome = checked.records[carrier].fields[0]
            .ty
            .unwrap()
            .union()
            .unwrap();
        let members = &checked.unions[outcome.index()].members;
        let [ok, err] = crate::result_identity::variants(&checked.records, members).unwrap();
        let ok_tag = members
            .iter()
            .position(|member| *member == Type::Record(ok))
            .unwrap();
        let err_tag = members
            .iter()
            .position(|member| *member == Type::Record(err))
            .unwrap();
        let rust = crate::emit::emit(&checked);
        assert!(
            rust.contains(&format!("struct Record{read}<'a> {{ bytes: &'a [u8] }}")),
            "borrowed native representation missing"
        );
        assert!(
            rust.contains(&format!(
                "struct Record{write}<'a> {{ bytes: &'a mut [u8] }}"
            )),
            "exclusive native representation missing"
        );
        let rust = rust.replace("fn main()", "fn unused_source_main()")
            + &format!(
                r#"
fn main() {{
    let mut owner = vec![0u8, 65, 66, 255];
    let pointer = owner.as_ptr();
    {{
        let shelf = Record{shelf} {{
            field0: Record{read} {{ bytes: &owner[..2] }},
            field1: RecordArray{read}::from([Record{read} {{ bytes: &owner[2..] }}]),
        }};
        assert_eq!(shelf.field0.bytes.as_ptr(), pointer);
        assert_eq!(shelf.field1[0].bytes, [66, 255]);
    }}
    {{
        let mutable = Record{write} {{ bytes: &mut owner[1..3] }};
        mutable.bytes[0] = 7;
    }}
    assert_eq!(owner, [0, 7, 66, 255]);
    let outcome: Union{union}<Record{ok}<Record{read}<'_>>, Record{err}> =
        Union{union}::V{ok_tag}(Record{ok} {{ field0: Record{read} {{ bytes: &owner[1..] }} }});
    let carrier = Record{carrier} {{ field0: outcome }};
    match carrier.field0.tsr_view() {{
        ViewUnion{union}::V{ok_tag}(value) => assert_eq!(value.field0.bytes, [7, 66, 255]),
        ViewUnion{union}::V{err_tag}(_) => panic!("unexpected error"),
    }}
}}
"#,
                union = outcome.index()
            );
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "tsrust-native-view-abi-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let input = directory.join("views.rs");
        std::fs::write(&input, rust).unwrap();
        for level in [0, 3] {
            let output = directory.join(format!("views-{level}"));
            let compiled = std::process::Command::new("rustup")
                .args([
                    "run",
                    crate::RUST_BACKEND_VERSION,
                    "rustc",
                    "--edition=2024",
                    &format!("-Copt-level={level}"),
                ])
                .arg(&input)
                .arg("-o")
                .arg(&output)
                .output()
                .unwrap();
            assert!(
                compiled.status.success(),
                "{}",
                String::from_utf8_lossy(&compiled.stderr)
            );
            let run = std::process::Command::new(output).output().unwrap();
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
    #[test]
    fn native_view_receiver_signatures_keep_owned_storage_borrowed() {
        use crate::analysis::native_view_loans::{Loan,LifetimeSource};
        let source="type Window={};function factory(owner:string):Function<int>{return ():int=>owner.length;}function main(){const callback=factory('bytes');callback();}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let directory=std::env::temp_dir().join(format!("tsrust-native-receiver-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());
        for borrowed in [false,true] {
        for exclusive in [false,true] {
            let formatted=if borrowed {
                crate::formatting::format("type Window={};function main(){let owner='bytes';const callback=():int=>{owner='next';return owner.length;};callback();}").unwrap()
            } else {formatted.clone()};
            assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
            let mut parsed=crate::syntax::parse(&formatted).unwrap();
            parsed.records[0].native_type=Some(if exclusive {crate::native_signatures::NativeTypeId::ByteViewMut}else{crate::native_signatures::NativeTypeId::ByteView});
            let mut checked=crate::analysis::analyze(parsed).unwrap();
            let closure=&mut checked.closures[0];let capture=closure.captures[0].inner;
            assert_eq!(closure.captures[0].storage,if borrowed {crate::Mode::Mutate}else{crate::Mode::Consume});
            closure.invocation=if exclusive || borrowed {crate::Mode::Mutate}else{crate::Mode::Read};
            closure.function.result=Type::Record(0);
            closure.function.native_returns=Some(Box::new(vec![Loan{ whole: false,
                root:capture,owner_fields:vec![],source:LifetimeSource::Storage,
                access:if exclusive {BorrowAccess::Exclusive}else{BorrowAccess::ReadOnly},
                creation:crate::Span::default(),retained:true,path:vec![],guards:vec![],
            }]));
            let generated=crate::emit::emit(&checked);
            let signature=generated.lines().find(|line|line.trim_start().starts_with("fn call")).unwrap().trim();
            // Native factory bodies remain guarded; exercise the actual planned header
            // with a safe Vec-backed owner model, including exclusive reborrowing.
            let reference=if exclusive {"mut "}else{""};
            let operation=if exclusive {"as_mut_slice"}else{"as_slice"};
            let (declaration,implementation,field,backing,initial)=if borrowed {
                ("Owner<'c0>","impl<'c0> Owner<'c0>","&'c0 mut Vec<u8>","let mut backing=vec![1,2];", "&mut backing")
            }else{("Owner","impl Owner","Vec<u8>","","vec![1,2]")};
            let callback=format!("let mut callback=Owner{{capture{capture}:{initial}}};");
            let definitions=format!("struct Record0<'a>{{bytes:&'a {reference}[u8]}} struct {declaration}{{capture{capture}:{field}}} {implementation}{{{signature} Record0{{bytes:self.capture{capture}.{operation}()}} }} }}");
            let change=if exclusive {"returned.bytes[0]=9;"}else if borrowed {""}else{"let second=callback.call();assert_eq!(second.bytes.as_ptr(),pointer);"};
            let expected=if exclusive {9}else{1};
            let positive=definitions.clone()+&format!(r#"
fn main() {{
    {backing}{callback}let pointer=callback.capture{capture}.as_ptr();
    {{let returned=callback.call();assert_eq!(returned.bytes.as_ptr(),pointer);{change}assert_eq!(returned.bytes[1],2);}}
    callback.capture{capture}.push(3);assert_eq!(callback.capture{capture}[0],{expected});
}}
"#);
            let input=directory.join(format!("positive-{borrowed}-{exclusive}.rs"));std::fs::write(&input,positive).unwrap();
            for level in [0,3] {
                let output=directory.join(format!("positive-{borrowed}-{exclusive}-{level}"));
                let built=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024",&format!("-Copt-level={level}")]).arg(&input).arg("-o").arg(&output).output().unwrap();
                assert!(built.status.success(),"{}",String::from_utf8_lossy(&built.stderr));
                let run=std::process::Command::new(output).output().unwrap();assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
            }
            let mut failures=vec![
                (format!("{backing}let escaped;{{{callback}escaped=callback.call();}}assert_eq!(escaped.bytes[0],1);"),"does not live long enough"),
                (format!("{backing}{callback}let returned=callback.call();drop(callback);assert_eq!(returned.bytes[0],1);"),"cannot move out"),
                (format!("{backing}{callback}let returned=callback.call();callback.capture{capture}.push(3);assert_eq!(returned.bytes[0],1);"),"cannot borrow"),
            ];
            if exclusive || borrowed {failures.push((format!("{backing}{callback}let first=callback.call();let second=callback.call();assert_eq!(first.bytes[0],second.bytes[0]);"),"cannot borrow"));}
            for (case,(body,expected)) in failures.into_iter().enumerate() {
                let input=directory.join(format!("negative-{borrowed}-{exclusive}-{case}.rs"));std::fs::write(&input,definitions.clone()+"fn main(){"+&body+"}").unwrap();
                let failed=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024"]).arg(&input).arg("-o").arg(directory.join(format!("negative-{borrowed}-{exclusive}-{case}"))).output().unwrap();
                assert!(!failed.status.success(),"receiver borrow conflict {exclusive}/{case} compiled");
                assert!(String::from_utf8_lossy(&failed.stderr).contains(expected),"{}",String::from_utf8_lossy(&failed.stderr));
            }
        }
        }
    }
    #[test]
    fn native_view_of_borrowed_capture_uses_owner_reference_lifetime() {
        use crate::analysis::native_view_loans::{Loan,LifetimeSource};
        let source="type Window={};function sample(owner:ReadOnly<string>):void{const callback=():int=>owner.length;callback();}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        let mut checked=crate::analysis::analyze(parsed).unwrap();
        let closure=&mut checked.closures[0];
        let capture=closure.captures[0].inner;
        assert_eq!(closure.captures[0].storage,crate::Mode::Read);
        // Inject the private owner-borrow summary. Source factories are still guarded.
        closure.function.result=Type::Record(0);
        closure.function.native_returns=Some(Box::new(vec![Loan{ whole: false,
            root:capture,owner_fields:vec![],source:LifetimeSource::Storage,access:BorrowAccess::ReadOnly,
            creation:crate::Span::default(),retained:true,path:vec![],guards:vec![],
        }]));
        let generated=crate::emit::emit(&checked);
        let signature=generated.lines().find(|line|line.trim_start().starts_with("fn call")).unwrap().trim();
        // Test the actual planned signature against a safe slice-producing body.
        let definitions=format!("struct Record0<'a>{{bytes:&'a[u8]}} struct Borrowed<'c0>{{capture{capture}:&'c0 String}} impl<'c0> Borrowed<'c0>{{{signature} Record0{{bytes:self.capture{capture}.as_bytes()}} }} }}");
        let positive=definitions.clone()+&format!(r#"
fn main() {{
    let owner=String::from("bytes");let returned;
    {{let callback=Borrowed{{capture{capture}:&owner}};returned=callback.call();}}
    assert_eq!(returned.bytes.as_ptr(),owner.as_bytes().as_ptr());
    assert_eq!(returned.bytes,b"bytes");
}}
"#);
        let negative=definitions+&format!(r#"
fn main() {{
    let escaped;
    {{let owner=String::from("bytes");let callback=Borrowed{{capture{capture}:&owner}};escaped=callback.call();}}
    assert_eq!(escaped.bytes,b"bytes");
}}
"#);
        let directory=std::env::temp_dir().join(format!("tsrust-borrowed-owner-capture-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());
        for (name,source) in [("positive",positive),("negative",negative)] {
            let input=directory.join(format!("{name}.rs"));std::fs::write(&input,source).unwrap();
            for level in [0,3] {
                let output=directory.join(format!("{name}-{level}"));
                let built=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024",&format!("-Copt-level={level}")]).arg(&input).arg("-o").arg(&output).output().unwrap();
                if name=="negative" {
                    assert!(!built.status.success(),"borrowed capture escaped its byte owner");
                    assert!(String::from_utf8_lossy(&built.stderr).contains("does not live long enough"));
                } else {
                    assert!(built.status.success(),"{}",String::from_utf8_lossy(&built.stderr));
                    let run=std::process::Command::new(output).output().unwrap();
                    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
                }
            }
        }
    }
    #[test]
    fn captured_native_view_return_preserves_outer_origin() {
        let source = "type Window={};function captured(unused:int,input:Window):Window{const alias=input;const callback=():Window=>alias;return callback();}function main(){}";
        let formatted = crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut parsed = crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let checked = crate::analysis::analyze(parsed).unwrap();
        let function = checked.functions.iter().find(|function| function.name == "captured").unwrap();
        let origins = function.native_returns.as_ref().expect("captured return must retain an input lifetime");
        assert_eq!(origins.len(), 1);
        assert_eq!(origins[0].root, 1);
    }
    #[test]
    fn returned_native_views_compile_with_distinct_input_lifetimes() {
        let source="type Window={};type WriteWindow={};type Shelf={item:Window;};type Pair={one:Window;two:Window;};type Outcome=Result<Window,int>;type Double=Result<Window,Window>;function capturedStatus(input:ReadOnly<Outcome>):boolean{const callback=():boolean=>{match(input){Ok(_) => {return true;},Err(_) => {return false;}}};return callback();}function capturedOutcome(input:Outcome):Outcome{const callback=():Outcome=>input;return callback();}function capturedPair(input:Pair):Window{const callback=():Window=>input.one;return callback();}function capturedWrite(input:WriteWindow):WriteWindow{const callback=():WriteWindow=>input;return callback();}function captured(input:Window):Window{const callback=():Window=>input;return callback();}function identity(input:Window):Window{return input;}function relay(input:Window):Window{return identity(input);}function arrowChoose(left:Window,right:Window,flag:boolean):Window{const callback=(first:Window,second:Window):Window=>{if(flag){return first;}return second;};return callback(left,right);}function arrowRelay(input:Window):Window{const identity=(value:Window):Window=>value;return identity(input);}function savedRelay(input:Window):Window{const saved=identity;return saved(input);}function outcomeIdentity(input:Outcome):Outcome{return input;}function capturedDouble(input:Double):Double{const callback=():Double=>input;return callback();}function pairFirst(input:Pair):Window{return input.one;}function unwrap(input:Shelf):Window{return input.item;}function writeIdentity(input:WriteWindow):WriteWindow{return input;}function arrayIdentity(input:Window[]):Window[]{return input;}function first(left:Window,right:Window):Window{return left;}function choose(left:Window,right:Window,flag:boolean):Window{if(flag){return left;}return right;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        parsed.records[1].native_type=Some(crate::native_signatures::NativeTypeId::ByteViewMut);
        let checked=crate::analysis::analyze(parsed).unwrap();
        let captured_status=checked.functions.iter().position(|function|function.name=="capturedStatus").unwrap();
        let captured_outcome=checked.functions.iter().position(|function|function.name=="capturedOutcome").unwrap();
        let captured_pair=checked.functions.iter().position(|function|function.name=="capturedPair").unwrap();
        let captured_write=checked.functions.iter().position(|function|function.name=="capturedWrite").unwrap();
        let captured=checked.functions.iter().position(|function|function.name=="captured").unwrap();
        let relay=checked.functions.iter().position(|function|function.name=="relay").unwrap();
        let arrow_choose=checked.functions.iter().position(|function|function.name=="arrowChoose").unwrap();
        let arrow_relay=checked.functions.iter().position(|function|function.name=="arrowRelay").unwrap();
        let saved_relay=checked.functions.iter().position(|function|function.name=="savedRelay").unwrap();
        let captured_double=checked.functions.iter().position(|function|function.name=="capturedDouble").unwrap();
        let double=checked.functions[captured_double].result.union().unwrap();
        let [double_ok,double_err]=crate::result_identity::variants(&checked.records,&checked.unions[double.index()].members).unwrap();
        let double_ok_tag=checked.unions[double.index()].members.iter().position(|member|*member==Type::Record(double_ok)).unwrap();
        let double_id=double.index();
        let double_err_tag=checked.unions[double_id].members.iter().position(|member|*member==Type::Record(double_err)).unwrap();
        let double_origins=checked.functions[captured_double].native_returns.as_ref().unwrap();
        assert_eq!(double_origins.len(),2);
        assert_ne!(double_origins[0].source,double_origins[1].source);
        let outcome_identity=checked.functions.iter().position(|function|function.name=="outcomeIdentity").unwrap();
        let outcome=checked.functions[outcome_identity].result.union().unwrap();
        let members=&checked.unions[outcome.index()].members;
        let [ok,err]=crate::result_identity::variants(&checked.records,members).unwrap();
        let ok_tag=members.iter().position(|member|*member==Type::Record(ok)).unwrap();
        let outcome_id=outcome.index();
        let err_tag=members.iter().position(|member|*member==Type::Record(err)).unwrap();
        let pair_first=checked.functions.iter().position(|function|function.name=="pairFirst").unwrap();
        let unwrap=checked.functions.iter().position(|function|function.name=="unwrap").unwrap();
        let write_identity=checked.functions.iter().position(|function|function.name=="writeIdentity").unwrap();
        let array_identity=checked.functions.iter().position(|function|function.name=="arrayIdentity").unwrap();
        let first=checked.functions.iter().position(|function|function.name=="first").unwrap();
        let choose=checked.functions.iter().position(|function|function.name=="choose").unwrap();
        let mut rust=crate::emit::emit(&checked).replace("fn main()", "fn unused_source_main()");
        let base=rust.clone();
        rust.push_str(&format!(r#"
fn double_success<'a,'b>(owner:&'a[u8],_other:&'b[u8])->Union{double_id}<Record{double_ok}<Record0<'a>>,Record{double_err}<Record0<'b>>> {{
    Union{double_id}::V{double_ok_tag}(Record{double_ok}{{field0:Record0{{bytes:owner}}}})
}}
fn double_error<'a,'b>(_other:&'a[u8],owner:&'b[u8])->Union{double_id}<Record{double_ok}<Record0<'a>>,Record{double_err}<Record0<'b>>> {{
    Union{double_id}::V{double_err_tag}(Record{double_err}{{field0:Record0{{bytes:owner}}}})
}}
fn main() {{
    let owner=vec![1u8,2,3];
    let selected_double_error;
    {{let short=vec![9u8];let result=function{captured_double}(double_error(&short,&owner));selected_double_error=match result {{Union{double_id}::V{double_err_tag}(value)=>value.field0,_=>panic!("unexpected success")}};}}
    assert_eq!(selected_double_error.bytes.as_ptr(),owner.as_ptr());
    let selected_double;
    {{let short=vec![9u8];let result=function{captured_double}(double_success(&owner,&short));selected_double=match result {{Union{double_id}::V{double_ok_tag}(value)=>value.field0,_=>panic!("unexpected error")}};}}
    assert_eq!(selected_double.bytes.as_ptr(),owner.as_ptr());
    assert_eq!(function{captured}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    assert_eq!(function{relay}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    assert_eq!(function{arrow_relay}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    assert_eq!(function{saved_relay}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    let returned;
    {{ let short=vec![9u8]; returned=function{first}(Record0{{bytes:&owner}}, &Record0{{bytes:&short}}); }}
    assert_eq!(returned.bytes.as_ptr(),owner.as_ptr());
    let captured_field;
    {{ let short=vec![8u8];captured_field=function{captured_pair}(Record3{{field0:Record0{{bytes:&owner}},field1:Record0{{bytes:&short}}}}); }}
    assert_eq!(captured_field.bytes.as_ptr(),owner.as_ptr());
    let field;
    {{ let short=vec![8u8];field=function{pair_first}(Record3{{field0:Record0{{bytes:&owner}},field1:Record0{{bytes:&short}}}}); }}
    assert_eq!(field.bytes.as_ptr(),owner.as_ptr());
    let wrapped=function{unwrap}(Record2{{field0:Record0{{bytes:&owner}}}});
    assert_eq!(wrapped.bytes.as_ptr(),owner.as_ptr());
    let values=function{array_identity}(RecordArray0::from([Record0{{bytes:&owner}}]));
    assert_eq!(values[0].bytes.as_ptr(),owner.as_ptr());
    assert!(function{captured_status}(Union{outcome_id}::V{ok_tag}(Record{ok}{{field0:Record0{{bytes:&owner}}}}).tsr_view()));
    assert!(!function{captured_status}(Union{outcome_id}::V{err_tag}(Record{err}{{field0:17}}).tsr_view()));
    let captured_outcome=function{captured_outcome}(Union{outcome_id}::V{ok_tag}(Record{ok}{{field0:Record0{{bytes:&owner}}}}));
    match captured_outcome {{ Union{outcome_id}::V{ok_tag}(value)=>assert_eq!(value.field0.bytes.as_ptr(),owner.as_ptr()),_=>panic!("captured success lost") }}
    let captured_error=function{captured_outcome}(Union{outcome_id}::V{err_tag}(Record{err}{{field0:17}}));
    match captured_error {{ Union{outcome_id}::V{err_tag}(value)=>assert_eq!(value.field0,17),_=>panic!("captured error lost") }}
    let outcome=function{outcome_identity}(Union{outcome_id}::V{ok_tag}(Record{ok}{{field0:Record0{{bytes:&owner}}}}));
    match outcome {{ Union{outcome_id}::V{ok_tag}(value)=>assert_eq!(value.field0.bytes.as_ptr(),owner.as_ptr()),_=>panic!("unexpected error") }}
    let mut writable=vec![4u8,5];
    {{ let returned=function{write_identity}(Record1{{bytes:&mut writable}});returned.bytes[0]=8; }}
    assert_eq!(writable[0],8);
    {{ let returned=function{captured_write}(Record1{{bytes:&mut writable}});returned.bytes[1]=6; }}
    assert_eq!(writable[1],6);
    let other=vec![7u8];
    assert_eq!(function{arrow_choose}(Record0{{bytes:&owner}},Record0{{bytes:&other}},false).bytes.as_ptr(),other.as_ptr());
    let selected=function{choose}(Record0{{bytes:&owner}},Record0{{bytes:&other}},false);
    assert_eq!(selected.bytes.as_ptr(),other.as_ptr());
}}
"#));
        let directory=std::env::temp_dir().join(format!("tsrust-returned-view-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());
        let input=directory.join("returns.rs");std::fs::write(&input,rust).unwrap();
        let rejected=directory.join("escape.rs");
        std::fs::write(&rejected,base+&format!(r#"
fn main() {{
    let owner=vec![1u8];
    let escaped;
    {{ let short=vec![2u8]; escaped=function{choose}(Record0{{bytes:&owner}},Record0{{bytes:&short}},false); }}
    assert_eq!(escaped.bytes[0],2);
}}
"#)).unwrap();
        let rejected_output=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024"]).arg(&rejected).arg("-o").arg(directory.join("escape")).output().unwrap();
        assert!(!rejected_output.status.success(),"escaping selected input unexpectedly compiled");
        assert!(String::from_utf8_lossy(&rejected_output.stderr).contains("does not live long enough"));
        let captured_escape=directory.join("captured-escape.rs");
        std::fs::write(&captured_escape,crate::emit::emit(&checked).replace("fn main()", "fn unused_source_main()")+&format!(r#"
fn main() {{
    let escaped;
    {{ let short=vec![2u8]; escaped=function{captured}(Record0{{bytes:&short}}); }}
    assert_eq!(escaped.bytes[0],2);
}}
"#)).unwrap();
        let captured_rejected=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024"]).arg(&captured_escape).arg("-o").arg(directory.join("captured-escape")).output().unwrap();
        assert!(!captured_rejected.status.success(),"captured view escaped its original owner");
        assert!(String::from_utf8_lossy(&captured_rejected.stderr).contains("does not live long enough"));

        for (tag,record) in [(double_ok_tag,double_ok),(double_err_tag,double_err)] {
            let rejected=directory.join(format!("variant-escape-{tag}.rs"));
            std::fs::write(&rejected,crate::emit::emit(&checked).replace("fn main()", "fn unused_source_main()")+&format!(r#"
fn main() {{
    let escaped;
    {{ let short=vec![2u8]; escaped=match function{captured_double}(Union{double_id}::V{tag}(Record{record}{{field0:Record0{{bytes:&short}}}})) {{Union{double_id}::V{tag}(value)=>value.field0,_=>panic!("wrong variant")}}; }}
    assert_eq!(escaped.bytes[0],2);
}}
"#)).unwrap();
            let failed=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024"]).arg(&rejected).arg("-o").arg(directory.join(format!("variant-escape-{tag}"))).output().unwrap();
            assert!(!failed.status.success(),"variant {tag} escaped its byte owner");
            assert!(String::from_utf8_lossy(&failed.stderr).contains("does not live long enough"));
        }
        for level in [0,3] {
            let output=directory.join(format!("returns-{level}"));
            let compiled=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024",&format!("-Copt-level={level}")]).arg(&input).arg("-o").arg(&output).output().unwrap();
            assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
            assert!(std::process::Command::new(output).status().unwrap().success());
        }
    }

}

#[cfg(test)]
#[test]
fn native_view_storage_shapes_follow_concrete_and_scoped_family_environments() {
    let source="type Window={};type Holder={value:int;seed:int;};function save(view:Window){return ()=>view;}function main(){}";
    let formatted=crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
    let mut checked=crate::analysis::analyze(parsed).unwrap();
    let result=checked.closures[0].function.result;
    let structural=checked.callable_shapes.intern([],result,Some(crate::syntax::CallbackKind::OwnedOnce));
    let callback=crate::analysis::CallbackContract{structural,parameter_names:vec![],params:vec![],result,kind:crate::syntax::CallbackKind::OwnedOnce};
    let ordinary=checked.stored_contracts.intern(&checked.callable_shapes,&callback,0).unwrap();
    let scoped=checked.stored_contracts.intern_scoped_native(&checked.callable_shapes,&callback,0,[0],&checked.closures,crate::Span::default()).unwrap().unwrap();
    checked.records[1].fields[0].ty=Some(scoped);
    let shapes=Shapes::new(&checked);
    assert!(shapes.contains(Type::Closure(0)));
    assert!(shapes.contains(scoped));
    assert!(shapes.contains(Type::Record(1)));
    assert!(!shapes.contains(ordinary));
    let Type::StoredCallable(scoped_id)=scoped else {unreachable!()};
    assert!(!shapes.contains(Type::Closure(checked.closures.len()+scoped_id)));
    // A cycle through record storage must terminate and keep its native seed.
    let capture=checked.closures[0].captures[0].inner;
    checked.closures[0].function.bindings[capture].ty=Type::Record(1);
    checked.records[1].fields[1].ty=Some(Type::Record(0));
    assert!(Shapes::new(&checked).contains(scoped));
}
