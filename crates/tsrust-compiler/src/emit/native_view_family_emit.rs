//! Scoped owning native family ABI; source construction remains guarded.
use super::*;

use crate::analysis::native_view_loans::{LifetimeSource,Loan,Segment,VariantGuard};
use std::collections::BTreeMap;

fn captured_source(capture:usize,path:&[Segment],guards:&[VariantGuard])->LifetimeSource {
    let prefix=Segment::Field(format!("capture{capture}"));
    let mut path=path.to_vec();path.insert(0,prefix.clone());
    let guards=guards.iter().map(|guard|{let mut guard=guard.clone();guard.path.insert(0,prefix.clone());guard}).collect();
    LifetimeSource::carried(path,guards)
}

impl Emitter<'_> {
    pub(super) fn emit_native_view_family(&self,output:&mut String,id:usize,layout:&crate::native_view_callable_storage::Family,contract:&crate::analysis::CallbackContract)->Result<(),crate::Diagnostic> {
        let unsupported=|message:&str|crate::Diagnostic::new("native-view-family-emission",message,crate::Span::default());
        crate::native_view_callable_storage::validate_abi(layout,contract,&self.program.closures,&self.program.error_effects,crate::Span::default())?;
        let mut inputs=BTreeMap::new();
        let mut lifetimes=Vec::new();
        let argument_offset=layout.alternatives.len();
        let mut argument_names=Vec::new();
        let mut argument_slots=vec![0;contract.params.len()];
        let mut returns=Vec::<Loan>::new();
        for (arm,alternative) in layout.alternatives.iter().enumerate() {
            let closure=self.program.closures.get(alternative.origin).ok_or_else(||unsupported("Unknown native callable family environment."))?;
            if self.native_captures.lifetimes[alternative.origin]!=alternative.slots.len() {
                return Err(unsupported("Native callable family capture lifetimes do not match its carried-view slots."));
            }
            let checked_slots=crate::native_view_callable_storage::carried_slots(closure).unwrap_or_default();
            if checked_slots!=alternative.slots {
                return Err(unsupported("Native callable family capture paths and access must match the checked environment."));
            }
            for (slot_index,slot) in alternative.slots.iter().enumerate() {
                let name=format!("'native_family{arm}_{slot_index}");
                inputs.insert((arm,captured_source(slot.capture,&slot.path,&slot.guards)),name.clone());
                lifetimes.push(name);
            }
            // Alternative closures have different binding indices. Invocation
            // parameters share one positional ABI, outside the capture-root range.
            for (position,&binding) in closure.parameters.iter().enumerate() {
                let root=argument_offset+position;
                for loan in closure.function.bindings[binding].native_view_loans.as_deref().into_iter().flatten()
                    .chain(closure.function.native_returns.as_deref().into_iter().flatten().filter(|loan|loan.root==binding)) {
                    let key=(root,loan.source.clone());
                    if let std::collections::btree_map::Entry::Vacant(entry)=inputs.entry(key) {
                        let name=format!("'native_argument{position}_{}",argument_slots[position]);
                        argument_slots[position]+=1;argument_names.push(name.clone());entry.insert(name);
                    }
                }
            }
            for loan in closure.function.native_returns.as_deref().into_iter().flatten() {
                if let Some(position)=closure.parameters.iter().position(|&binding|binding==loan.root) {
                    let mut returned=loan.clone();returned.root=argument_offset+position;returns.push(returned);continue;
                }
                let mut returned=loan.clone();
                returned.root=arm;
                returned.source=match &loan.source {
                    LifetimeSource::Carried(path)=>captured_source(loan.root,path,&[]),
                    LifetimeSource::VariantCarried{path,guards}=>captured_source(loan.root,path,guards),
                    LifetimeSource::Storage=>return Err(unsupported("A consuming native family cannot return a borrow of destroyed capture storage.")),
                };
                returns.push(returned);
            }
        }
        let plan=native_view_lifetime_emit::Lifetimes::with_named_sources(inputs,&returns)?;
        let name=format!("NativeViewCallable{id}");
        let generics=format!("<{}>",lifetimes.join(", "));
        let mut generated=String::new();
        writeln!(generated,"enum {name}{generics} {{").unwrap();
        for (arm,alternative) in layout.alternatives.iter().enumerate() {
            let environment=self.native_view_environment_type(alternative.origin,Some(arm),&plan,&[],&[]);
            writeln!(generated,"V{}({environment}),",alternative.origin).unwrap();
        }
        generated.push_str("}\n");
        argument_names.extend(plan.declarations.iter().cloned());
        let method_generics=lifetime_parameters(&argument_names);
        let bounds=if plan.bounds.is_empty(){String::new()}else{format!(" where {}",plan.bounds.join(", "))};
        let parameters=contract.params.iter().enumerate().map(|(position,&parameter)|format!(",argument{position}:{}",self.native_view_family_argument_type(parameter,argument_offset+position,&plan))).collect::<String>();
        let arguments=(0..contract.params.len()).map(|position|format!("argument{position}")).collect::<Vec<_>>().join(", ");
        let result=self.native_view_signature_type(contract.result,None,&plan);
        // A family with a throwing alternative uses the error layout planned for its
        // identity; alternatives that cannot throw are adapted to it like ordinary families.
        let result=match self.result_storage.borrow().get(&id) {Some((error,_))=>format!("Result<{result},{error}>"),None=>result};
        // A reusable family is called through a borrow of its environment.
        let (receiver,pattern)=match layout.invocation {Mode::Consume=>("self","mut value"),Mode::Read=>("&self","value"),_=>("&mut self","value")};
        writeln!(generated,"impl{generics} {name}{generics} {{fn call{method_generics}({receiver}{parameters})->{result}{bounds} {{match self {{").unwrap();
        for alternative in &layout.alternatives {
            writeln!(generated,"Self::V{}({pattern})=>{},",alternative.origin,self.stored_origin_result(id,alternative.origin,format!("value.call({arguments})"))).unwrap();
        }
        generated.push_str("}}}\n");
        output.push_str(&generated);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_view_once_family_emits_real_alternative_lifetimes() {
        for mutable in [false,true] {assert_family(mutable);}
    }

    fn assert_family(mutable:bool) {
        let source="type Window={};type Holder={read:Once<Function<Window>>;marker:int;};function first(view:Window){return ()=>view;}function second(view:Window){return ()=>view;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(if mutable {crate::native_signatures::NativeTypeId::ByteViewMut}else{crate::native_signatures::NativeTypeId::ByteView});
        let mut checked=crate::analysis::analyze(parsed).unwrap();
        let result=checked.closures[0].function.result;
        let structural=checked.callable_shapes.intern([],result,Some(crate::syntax::CallbackKind::OwnedOnce));
        let contract=crate::analysis::CallbackContract {structural,parameter_names:vec![],params:vec![],result,kind:crate::syntax::CallbackKind::OwnedOnce};
        let layout=crate::native_view_callable_storage::family_slots(&contract,&checked.callable_shapes,[0,1],&checked.closures,crate::Span::default()).unwrap().unwrap();
        let emitter=emitter_for(&checked,"");
        let mut family=String::new();
        emitter.emit_native_view_family(&mut family,7,&layout,&contract).unwrap();
        assert!(!family.contains("Box<") && !family.contains("clone") && !family.contains("unsafe"));
        let mut invalid=layout.clone();invalid.invocation=Mode::Read;
        let mut unchanged="sentinel".to_owned();
        assert!(emitter.emit_native_view_family(&mut unchanged,8,&invalid,&contract).is_err());
        assert_eq!(unchanged,"sentinel");
        invalid=layout.clone();invalid.alternatives[0].slots.clear();
        assert!(emitter.emit_native_view_family(&mut unchanged,8,&invalid,&contract).is_err());
        assert_eq!(unchanged,"sentinel");
        invalid=layout.clone();
        invalid.alternatives[0].slots[0].access=if mutable {crate::native_return_borrows::BorrowAccess::ReadOnly}else{crate::native_return_borrows::BorrowAccess::Exclusive};
        assert!(emitter.emit_native_view_family(&mut unchanged,8,&invalid,&contract).is_err());
        assert_eq!(unchanged,"sentinel");
        let declaration=checked.callback_contracts.len();checked.callback_contracts.push(crate::analysis::CallbackContract{structural:contract.structural,parameter_names:contract.parameter_names.clone(),params:contract.params.clone(),result:contract.result,kind:contract.kind});
        let scoped=checked.stored_contracts.intern_scoped_native(&checked.callable_shapes,&contract,declaration,[0,1],&checked.closures,crate::Span::default()).unwrap().unwrap();
        let Type::StoredCallable(scoped)=scoped else {unreachable!()};
        let holder=checked.scoped_records.materialize(1,vec![Type::StoredCallable(scoped),Type::Int],&mut checked.records);
        let emitter=emitter_for(&checked,"");
        let mut inputs=BTreeMap::new();let mut returns=Vec::new();let mut names=Vec::new();
        for alternative in &layout.alternatives {
            for slot in &alternative.slots {
                let name=format!("'scope{}",names.len());names.push(name.clone());
                let mut path=vec![Segment::Field("read".into()),Segment::CallableAlternative(alternative.origin),Segment::Field(format!("capture{}",slot.capture))];path.extend(slot.path.iter().cloned());
                let source=LifetimeSource::carried(path.clone(),vec![]);inputs.insert((0,source.clone()),name);
                returns.push(Loan{ whole: false,root:0,owner_fields:vec![],source,access:slot.access,creation:crate::Span::default(),retained:true,path,guards:vec![]});
            }
        }
        let plan=native_view_lifetime_emit::Lifetimes::with_named_sources(inputs,&returns).unwrap();
        let input=emitter.native_view_signature_type(Type::Record(holder),Some(0),&plan);
        let result=emitter.native_view_signature_type(Type::Record(holder),None,&plan);
        let relay=format!("fn relay_family<{}>(value:{input})->{result}{{value}}",names.join(","));
        let mut generated=crate::emit::emit(&checked);
        generated.push_str(&relay);
        let main=generated.find("fn main()").unwrap();
        let opening=main+generated[main..].find('{').unwrap()+1;
        generated.insert_str(opening,"family_probe();");
        let probe=if mutable {
            "fn family_probe(){for flag in [true,false]{let mut first=vec![1u8];let mut second=vec![7u8,8];let first_ptr=first.as_ptr();let second_ptr=second.as_ptr();{let selected=if flag{NativeViewCallable7::V0(function0(Record0{bytes:&mut first}))}else{NativeViewCallable7::V1(function1(Record0{bytes:&mut second}))};let result=selected.call();assert_eq!(result.bytes.as_ptr(),if flag{first_ptr}else{second_ptr});result.bytes[0]+=10;}assert_eq!(first[0],if flag{11}else{1});assert_eq!(second[0],if flag{7}else{17});}}"
        } else {"fn family_probe(){let first=vec![1u8];let second=vec![7u8,8];for flag in [true,false]{let selected=if flag{NativeViewCallable7::V0(function0(Record0{bytes:&first}))}else{NativeViewCallable7::V1(function1(Record0{bytes:&second}))};let result=selected.call();assert_eq!(result.bytes.as_ptr(),if flag{first.as_ptr()}else{second.as_ptr()});assert_eq!(result.bytes.len(),if flag{1}else{2});}}"};

        let owner=if mutable {"let mut owner=vec![1u8];"}else{"let owner=vec![1u8];"};
        let reference=if mutable {"&mut owner"}else{"&owner"};
        let negative=format!("fn invalid_escape(){{let result;{{{owner}let selected=NativeViewCallable7::V0(function0(Record0{{bytes:{reference}}}));result=selected.call();}}assert_eq!(result.bytes.len(),1);}}");
        let repeated=format!("fn invalid_reuse(){{{owner}let selected=NativeViewCallable7::V0(function0(Record0{{bytes:{reference}}}));let first=selected.call();let second=selected.call();assert_eq!(first.bytes.len(),second.bytes.len());}}");
        let mut rejections=vec![(negative,"does not live long enough"),(repeated,"moved value")];
        if mutable {
            rejections.push(("fn invalid_mutation(){let mut owner=vec![1u8];let selected=NativeViewCallable7::V0(function0(Record0{bytes:&mut owner}));let result=selected.call();owner[0]=2;assert_eq!(result.bytes[0],1);}".into(),"cannot borrow"));
        }
        let name=format!("NativeViewCallable{scoped}");
        let probe=probe.replace("NativeViewCallable7",&name).replace("selected.call()","relay_family(Record1{field0:selected,field1:3}).field0.call()");
        let rejections=rejections.into_iter().map(|(source,message)|(source.replace("NativeViewCallable7",&name).replace("selected.call()","relay_family(Record1{field0:selected,field1:3}).field0.call()"),message)).collect();
        compile_family(&generated,"",&probe,rejections,if mutable {"mutable"}else{"readonly"});
    }

    fn compile_family(generated:&str,family:&str,probe:&str,rejections:Vec<(String,&str)>,label:&str) {
        let directory=std::env::temp_dir().join(format!("tsrust-native-view-family-{label}-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());
        let input=directory.join("family.rs");
        std::fs::write(&input,format!("{generated}\n{family}\n{probe}")).unwrap();
        for level in ["0","3"] {
            let output=directory.join(format!("family-{level}"));
            let compiled=std::process::Command::new("rustc").arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg(&input).arg("-o").arg(&output).output().unwrap();
            assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
            assert!(std::process::Command::new(output).status().unwrap().success());
        }
        for (negative,diagnostic) in rejections {
            std::fs::write(&input,format!("{generated}\n{family}\n{probe}\n{negative}")).unwrap();
            let rejected=std::process::Command::new("rustc").arg("--edition=2024").arg(&input).arg("-o").arg(directory.join("rejected")).output().unwrap();
            assert!(!rejected.status.success());
            assert!(String::from_utf8_lossy(&rejected.stderr).contains(diagnostic),"{}",String::from_utf8_lossy(&rejected.stderr));
        }
    }
    #[test]
    fn native_view_family_result_keeps_success_lifetimes_and_error_branch() {
        for mutable in [false,true] {assert_result_family(mutable);}
    }

    fn assert_result_family(mutable:bool) {
        let source="type Window={};function first(view:Window,success:boolean){return ():Result<Window,int>=>{if(success){return Ok<Window,int>(view);}const discarded=view;return Err<Window,int>(17);};}function second(view:Window,success:boolean){return ():Result<Window,int>=>{if(success){return Ok<Window,int>(view);}const discarded=view;return Err<Window,int>(17);};}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(if mutable {crate::native_signatures::NativeTypeId::ByteViewMut}else{crate::native_signatures::NativeTypeId::ByteView});
        let mut checked=crate::analysis::analyze(parsed).unwrap();
        let result=checked.closures[0].function.result;
        let structural=checked.callable_shapes.intern([],result,Some(crate::syntax::CallbackKind::OwnedOnce));
        let contract=crate::analysis::CallbackContract{structural,parameter_names:vec![],params:vec![],result,kind:crate::syntax::CallbackKind::OwnedOnce};
        let layout=crate::native_view_callable_storage::family_slots(&contract,&checked.callable_shapes,[0,1],&checked.closures,crate::Span::default()).unwrap().unwrap();
        let emitter=emitter_for(&checked,"");
        let mut family=String::new();emitter.emit_native_view_family(&mut family,7,&layout,&contract).unwrap();
        let Type::Union(union)=result else {panic!("Result union expected")};
        let members=&checked.unions[union.index()].members;
        let ok=members.iter().position(|member|matches!(member,Type::Record(id) if checked.records[*id].fields[0].ty==Some(Type::Record(0)))).unwrap();
        let err=1-ok;
        let union=union.index();
        let first_function=checked.functions.iter().position(|function|function.name=="first" && !function.template).unwrap();
        let second_function=checked.functions.iter().position(|function|function.name=="second" && !function.template).unwrap();
        let Type::Closure(first_origin)=checked.functions[first_function].result else {panic!("factory closure expected")};
        let Type::Closure(second_origin)=checked.functions[second_function].result else {panic!("factory closure expected")};
        let reference=if mutable {"&mut"}else{"&"};
        let write=if mutable {"value.field0.bytes[0]+=10;"}else{""};
        let probe=format!("fn family_probe(){{for flag in [true,false]{{for success in [true,false]{{let mut first=vec![1u8];let mut second=vec![7u8,8];let first_ptr=first.as_ptr();let second_ptr=second.as_ptr();{{let selected=if flag{{NativeViewCallable7::V{first_origin}(function{first_function}(Record0{{bytes:{reference} first}},success))}}else{{NativeViewCallable7::V{second_origin}(function{second_function}(Record0{{bytes:{reference} second}},success))}};match selected.call(){{Union{union}::V{ok}(value)=>{{assert!(success);assert_eq!(value.field0.bytes.as_ptr(),if flag{{first_ptr}}else{{second_ptr}});{write}}},Union{union}::V{err}(value)=>{{assert!(!success);assert_eq!(value.field0,17);}}}}}}assert_eq!(first[0],if {mutable}&&success&&flag{{11}}else{{1}});assert_eq!(second[0],if {mutable}&&success&&!flag{{17}}else{{7}});}}}}}}");
        let extract=format!("match selected.call(){{Union{union}::V{ok}(value)=>value.field0,_=>panic!(\"expected success\")}}");
        let negative=format!("fn invalid_escape(){{let result;{{let mut owner=vec![1u8];let selected=NativeViewCallable7::V{first_origin}(function{first_function}(Record0{{bytes:{reference} owner}},true));result={extract};}}assert_eq!(result.bytes.len(),1);}}");
        let mut generated=crate::emit::emit(&checked);let main=generated.find("fn main()").unwrap();let opening=main+generated[main..].find('{').unwrap()+1;generated.insert_str(opening,"family_probe();");
        compile_family(&generated,&family,&probe,vec![(negative,"does not live long enough")],if mutable {"result-mut"}else{"result-read"});
    }

    #[test]
    fn native_view_family_record_keeps_field_lifetimes_independent() {
        for mutable in [false,true] {assert_record_family(mutable);}
    }

    fn assert_record_family(mutable:bool) {
        let source="type Window={};type Pair={one:Window;two:Window;};function first(pair:Pair){return ()=>pair;}function second(pair:Pair){return ()=>pair;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(if mutable {crate::native_signatures::NativeTypeId::ByteViewMut}else{crate::native_signatures::NativeTypeId::ByteView});
        let mut checked=crate::analysis::analyze(parsed).unwrap();
        let first=checked.functions.iter().position(|function|function.name=="first" && !function.template).unwrap();
        let second=checked.functions.iter().position(|function|function.name=="second" && !function.template).unwrap();
        let Type::Closure(left)=checked.functions[first].result else {panic!("factory closure expected")};
        let Type::Closure(right)=checked.functions[second].result else {panic!("factory closure expected")};
        let result=checked.closures[left].function.result;
        let structural=checked.callable_shapes.intern([],result,Some(crate::syntax::CallbackKind::OwnedOnce));
        let contract=crate::analysis::CallbackContract{structural,parameter_names:vec![],params:vec![],result,kind:crate::syntax::CallbackKind::OwnedOnce};
        let layout=crate::native_view_callable_storage::family_slots(&contract,&checked.callable_shapes,[left,right],&checked.closures,crate::Span::default()).unwrap().unwrap();
        assert!(layout.alternatives.iter().all(|alternative|alternative.slots.len()==2));
        let emitter=emitter_for(&checked,"");
        let mut family=String::new();emitter.emit_native_view_family(&mut family,7,&layout,&contract).unwrap();
        let reference=if mutable {"&mut"}else{"&"};
        let write=if mutable {"retained.bytes[0]+=10;"}else{""};
        // Retain only field one after field two's independent owner ends.
        let probe=format!("fn family_probe(){{for flag in [true,false]{{let mut owner=vec![1u8];let pointer=owner.as_ptr();{{let retained;{{let mut short=vec![7u8,8];let short_pointer=short.as_ptr();let pair=Record1{{field0:Record0{{bytes:{reference} owner}},field1:Record0{{bytes:{reference} short}}}};let selected=if flag{{NativeViewCallable7::V{left}(function{first}(pair))}}else{{NativeViewCallable7::V{right}(function{second}(pair))}};let result=selected.call();assert_eq!(result.field1.bytes.as_ptr(),short_pointer);retained=result.field0;}}assert_eq!(retained.bytes.as_ptr(),pointer);{write}}}assert_eq!(owner[0],if {mutable}{{11}}else{{1}});}}}}");
        let negative=format!("fn invalid_escape(){{let mut survivor=vec![1u8];let retained;{{let mut short=vec![7u8];let selected=NativeViewCallable7::V{left}(function{first}(Record1{{field0:Record0{{bytes:{reference} survivor}},field1:Record0{{bytes:{reference} short}}}}));retained=selected.call().field1;}}assert_eq!(retained.bytes.len(),1);}}");
        let mut generated=crate::emit::emit(&checked);let main=generated.find("fn main()").unwrap();let opening=main+generated[main..].find('{').unwrap()+1;generated.insert_str(opening,"family_probe();");
        compile_family(&generated,&family,&probe,vec![(negative,"does not live long enough")],if mutable {"record-mut"}else{"record-read"});
    }

    #[test]
    fn recursive_view_callback_native_chain_keeps_identity_and_bounded_drop() {
        for (callback_only, nullable) in [(false,false),(true,false),(false,true),(true,true)] {
        let source = "type Window={};type Holder={data:Window;read:Once<Function<Window>>;next?:Holder;};function build(first:Window,second:Window):Holder{return {data:first,read:()=>second};}function other(first:Window,second:Window):Holder{return {data:first,read:()=>second};}function take(holder:Holder){return holder.read;}function main(){}";
        let source = if callback_only { source.replace("data:Window;", "data:Once<Function<Window>>;").replace("data:first,", "data:()=>first,") } else { source.to_owned() };
        let source = if nullable { source.replace("read:Once<Function<Window>>;", "read?:Once<Function<Window>>;") } else { source };
        let source = crate::formatting::format(&source).unwrap();
        assert_eq!(crate::formatting::format(&source).unwrap(), source);
        let mut parsed = crate::syntax::parse(&source).unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let checked = crate::analysis::analyze(parsed).unwrap();
        let factory = checked.functions.iter().position(|function| function.name == "build" && !function.template).unwrap();
        let Type::Record(record) = checked.functions[factory].result else { panic!("concrete recursive record") };
        let other = checked.functions.iter().position(|function| function.name == "other" && !function.template).unwrap();
        assert_eq!(checked.functions[other].result, Type::Record(record));
        let callback_type = checked.records[record].fields[1].ty.unwrap();
        let payload = if let Type::Union(id) = callback_type { checked.unions[id.index()].members.iter().copied().find(|ty| matches!(ty, Type::StoredCallable(_))).unwrap() } else { callback_type };
        let Type::StoredCallable(family) = payload else { unreachable!() };
        let invoke = |holder: &str| {
            let call = format!("function{take}({holder})", take = checked.functions.iter().position(|function| function.name == "take").unwrap());
            if let Type::Union(id) = callback_type {
                let tag = checked.unions[id.index()].members.iter().position(|ty| *ty == payload).unwrap();
                format!("match {call}{{Union{}::V{tag}(callback)=>callback.call(),_=>panic!(\"missing callback\")}}", id.index())
            } else { format!("{call}.call()") }
        };
        assert_eq!(checked.stored_contracts.origins(family).unwrap().len(), 2);
        let invoke_root = invoke("root");
        let invoke_holder = invoke("holder");
        let union = checked.records[record].fields[2].ty.unwrap().union().unwrap();
        let tag = checked.unions[union.index()].members.iter().position(|ty| *ty == Type::Record(record)).unwrap();
        let union = union.index();
        let make = format!("function{factory}(Record0{{bytes:&first}},Record0{{bytes:&second}})");
        let alternate = format!("function{other}(Record0{{bytes:&first}},Record0{{bytes:&second}})");
        let first_pointer = if callback_only { "" } else { "assert_eq!(root.field0.bytes.as_ptr(),first.as_ptr());" };
        let probe = format!("fn family_probe(){{std::thread::Builder::new().stack_size(64*1024).spawn(||{{let first=vec![3u8];let second=vec![7u8];let mut root={make};for index in 0..10000{{let mut parent=if index%2==0{{{make}}}else{{{alternate}}};parent.field2=Union{union}::V{tag}(RecordLink{record}::new(root));root=parent;}}{first_pointer}let result={invoke_root};assert_eq!(result.bytes.as_ptr(),second.as_ptr());assert_eq!(result.bytes[0],7);}}).unwrap().join().unwrap();}}");
        let negative = format!("fn invalid_escape(){{let retained;let first=vec![3u8];{{let second=vec![7u8];let holder={make};retained={invoke_holder};}}assert_eq!(retained.bytes[0],7);}}");
        let mut generated = crate::emit::emit(&checked);
        let main = generated.find("fn main()").unwrap();
        let opening = main + generated[main..].find('{').unwrap() + 1;
        generated.insert_str(opening,"family_probe();");
        let sibling_owner = format!("fn invalid_sibling_escape(){{let retained;let second=vec![7u8];{{let first=vec![3u8];let holder={make};retained={invoke_holder};}}assert_eq!(retained.bytes[0],7);}}");
        compile_family(&generated,"",&probe,vec![(negative,"does not live long enough"),(sibling_owner,"does not live long enough")],if nullable {"recursive-nullable-callback-chain"} else if callback_only {"recursive-captured-chain"} else {"recursive-chain"});
        }
    }

    #[test]
    fn recursive_parameter_mutation_preserves_native_identity_and_owner_bounds() {
        let source="type Window={};type Holder={data:Window;read:Once<Function<Window>>;next?:Holder;};function build(first:Window,second:Window):Holder{return {data:first,read:()=>second};}function replace(holder:Mutable<Holder>,replacement:Window):void{holder.read=()=>replacement;}function take(holder:Holder){return holder.read;}function main(){}";
        let source=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&source).unwrap(),source);
        let mut parsed=crate::syntax::parse(&source).unwrap();
        parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        let checked=crate::analysis::analyze(parsed).unwrap();
        let function=|name:&str|checked.functions.iter().position(|function|function.name==name&&!function.template).unwrap();
        let factory=function("build");let replace=function("replace");let take=function("take");
        let Type::Record(record)=checked.functions[factory].result else{unreachable!()};
        let union=checked.records[record].fields[2].ty.unwrap().union().unwrap();
        let tag=checked.unions[union.index()].members.iter().position(|ty|*ty==Type::Record(record)).unwrap();
        let union=union.index();
        let make=format!("function{factory}(Record0{{bytes:&first}},Record0{{bytes:&second}})");
        let store=format!("function{replace}(&mut root,Record0{{bytes:&third}})");
        let probe=format!("fn family_probe(){{std::thread::Builder::new().stack_size(64*1024).spawn(||{{let first=vec![3u8];let second=vec![7u8];let third=vec![11u8];let mut root={make};for _ in 0..10000{{let mut parent={make};parent.field2=Union{union}::V{tag}(RecordLink{record}::new(root));root=parent;}}{store};assert_eq!(root.field0.bytes.as_ptr(),first.as_ptr());let result=function{take}(root).call();assert_eq!(result.bytes.as_ptr(),third.as_ptr());assert_eq!(result.bytes[0],11);}}).unwrap().join().unwrap();}}");
        let new_owner=format!("fn invalid_new_owner(){{let first=vec![3u8];let second=vec![7u8];let mut root={make};{{let third=vec![11u8];{store};}}let result=function{take}(root).call();assert_eq!(result.bytes[0],11);}}");
        let sibling=format!("fn invalid_sibling(){{let second=vec![7u8];let third=vec![11u8];let retained;{{let first=vec![3u8];let mut root={make};{store};retained=function{take}(root).call();}}assert_eq!(retained.bytes[0],11);}}");
        let mut generated=crate::emit::emit(&checked);let main=generated.find("fn main()").unwrap();let opening=main+generated[main..].find('{').unwrap()+1;generated.insert_str(opening,"family_probe();");
        compile_family(&generated,"",&probe,vec![(new_owner,"does not live long enough"),(sibling,"does not live long enough")],"recursive-parameter-mutation");
    }

}
