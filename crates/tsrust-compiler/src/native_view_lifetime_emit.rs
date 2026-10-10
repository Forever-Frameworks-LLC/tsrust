//! Named native lifetimes preserve independent input origins and returned intersections.
use super::*;
use crate::analysis::native_view_loans::{LifetimeSource, Segment, VariantGuard};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Lifetimes {
    inputs: BTreeMap<(usize,LifetimeSource),String>,
    outputs: BTreeMap<(Vec<Segment>,Vec<VariantGuard>),String>,
    pub(super) declarations: Vec<String>,
    pub(super) bounds: Vec<String>,
}
impl Lifetimes {
    pub(super) fn with_captures(function:&CheckedFunction,parameters:&[usize],captures:&BTreeMap<(usize,LifetimeSource),String>)->Self {
        let mut plan=Self::default();
        for &root in parameters {
            for loan in function.bindings[root].native_view_loans.as_deref().into_iter().flatten() {
                if !matches!(loan.source,LifetimeSource::Storage) {plan.inputs.entry((root,loan.source.clone())).or_default();}
            }
        }
        for loan in function.native_returns.as_deref().into_iter().flatten() {
            plan.inputs.entry((loan.root,loan.source.clone())).or_default();
        }
        let mut slots=BTreeMap::<usize,usize>::new();
        for ((root,source),name) in &mut plan.inputs {
            if let Some(captured)=captures.get(&(*root,source.clone())) {*name=captured.clone();continue;}
            let slot=slots.entry(*root).or_default();
            *name=format!("'native_input{root}_{slot}");*slot+=1;
            plan.declarations.push(name.clone());
        }
        let mut outputs=BTreeMap::<(Vec<Segment>,Vec<VariantGuard>),Vec<String>>::new();
        for loan in function.native_returns.as_deref().into_iter().flatten() {
            outputs.entry((loan.path.clone(),loan.guards.clone())).or_default().push(plan.inputs[&(loan.root,loan.source.clone())].clone());
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
    fn slot(&self,input:Option<usize>,path:&[Segment],guards:&[VariantGuard])->&str {
        match input {
            Some(root)=>self.inputs.get(&(root,LifetimeSource::carried(path.to_vec(),guards.to_vec()))),
            None=>self.outputs.get(&(path.to_vec(),guards.to_vec())),
        }.map_or("'_",String::as_str)
    }
}
impl Emitter<'_> {
    pub(super) fn native_view_closure_signature(&self,closure:&crate::analysis::CheckedClosure,method:Option<usize>,function:&CheckedFunction,parameters:&[usize])->Option<native_function_signature_emit::NativeSignature> {
        if function.native_returns.is_none() && !parameters.iter().any(|&id|self.native_views.contains(function.bindings[id].input_ty.unwrap_or(function.bindings[id].ty))) {return None;}
        let (lifetimes,_)=self.capture_layout(closure);
        let mut next=0;
        let mut captures=BTreeMap::new();
        let mut receiver=false;
        for (position,capture) in closure.captures.iter().enumerate() {
            let root=method.map_or(capture.inner,|method|closure.methods[method].captures[position]);
            if capture.storage==Mode::Read {captures.insert((root,LifetimeSource::Storage),lifetimes[next].clone());}
            else if function.native_returns.as_deref().into_iter().flatten().any(|loan|loan.root==root && loan.source==LifetimeSource::Storage) {
                captures.insert((root,LifetimeSource::Storage),"'native_receiver".into());receiver=true;
            }
            next+=usize::from(matches!(capture.storage,Mode::Read|Mode::Mutate));
            let ty=self.native_captures.ty(&closure.function,capture.inner);
            self.native_view_capture_slots(ty,root,&mut Vec::new(),&mut Vec::new(),&lifetimes,&mut next,&mut captures);
        }
        let mut signature=self.native_signature_from_plan(native_function_signature_emit::NativeSignaturePlan {
            native_capture_lifetimes:captures,
            function,
            parameters:parameters.to_vec(),
            source_parameters:parameters.to_vec(),
            borrow_lifetimes:Default::default(),
            owner_inputs:Vec::new(),
            owner_parameters:String::new(),
            record_bounds:Vec::new(),
            replacement_bounds:Vec::new(),
        },&[]);
        if receiver {
            let invocation=method.map_or(closure.invocation,|method|closure.methods[method].invocation);
            signature.receiver=Some(invocation);
            signature.generics=if signature.generics.is_empty() {"<'native_receiver>".into()}else{format!("<'native_receiver, {}",&signature.generics[1..])};
        }
        Some(signature)
    }

    fn native_view_capture_slots(&self,ty:Type,root:usize,path:&mut Vec<Segment>,guards:&mut Vec<VariantGuard>,lifetimes:&[String],next:&mut usize,slots:&mut BTreeMap<(usize,LifetimeSource),String>) {
        match ty {
            Type::Record(id) if native_view_emit::access(self.program,id).is_some()=>{
                slots.insert((root,LifetimeSource::carried(path.clone(),guards.clone())),lifetimes[*next].clone());*next+=1;
            }
            Type::Record(id) if self.native_views.contains(ty)=>{
                for field in &self.program.records[id].fields {
                    path.push(Segment::Field(field.name.text.clone()));
                    self.native_view_capture_slots(field.ty.expect("checked field"),root,path,guards,lifetimes,next,slots);path.pop();
                }
            }
            Type::Array(element) if self.native_views.contains(ty)=>{
                path.push(Segment::Element);self.native_view_capture_slots(element.ty(),root,path,guards,lifetimes,next,slots);path.pop();
            }
            Type::Union(id)|Type::BorrowedUnion(id) if self.native_views.contains(ty)=>{
                for &member in &self.program.unions[id.index()].members {
                    let variant=if let Type::Record(record)=member {guards.push(VariantGuard{path:path.clone(),success:record});true}else{false};
                    self.native_view_capture_slots(member,root,path,guards,lifetimes,next,slots);
                    if variant {guards.pop();}
                }
            }
            _=>*next+=self.native_captures.type_lifetimes(self.program,&self.borrowed_lifetime_alternatives,ty),
        }
    }

    pub(super) fn native_view_signature_type(&self,ty:Type,input:Option<usize>,plan:&Lifetimes)->String {
        self.native_view_signature_slot(ty,input,plan,&mut Vec::new(),&mut Vec::new())
    }
    fn native_view_signature_slot(&self,ty:Type,input:Option<usize>,plan:&Lifetimes,path:&mut Vec<Segment>,guards:&mut Vec<VariantGuard>)->String {
        if !self.native_views.contains(ty) {return self.native_type(ty);}
        match ty {
            Type::Record(id) if native_view_emit::access(self.program,id).is_some()=>format!("Record{id}<{}>",plan.slot(input,path,guards)),
            Type::Record(id)=>{
                let base=self.program.scoped_records.declaration(id);
                let parameters=self.program.records[id].fields.iter().zip(&self.program.records[base].fields).filter_map(|(field,declared)|{
                    let ty=field.ty.expect("checked field");
                    if !(self.scoped_record_native_parameter(ty)||self.scoped_record_native_parameter(declared.ty.expect("checked field"))) {return None;}
                    path.push(Segment::Field(field.name.text.clone()));
                    let result=self.native_view_signature_slot(ty,input,plan,path,guards);path.pop();Some(result)
                }).collect::<Vec<_>>();
                if parameters.is_empty(){self.native_type(ty)}else{format!("Record{base}<{}>",parameters.join(", "))}
            }
            Type::Array(element)=>{
                path.push(Segment::Element);
                let payload=self.native_view_signature_slot(element.ty(),input,plan,path,guards);path.pop();
                match element {
                    ArrayElement::Record(id,0)=>{
                        if let Some((_,arguments))=payload.split_once('<'){format!("RecordArray{id}<{arguments}")}else{format!("RecordArray{id}")}
                    }
                    _=>format!("Vec<{payload}>"),
                }
            }
            Type::Union(id)|Type::BorrowedUnion(id)=>{
                let arguments=self.program.unions[id.index()].members.iter().filter(|member|!member.is_copy()).map(|&member|{
                    let variant=if let Type::Record(record)=member {guards.push(VariantGuard{path:path.clone(),success:record});true}else{false};
                    let rendered=self.native_view_signature_slot(member,input,plan,path,guards);
                    if variant {guards.pop();}
                    rendered
                }).collect::<Vec<_>>().join(", ");
                if matches!(ty,Type::BorrowedUnion(_)){format!("ViewUnion{}<'_,{arguments}>",id.index())}else{format!("Union{}<{arguments}>",id.index())}
            }
            _=>self.native_type(ty),
        }
    }
}
