//! Once-only spread staging uses native owners and references.
use super::*;

// Keep eager snapshots: later arguments may change the source array. Small
// scalar snapshots use bounded inline storage and spill when they grow.
fn inline_copy_snapshot(plan:&CheckedSpreadCall,segment:&SpreadSegment)->bool {
    plan.builtin.is_none() && segment.spread && segment.access==Mode::Copy
        && matches!(segment.element,Type::Int|Type::Number|Type::Boolean)
}

pub(super) const SPREAD_SLOTS:&str=r#"
enum TsrSpreadSlot<'a,T> { Owned(T), Read(&'a T), Mutate(&'a mut T), Null, Undefined }
impl<'a,T> TsrSpreadSlot<'a,T> {
    fn tsr_read(&self)->&T {match self {Self::Owned(value)=>value,Self::Read(value)=>value,Self::Mutate(value)=>value,_=>unreachable!("checked present spread slot")}}
    fn tsr_mutate(&mut self)->&mut T {match self {Self::Owned(value)=>value,Self::Mutate(value)=>value,_=>unreachable!("checked exclusive spread slot")}}
    fn tsr_owned(self)->T {match self {Self::Owned(value)=>value,_=>unreachable!("checked owning spread slot")}}
}
"#;

impl Emitter<'_> {
    fn spread_multiple(&self, union:UnionId)->bool {self.program.unions[union.index()].members.iter().filter(|ty|!matches!(ty,Type::Null|Type::Undefined)).count()>1}
    fn spread_carrier(&self,union:UnionId,member:Type,carrier:&str,payload:Option<&str>)->String {
        let slot=if let Some(payload)=payload {format!("TsrSpreadSlot::{carrier}({payload})")}else{format!("TsrSpreadSlot::{carrier}")};
        if self.spread_multiple(union) {let index=self.program.unions[union.index()].members.iter().position(|ty|*ty==member).unwrap();format!("TsrDataSpread{}::V{index}({slot})",union.index())}else{slot}
    }
    pub(super) fn data_spread_declarations(&self,output:&mut String){
        for (index,union) in self.program.unions.iter().enumerate(){
            if !output.contains(&format!("TsrDataSpread{index}")){continue;}
            write!(output,"enum TsrDataSpread{index}<'a> {{").unwrap();
            for (position,member) in union.members.iter().enumerate(){write!(output,"V{position}(TsrSpreadSlot<'a,{}>),",self.native_type(*member)).unwrap();}
            output.push_str("}\n");
        }
    }

    pub(super) fn spread_selected_expression(&self,expression:&Expr,selected:&SpreadSelected,mode:Mode)->String {
        let name=selected.selection.native_name(selected.slot);
        if selected.carrier {
            if matches!(expression.ty,Some(Type::BorrowedUnion(_))) && mode==Mode::Read {return format!("*{name}.tsr_read()");}
            if let Some(union)=selected.normalized_union {return self.spread_nullable_slot(&name,union,mode);}
            if expression.ty.is_some_and(Type::is_copy) && mode!=Mode::Mutate {return format!("*{name}.tsr_read()");}
            return match mode {Mode::Copy=>format!("*{name}.tsr_read()"),Mode::Read=>format!("{name}.tsr_read()"),Mode::Mutate=>format!("{name}.tsr_mutate()"),Mode::Consume=>format!("{name}.tsr_owned()")};
        }
        if expression.ty.is_some_and(Type::is_copy) {return name;}
        match (selected.access,mode) {
            (Mode::Read,Mode::Read)=>name,
            (Mode::Mutate,Mode::Read)=>format!("&*{name}"),
            (Mode::Mutate,Mode::Mutate)=>format!("&mut *{name}"),
            (_,Mode::Read)=>format!("&{name}"),
            (_,Mode::Mutate)=>format!("&mut {name}"),
            _=>name,
        }
    }


    /// A nullable carrier may hold an owner or an exclusive loan at runtime.
    /// Lend either one directly into the mutable parameter union; a staged
    /// owning temporary would detach a borrowed element from its source.
    pub(super) fn spread_mutable_input(&self,value:&Expr,target:UnionId)->Option<String> {
        let mut selected=value;
        while let ExprKind::Inject{value,..}=&selected.kind {selected=value;}
        let ExprKind::SpreadValue(slot)=&selected.kind else {return None;};
        let source=slot.normalized_union.filter(|_|slot.carrier)?;
        let name=slot.selection.native_name(slot.slot);
        let members=&self.program.unions[target.index()].members;
        let mut arms=Vec::new();
        for member in &self.program.unions[source.index()].members {
            let start=arms.len();
            let to=members.iter().position(|ty|ty==member)?;
            if matches!(member,Type::Null|Type::Undefined) {
                let variant=if *member==Type::Null {"Null"}else{"Undefined"};
                arms.push(format!("TsrSpreadSlot::{variant}=>Union{}::V{to}(Tsr{variant})",target.index()));
            } else if member.is_copy() {
                arms.push(format!("TsrSpreadSlot::Owned(payload)=>Union{}::V{to}(*payload)",target.index()));
                arms.push(format!("TsrSpreadSlot::Read(payload)=>Union{}::V{to}(**payload)",target.index()));
                arms.push(format!("TsrSpreadSlot::Mutate(payload)=>Union{}::V{to}(**payload)",target.index()));
            } else {
                arms.push(format!("TsrSpreadSlot::Owned(payload)=>Union{}::V{to}(payload)",target.index()));
                arms.push(format!("TsrSpreadSlot::Mutate(payload)=>Union{}::V{to}(&mut **payload)",target.index()));
            }
            if self.spread_multiple(source) {let index=self.program.unions[source.index()].members.iter().position(|ty|ty==member).unwrap();for arm in &mut arms[start..] {let (pattern,result)=arm.split_once("=>").unwrap();*arm=format!("TsrDataSpread{}::V{index}({pattern})=>{result}",source.index());}}
        }
        arms.push("_=>unreachable!(\"checked nullable spread slot\")".into());
        Some(format!("match &mut {name} {{{}}}",arms.join(",")))
    }

    fn spread_nullable_slot(&self,name:&str,union:UnionId,mode:Mode)->String {
        let members=&self.program.unions[union.index()].members;
        let view=!union.copy && mode==Mode::Read;
        let tag=if view {"ViewUnion"}else{"Union"};
        let mut arms=Vec::new();
        for (index,member) in members.iter().enumerate() {
            let start=arms.len();
            if matches!(member,Type::Null|Type::Undefined) {
                let variant=if *member==Type::Null {"Null"}else{"Undefined"};
                let payload=if view {format!("&Tsr{variant}")}else{format!("Tsr{variant}")};
                arms.push(format!("TsrSpreadSlot::{variant}=>{tag}{}::V{index}({payload})",union.index()));
            }else {
                for carrier in ["Owned","Read","Mutate"] {
                    let payload=match mode {
                        Mode::Copy=>if carrier=="Owned" {"*payload"}else{"**payload"},
                        Mode::Read if view=>if carrier=="Owned" {"payload"}else if carrier=="Read" {"*payload"}else{"&**payload"},
                        Mode::Read=>if carrier=="Owned" {"*payload"}else{"**payload"},
                        Mode::Mutate=>if carrier=="Owned" {"payload"}else if carrier=="Mutate" {"&mut **payload"}else{continue},
                        Mode::Consume=>if carrier=="Owned" {"payload"}else{continue},
                    };
                    arms.push(format!("TsrSpreadSlot::{carrier}(payload)=>{tag}{}::V{index}({payload})",union.index()));
                }
            }
            if self.spread_multiple(union) {for arm in &mut arms[start..] {let (pattern,result)=arm.split_once("=>").unwrap();*arm=format!("TsrDataSpread{}::V{index}({pattern})=>{result}",union.index());}}
        }
        arms.push("_=>unreachable!(\"checked nullable spread slot\")".into());
        let reference=match mode {Mode::Consume=>"",Mode::Mutate=>"&mut ",_=>"&"};
        format!("match {reference}{name} {{{}}}",arms.join(","))
    }

    pub(super) fn spread_call_expression(&self,fun:&CheckedFunction,plan:&CheckedSpreadCall,mode:Mode)->String {
        if matches!(mode,Mode::Read|Mode::Mutate) && plan.call.ty.is_some_and(|ty|!ty.is_copy() && !matches!(ty,Type::BorrowedUnion(_))) {
            let owned=self.spread_call_expression(fun,plan,Mode::Consume);
            return format!("{}({owned})",if mode==Mode::Read {"&"}else{"&mut "});
        }
        let mut output=String::from("{ ");
        if let Some(callee)=&plan.callee {
            let name=callee.selection.native_name(plan.slot);
            let code=self.expr(fun,&callee.value,callee.access);
            write!(output,"let mut {name} = {code}; ").unwrap();
        }
        for (index,segment) in plan.segments.iter().enumerate() {
            let name=format!("spread_segment{}_{index}",plan.slot);
            let code=self.expr(fun,&segment.value,if segment.spread && (segment.access==Mode::Copy || segment.value.ty==Some(Type::String)) {Mode::Read}else{segment.access});
            if segment.spread {
                let source=format!("spread_source{}_{index}",plan.slot);
                write!(output,"let mut {source} = {code}; ").unwrap();
                let iterator=if let Some(Type::Map(id))=segment.value.ty && self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap) && segment.access==Mode::Consume {
                    let Type::Record(pair)=segment.element else {unreachable!("checked ordered map entry")};
                    let pair=format!("Record{}",self.program.scoped_records.declaration(pair));
                    format!("{source}.into_source_entries().map(|(key,value)|{pair}{{field0:key,field1:value}})")
                }else if let Some(Type::Record(id))=segment.value.ty && let Some(native)=self.program.records[id].native_type.filter(|native|matches!(native,crate::native_signatures::NativeTypeId::Deque | crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet)) {
                    let ordered=matches!(native,crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet);
                    match segment.access {
                        Mode::Copy=>format!("{source}.values.{}().copied()",if ordered {"key_iter"}else{"iter"}),
                        Mode::Read=>format!("{source}.values.{}()",if ordered {"key_iter"}else{"iter"}),
                        Mode::Mutate=>{assert!(!ordered,"checked immutable ordered keys");format!("{source}.values.iter_mut()")},
                        Mode::Consume=>format!("{source}.values.{}()",if ordered {"into_keys"}else{"into_items"}),
                    }
                }else if segment.value.ty==Some(Type::String) {format!("{source}.as_ref().chars().map(|character|String::Owned(character.to_string()))")}else if matches!(segment.value.ty,Some(Type::Map(_))) && segment.access==Mode::Read {format!("{source}.entry_iter()")}else{match segment.access {Mode::Copy=>format!("{source}.iter().copied()"),Mode::Read=>format!("{source}.iter()"),Mode::Mutate=>format!("{source}.tsr_spread_mut_items()"),Mode::Consume=>format!("{source}.tsr_owned_items()")}};
                let iterator=if matches!(plan.call.kind,ExprKind::SpreadValue(_)) && let Some(Type::Array(element))=plan.call.ty && element.ty()!=segment.element {format!("{iterator}.map(|item|{})",self.element_alternative(segment.element,element.ty(),"item"))}else{iterator};
                if inline_copy_snapshot(plan,segment) {
                    write!(output,"let mut {name}: Array<_> = Array::from([]); for spread_snapshot_item in {iterator} {{ {name}.push(spread_snapshot_item); }} ").unwrap();
                } else {
                    write!(output,"let {name}: Vec<_> = {iterator}.collect(); ").unwrap();
                }
            }else {write!(output,"let {name} = Some({code}); ").unwrap();}
        }
        write!(output,"let spread_count{} = 0usize",plan.slot).unwrap();
        for (index,segment) in plan.segments.iter().enumerate() {let count=if segment.spread {format!("spread_segment{}_{index}.len()",plan.slot)}else{format!("usize::from(spread_segment{}_{index}.is_some())",plan.slot)};write!(output,".checked_add({count}).expect(\"expanded argument count overflow\")").unwrap();}
        output.push_str("; ");
        output.push_str(&self.spread_arity_guard(plan));
        let direct_variadic=matches!(plan.builtin,Some(SpreadBuiltinKind::NativeStringRest|SpreadBuiltinKind::MathVariadic|SpreadBuiltinKind::StringConcat|SpreadBuiltinKind::ConsoleVariadic|SpreadBuiltinKind::ObserveVariadic|SpreadBuiltinKind::ArrayUnshift|SpreadBuiltinKind::ArrayConcat));
        if !direct_variadic {
            let fixed=plan.selected_types.len();
            for (position,ty) in plan.selected_types.iter().enumerate() {
                let slot_type=if let Some(id)=plan.selected_unions[position].filter(|id|self.spread_multiple(*id)){format!("TsrDataSpread{}<'_>",id.index())}else{format!("TsrSpreadSlot<'_,{}>",self.native_type(plan.selected_unions[position].and_then(|id|self.program.unions[id.index()].members.iter().copied().find(|ty|!matches!(ty,Type::Null|Type::Undefined))).unwrap_or(if plan.selected_unions[position].is_some(){Type::Undefined}else{*ty})))};
                write!(output,"let mut {}: Option<{slot_type}> = None; ",SpreadSelection::Argument(position).native_name(plan.slot)).unwrap();
            }
            let rest=plan.maximum.is_none();
            if rest {
                let ty=plan.parameters.last().expect("checked owning rest pack").ty;
                write!(output,"let mut {} = <{}>::from([]); ",SpreadSelection::Rest.native_name(plan.slot),self.native_type(ty)).unwrap();
            }
            write!(output,"let mut spread_position{} = 0usize; ",plan.slot).unwrap();
            for (index,segment) in plan.segments.iter().enumerate() {
                if segment.length==Some(0) {continue;}
                let iterator=if inline_copy_snapshot(plan,segment) {"tsr_owned_items"}else{"into_iter"};
                write!(output,"for spread_item in spread_segment{}_{index}.{iterator}() {{ match spread_position{} {{ ",plan.slot,plan.slot).unwrap();
                let carrier=if matches!(segment.element,Type::BorrowedUnion(_)) {"Owned"}else{match segment.access {Mode::Copy|Mode::Consume=>"Owned",Mode::Read=>"Read",Mode::Mutate=>"Mutate"}};
                let (lower,upper)=crate::analysis::spread_position_range(&plan.segments,index,plan.minimum,plan.maximum);
                for position in lower..fixed.min(upper.map_or(fixed,|upper|upper+1)) {
                    let value=if let Some(selected_union)=plan.selected_unions[position] {
                        if matches!(segment.element,Type::Null|Type::Undefined) {self.spread_carrier(selected_union,segment.element,if segment.element==Type::Null {"Null"}else{"Undefined"},None)}
                        else if let Some(source)=segment.element.union() {
                            let tag=if matches!(segment.element,Type::BorrowedUnion(_)){"ViewUnion"}else{"Union"};
                            let payload_carrier=if matches!(segment.element,Type::BorrowedUnion(_)){"Read"}else{carrier};
                            let arms=self.program.unions[source.index()].members.iter().enumerate().map(|(index,member)|{
                                let value=if matches!(member,Type::Null|Type::Undefined){self.spread_carrier(selected_union,*member,if *member==Type::Null{"Null"}else{"Undefined"},None)}else{self.spread_carrier(selected_union,*member,payload_carrier,Some("payload"))};
                                format!("{tag}{}::V{index}(payload)=>{value}",source.index())
                            }).collect::<Vec<_>>().join(",");
                            format!("match spread_item {{{arms}}}")
                        }else{self.spread_carrier(selected_union,segment.element,carrier,Some("spread_item"))}
                    }else{format!("TsrSpreadSlot::{carrier}(spread_item)")};
                    write!(output,"{position} => {} = Some({value}), ",SpreadSelection::Argument(position).native_name(plan.slot)).unwrap();
                }
                if rest && upper.is_none_or(|upper|upper>=fixed) {
                    write!(output,"_ => {}.push(spread_item), ",SpreadSelection::Rest.native_name(plan.slot)).unwrap();
                }else {output.push_str("_ => unreachable!(\"checked expanded fixed arity\"), ");}
                write!(output,"}} spread_position{} += 1; }} ",plan.slot).unwrap();
            }
        }
        let call=match plan.builtin {
            Some(SpreadBuiltinKind::NativeStringRest)=>self.spread_native_string_rest(plan),
            Some(SpreadBuiltinKind::MathVariadic)=>self.spread_builtin_math_call(plan),
            Some(SpreadBuiltinKind::StringConcat)=>self.spread_builtin_string_concat(fun,plan),
            Some(SpreadBuiltinKind::ConsoleVariadic)=>self.spread_builtin_console_call(plan),
            Some(SpreadBuiltinKind::ObserveVariadic)=>self.spread_builtin_observe_call(fun,plan,mode),
            Some(SpreadBuiltinKind::ArrayUnshift)=>self.spread_builtin_unshift_call(fun,plan),
            Some(SpreadBuiltinKind::ArrayConcat)=>self.spread_builtin_array_concat_call(fun,plan),
            Some(SpreadBuiltinKind::ArraySplice)=>self.spread_builtin_splice_call(fun,plan),
            Some(SpreadBuiltinKind::ArrayToSpliced)=>self.spread_builtin_to_spliced_call(fun,plan),
            Some(_)=>self.spread_builtin_fixed_call(fun,plan,mode),
            None=>self.spread_language_call(fun,plan,mode),
        };
        output.push_str(&call);output.push_str(" }");output
    }

    #[inline(never)]
    fn spread_arity_guard(&self,plan:&CheckedSpreadCall)->String {
        let mut condition=format!("spread_count{} >= {}",plan.slot,plan.minimum);
        if let Some(maximum)=plan.maximum {write!(condition," && spread_count{} <= {maximum}",plan.slot).unwrap();}
        let message=format!("expanded call arity outside checked contract at bytes {}..{}",plan.call.span.start,plan.call.span.end);
        if plan.segments.iter().any(|segment|segment.length.is_none()) && (plan.minimum>0 || plan.maximum.is_some())
            && let Some(exit)=self.runtime_failure_exit("TypeError","call-arity",&message,plan.call.span) {
            format!("if !({condition}) {exit} ")
        }else{format!("assert!({condition}, {message:?}); ")}
    }

    fn spread_language_call(&self,fun:&CheckedFunction,plan:&CheckedSpreadCall,mode:Mode)->String {
        let mut output=format!("match spread_count{} {{ ",plan.slot);
        let mut exhaustive=false;
        for variant in &plan.variants {
            let count=variant.count;
            if plan.maximum.is_none() && count==plan.selected_types.len() {exhaustive=true;output.push_str("_ => { ");}else{write!(output,"{count} => {{ ").unwrap();}
            for position in 0..count {let local=SpreadSelection::Argument(position).native_name(plan.slot);write!(output,"let mut {local} = {local}.take().expect(\"checked expanded slot\"); ").unwrap();}
            output.push_str(&self.expr(fun,&variant.call,mode));output.push_str(" }, ");
        }
        // Statically shorter sources omit the unbounded rest variant.
        if !exhaustive {output.push_str("_ => unreachable!(\"checked expanded arity\"), ");}
        output.push('}');output
    }
}
