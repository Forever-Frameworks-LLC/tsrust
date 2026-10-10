//! Named native lifetimes preserve independent input origins and returned intersections.
use super::*;
use crate::analysis::native_view_loans::{LifetimeSource, Segment};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Lifetimes {
    inputs: BTreeMap<(usize,LifetimeSource),String>,
    outputs: BTreeMap<Vec<Segment>,String>,
    pub(super) declarations: Vec<String>,
    pub(super) bounds: Vec<String>,
}
impl Lifetimes {
    pub(super) fn new(function:&CheckedFunction,parameters:&[usize])->Self {
        let mut plan=Self::default();
        for &root in parameters {
            for loan in function.bindings[root].native_view_loans.as_deref().into_iter().flatten() {
                if matches!(loan.source,LifetimeSource::Carried(_)) {plan.inputs.entry((root,loan.source.clone())).or_default();}
            }
        }
        for loan in function.native_returns.as_deref().into_iter().flatten() {
            plan.inputs.entry((loan.root,loan.source.clone())).or_default();
        }
        let mut slots=BTreeMap::<usize,usize>::new();
        for ((root,_),name) in &mut plan.inputs {
            let slot=slots.entry(*root).or_default();
            *name=format!("'native_input{root}_{slot}");*slot+=1;
            plan.declarations.push(name.clone());
        }
        let mut outputs=BTreeMap::<Vec<Segment>,Vec<String>>::new();
        for loan in function.native_returns.as_deref().into_iter().flatten() {
            outputs.entry(loan.path.clone()).or_default().push(plan.inputs[&(loan.root,loan.source.clone())].clone());
        }
        for (path,mut sources) in outputs {
            sources.sort();sources.dedup();
            let name=if sources.len()==1 {sources.remove(0)} else {
                let output=format!("'native_return{}",plan.outputs.len());
                plan.declarations.push(output.clone());
                plan.bounds.extend(sources.into_iter().map(|source|format!("{source}: {output}")));
                output
            };
            plan.outputs.insert(path,name);
        }
        plan
    }
    pub(super) fn storage(&self,root:usize)->Option<&str> {
        self.inputs.get(&(root,LifetimeSource::Storage)).map(String::as_str)
    }
    fn slot(&self,input:Option<usize>,path:&[Segment])->&str {
        match input {
            Some(root)=>self.inputs.get(&(root,LifetimeSource::Carried(path.to_vec()))),
            None=>self.outputs.get(path),
        }.map_or("'_",String::as_str)
    }
}
impl Emitter<'_> {
    pub(super) fn native_view_signature_type(&self,ty:Type,input:Option<usize>,plan:&Lifetimes)->String {
        self.native_view_signature_slot(ty,input,plan,&mut Vec::new())
    }
    fn native_view_signature_slot(&self,ty:Type,input:Option<usize>,plan:&Lifetimes,path:&mut Vec<Segment>)->String {
        if !self.native_views.contains(ty) {return self.native_type(ty);}
        match ty {
            Type::Record(id) if native_view_emit::access(self.program,id).is_some()=>format!("Record{id}<{}>",plan.slot(input,path)),
            Type::Record(id)=>{
                let base=self.program.scoped_records.declaration(id);
                let parameters=self.program.records[id].fields.iter().zip(&self.program.records[base].fields).filter_map(|(field,declared)|{
                    let ty=field.ty.expect("checked field");
                    if !(self.scoped_record_native_parameter(ty)||self.scoped_record_native_parameter(declared.ty.expect("checked field"))) {return None;}
                    path.push(Segment::Field(field.name.text.clone()));
                    let result=self.native_view_signature_slot(ty,input,plan,path);path.pop();Some(result)
                }).collect::<Vec<_>>();
                if parameters.is_empty(){self.native_type(ty)}else{format!("Record{base}<{}>",parameters.join(", "))}
            }
            Type::Array(element)=>{
                path.push(Segment::Element);
                let payload=self.native_view_signature_slot(element.ty(),input,plan,path);path.pop();
                match element {
                    ArrayElement::Record(id,0)=>{
                        if let Some((_,arguments))=payload.split_once('<'){format!("RecordArray{id}<{arguments}")}else{format!("RecordArray{id}")}
                    }
                    _=>format!("Vec<{payload}>"),
                }
            }
            Type::Union(id)|Type::BorrowedUnion(id)=>{
                let arguments=self.program.unions[id.index()].members.iter().filter(|member|!member.is_copy()).map(|&member|self.native_view_signature_slot(member,input,plan,path)).collect::<Vec<_>>().join(", ");
                if matches!(ty,Type::BorrowedUnion(_)){format!("ViewUnion{}<'_,{arguments}>",id.index())}else{format!("Union{}<{arguments}>",id.index())}
            }
            _=>self.native_type(ty),
        }
    }
}
