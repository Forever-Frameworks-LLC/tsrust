//! Native replacement converts a checked origin into an associated field carrier.
use super::*;
impl Emitter<'_>{
    /// Reconstruct the actual replacement payload in its original family.
    /// Moving through only proven variants ends the overwritten environment's
    /// native lifetime without a clone, placeholder or lifetime cast.
    pub(super) fn record_call_reification(&self,fun:&CheckedFunction,args:&[Expr],callee:usize)->String{
        let mut result=String::new();
        for(parameter,names)in self.program.record_call_releases.get(&callee).into_iter().flatten(){
            let Some(arg)=args.get(*parameter)else{continue;};
            let(root,steps)=self.storage_steps(arg);let Some(id)=root.binding else{continue;};
            if self.owned_record_layout(fun,id).is_none(){continue;}
            let mut path=Vec::new();let mut valid=true;
            for step in &steps{match step{StorageStep::Field(field)=>path.push(*field),StorageStep::Discard(_)=>{},_=>{valid=false;break;}}}
            if !valid{continue;}
            let mut ty=fun.bindings[id].ty;
            for &field in &path{let Type::Record(record)=ty else{valid=false;break;};ty=self.program.records[record].fields[field].ty.unwrap();}
            for name in names{let Type::Record(record)=ty else{valid=false;break;};let Some(field)=self.program.records[record].fields.iter().position(|field|field.name.text==*name)else{valid=false;break;};path.push(field);ty=self.program.records[record].fields[field].ty.unwrap();}
            if !valid{continue;}
            let Type::BorrowedCallable(family)=ty else{continue;};
            let origins=self.program.stored_contracts.origins(family).expect("checked record family");
            let mut allowed=self.program.record_parameter_replacements.get(&callee).into_iter().flatten().filter(|(slot,path,_)|slot==parameter && path==names).flat_map(|(_,_,ty)|match ty{Type::Closure(id)=>vec![*id],Type::BorrowedCallable(id)=>self.program.stored_contracts.origins(*id).unwrap().to_vec(),_=>Vec::new()}).collect::<Vec<_>>();
            allowed.sort();allowed.dedup();
            if allowed.is_empty() || allowed.iter().any(|origin|!origins.contains(origin)){continue;}
            let Some(place)=self.owned_record_leaf_place(fun,id,&path)else{continue;};
            let arms=allowed.iter().map(|&origin|{
                let position=origins.iter().position(|&candidate|candidate==origin).unwrap();
                let arm=crate::borrowed_callable_dispatch::variant(position,origins.len());
                format!("BorrowedCallable{family}::{arm}(payload)=>{}",self.stored_callable_constructor(family,origin,"payload"))
            }).collect::<Vec<_>>().join(",");
            let fallback=if allowed.len()==origins.len(){String::new()}else{",_=>unreachable!(\"checked record replacement retained an old origin\")".to_owned()};
            let _=write!(result,"{place}=match {place}{{{arms}{fallback}}};");
        }
        result
    }
    /// The borrowed family that a called record field holds in its current layout.
    pub(super) fn record_callee_family(&self,fun:&CheckedFunction,callee:&Expr)->Option<usize>{
        let(root,steps)=self.storage_steps(callee);
        let mut ty=fun.bindings[root.binding?].ty;
        if steps.is_empty(){return None;}
        for step in &steps{match step{
            StorageStep::Field(field)=>{let Type::Record(record)=ty else{return None;};ty=self.program.records[record].fields[*field].ty?;}
            StorageStep::Discard(_)=>{}
            _=>return None,
        }}
        match ty{Type::BorrowedCallable(family)=>Some(family),_=>None}
    }
    pub(super) fn record_replacement_bounds(&self,function:usize,parameter:usize,path:&[usize])->Vec<String>{
        let fun=&self.program.functions[function];
        let Some(layout)=self.record_parameter_layout(fun,parameter)else{return Vec::new();};
        let mut record=layout.record;let mut names=Vec::new();
        for(index,&field)in path.iter().enumerate(){let selected=&self.program.records[record].fields[field];names.push(selected.name.text.clone());if index+1<path.len(){let Some(Type::Record(child))=selected.ty else{return Vec::new();};record=child;}}
        let mut bounds=self.program.record_parameter_replacements.get(&function).into_iter().flatten().filter(|(id,selected,_)|*id==parameter && selected==&names).map(|(_,_,ty)|format!("From<{}>",self.native_type(*ty))).collect::<Vec<_>>();
        bounds.sort();bounds.dedup();bounds
    }
    pub(super) fn record_parameter_replacement(&self,fun:&CheckedFunction,target:&Expr,value:&Expr,code:&str)->Option<String>{
        let(root,steps)=self.storage_steps(target);
        let id=root.binding?;let binding=&fun.bindings[id];
        if !binding.parameter || id>=fun.parameter_count || binding.mode!=Mode::Mutate{return None;}
        let layout=self.record_parameter_layout(fun,id)?;
        let path=steps.iter().map(|step|match step{StorageStep::Field(field)=>Some(*field),_=>None}).collect::<Option<Vec<_>>>()?;
        let leaf=layout.leaves.iter().find(|leaf|leaf.path==path)?;
        if matches!(leaf.ty,Type::Union(_)) && matches!(value.ty,Some(Type::Union(_))) {
            let source=value.ty.unwrap();
            let converted=self.record_carrier_conversion(source,leaf.ty,"nullable_replacement");
            return Some(format!("{{let nullable_replacement={code};{converted}}}"));
        }
        if !matches!(leaf.ty,Type::Callback(_)) || !matches!(value.ty,Some(Type::Closure(_) | Type::BorrowedCallable(_))){return None;}
        let field=path.iter().map(|id|format!("Field{id}")).collect::<String>();
        Some(format!("<<R{id} as TsrReadRecord{}>::{field} as From<{}>>::from({code})",layout.record,self.native_type(value.ty.unwrap())))
    }
}
