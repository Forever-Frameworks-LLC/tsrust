//! Owned recursive carrier moves use the same Enter/Finish discipline as bounded
//! record cloning. Definitions and pair identities are local to one expression.
use super::*;

fn request(pairs:&mut Vec<(Type,Type)>,source:Type,target:Type)->usize {
    if let Some(index)=pairs.iter().position(|&pair|pair==(source,target)){return index;}
    let index=pairs.len();pairs.push((source,target));index
}

impl Emitter<'_> {
    pub(super) fn record_carrier_union_target(&self,targets:&[Type],member:Type)->usize {
        let callable=|ty|matches!(ty,Type::Callback(_)|Type::Closure(_)|Type::StoredCallable(_)|Type::BorrowedCallable(_));
        targets.iter().position(|&candidate|candidate.accepts_scalar_value(member))
            .or_else(||targets.iter().position(|&candidate|matches!((candidate,member),(Type::Array(_),Type::Array(_))|(Type::Record(_),Type::Record(_)))||(callable(candidate)&&callable(member))))
            .expect("checked nullable carrier member")
    }

    pub(super) fn bounded_record_carrier_conversion(&self,source:Type,target:Type,value:&str)->String {
        let mut pairs=vec![(source,target)];
        let mut tasks=String::new();let mut enters=String::new();let mut finishes=String::new();
        let mut next=0;
        while next<pairs.len(){
            let index=next;let(source,target)=pairs[index];next+=1;
            if target.accepts_scalar_value(source)||self.absent_record_field(source).is_some(){
                let converted=self.record_carrier_conversion(source,target,"value");
                let _=writeln!(enters,"CarrierTask::Enter{index}(value)=>completed.push(CarrierNode::N{index}({converted})),");
                continue;
            }
            match(source,target){
                (Type::Record(from),Type::Record(to))=>{
                    let original=&self.program.records[from].fields;let destination=&self.program.records[to].fields;
                    assert_eq!(original.len(),destination.len(),"checked carrier field count");
                    let fields=original.iter().zip(destination).map(|(a,b)|{assert_eq!(a.name.text,b.name.text,"checked carrier field order");request(&mut pairs,a.ty.unwrap(),b.ty.unwrap())}).collect::<Vec<_>>();
                    let ordered=self.record_runtime.ordered.contains(&to);
                    let metadata=if ordered{format!("([usize;{}])",original.len())}else{String::new()};
                    let _=writeln!(tasks,"Finish{index}{metadata},");
                    let finish=if ordered{"(value.key_order)"}else{""};
                    let _=writeln!(enters,"CarrierTask::Enter{index}(value)=>{{tasks.push(CarrierTask::Finish{index}{finish});");
                    for(position,&child)in fields.iter().enumerate().rev(){let _=writeln!(enters,"tasks.push(CarrierTask::Enter{child}(value.field{position}));");}
                    enters.push_str("},\n");
                    let pattern=if ordered{"(key_order)"}else{""};
                    let _=writeln!(finishes,"CarrierTask::Finish{index}{pattern}=>{{");
                    for(position,&child)in fields.iter().enumerate().rev(){let _=writeln!(finishes,"let CarrierNode::N{child}(field{position})=completed.pop().expect(\"completed carrier field\")else{{unreachable!(\"checked carrier field result\")}};");}
                    let mut values=(0..fields.len()).map(|position|format!("field{position}")).collect::<Vec<_>>().join(",");
                    if ordered{if !values.is_empty(){values.push(',');}values.push_str("key_order");}
                    let _=writeln!(finishes,"completed.push(CarrierNode::N{index}(Record{}{{{values}}}));}},",self.native_record_declaration(to));
                }
                (Type::Array(from),Type::Array(to))=>{
                    let child=request(&mut pairs,from.ty(),to.ty());
                    let _=writeln!(tasks,"Finish{index}(usize),");
                    // Push in reverse source order so the LIFO worklist evaluates
                    // items in original order. Only ownership moves, no cloning.
                    let _=writeln!(enters,"CarrierTask::Enter{index}(mut value)=>{{let length=value.len();tasks.push(CarrierTask::Finish{index}(length));while let Some(item)=value.pop(){{tasks.push(CarrierTask::Enter{child}(item));}}}},");
                    let native=self.recursive_view_native_type(target,"'carrier");
                    let _=writeln!(finishes,"CarrierTask::Finish{index}(length)=>{{let mut output:{native}=From::from([]);for _ in 0..length{{let CarrierNode::N{child}(item)=completed.pop().expect(\"completed carrier item\")else{{unreachable!(\"checked carrier item result\")}};output.push(item);}}output.reverse();completed.push(CarrierNode::N{index}(output));}},");
                }
                (Type::Union(from),destination) if matches!(destination,Type::Union(_)|Type::Array(_))=>{
                    let _=writeln!(enters,"CarrierTask::Enter{index}(value)=>match value{{");
                    for(tag,&member)in self.program.unions[from.index()].members.iter().enumerate(){
                        let(to,output)=if let Type::Union(to)=destination{
                            let targets=&self.program.unions[to.index()].members;let to_tag=self.record_carrier_union_target(targets,member);
                            (targets[to_tag],Some((to,to_tag)))
                        }else{(destination,None)};
                        if let Some((to_union,to_tag))=output&&to.accepts_scalar_value(member){
                            let payload=self.union_transfer_payload(from,to_union,member,"payload");
                            let _=writeln!(enters,"Union{}::V{tag}(payload)=>completed.push(CarrierNode::N{index}(Union{}::V{to_tag}({payload}))),",from.index(),to_union.index());
                            continue;
                        }
                        let child=request(&mut pairs,member,to);let owned=self.union_move_payload(from,member,"payload");
                        let _=writeln!(tasks,"Finish{index}_{tag},");
                        let _=writeln!(enters,"Union{}::V{tag}(payload)=>{{tasks.push(CarrierTask::Finish{index}_{tag});tasks.push(CarrierTask::Enter{child}({owned}));}},",from.index());
                        let converted=if let Some((to_union,to_tag))=output{format!("Union{}::V{to_tag}({})",to_union.index(),self.union_store_payload(to_union,to,"payload"))}else{"payload".into()};
                        let _=writeln!(finishes,"CarrierTask::Finish{index}_{tag}=>{{let CarrierNode::N{child}(payload)=completed.pop().expect(\"completed carrier alternative\")else{{unreachable!(\"checked carrier alternative result\")}};completed.push(CarrierNode::N{index}({converted}));}},");
                    }
                    enters.push_str("},\n");
                }
                (from,Type::Union(to))=>{
                    let targets=&self.program.unions[to.index()].members;
                    let tag=targets.iter().position(|&member|member.accepts_scalar_value(from)).or_else(||targets.iter().position(|member|matches!((from,member),(Type::Array(_),Type::Array(_))))).expect("checked array element alternative");
                    let child=request(&mut pairs,from,targets[tag]);let stored=self.union_store_payload(to,targets[tag],"payload");
                    let _=writeln!(tasks,"Finish{index},");
                    let _=writeln!(enters,"CarrierTask::Enter{index}(value)=>{{tasks.push(CarrierTask::Finish{index});tasks.push(CarrierTask::Enter{child}(value));}},");
                    let _=writeln!(finishes,"CarrierTask::Finish{index}=>{{let CarrierNode::N{child}(payload)=completed.pop().expect(\"completed carrier injection\")else{{unreachable!(\"checked carrier injection result\")}};completed.push(CarrierNode::N{index}(Union{}::V{tag}({stored})));}},",to.index());
                }
                _=>{
                    let converted=self.record_carrier_conversion(source,target,"value");
                    let _=writeln!(enters,"CarrierTask::Enter{index}(value)=>completed.push(CarrierNode::N{index}({converted})),");
                }
            }
        }
        // Every task type is a payload within the root recursive component.
        // Use precisely that component's existing shared-lifetime renderer;
        // acyclic enclosing expressions retain their independent outer slots.
        let native=pairs.iter().map(|&(source,target)|(self.recursive_view_native_type(source,"'carrier"),self.recursive_view_native_type(target,"'carrier"))).collect::<Vec<_>>();
        let task_scoped=native.iter().any(|(source,_)|source.contains("'carrier"));
        let node_scoped=native.iter().any(|(_,target)|target.contains("'carrier"));
        let task_generics=if task_scoped{"<'carrier>"}else{""};
        let node_generics=if node_scoped{"<'carrier>"}else{""};
        let generics=if task_scoped||node_scoped{"<'carrier>"}else{""};
        let entries=native.iter().enumerate().map(|(index,(source,_))|format!("Enter{index}({source}),")).collect::<String>();
        let nodes=native.iter().enumerate().map(|(index,(_,target))|format!("N{index}({target}),")).collect::<String>();
        let(input,output)=&native[0];
        format!("{{enum CarrierTask{task_generics}{{{entries}{tasks}}}enum CarrierNode{node_generics}{{{nodes}}}fn convert_carrier{generics}(value:{input})->{output}{{let mut tasks=vec![CarrierTask::Enter0(value)];let mut completed=Vec::new();while let Some(task)=tasks.pop(){{match task{{{enters}{finishes}}}}}let CarrierNode::N0(result)=completed.pop().expect(\"completed carrier root\")else{{unreachable!(\"checked carrier root result\")}};assert!(completed.is_empty());result}}convert_carrier({value})}}")
    }
}
