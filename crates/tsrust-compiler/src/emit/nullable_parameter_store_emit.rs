//! Conversion bounds for owned callbacks installed behind nullable record fields.
use super::*;
impl Emitter<'_> {
    pub(super) fn nullable_parameter_replacement_bounds(&self,function:usize)->Vec<String> {
        let fun=&self.program.functions[function];
        let mut bounds=Vec::new();
        for (parameter,path,incoming) in self.program.record_parameter_replacements.get(&function).into_iter().flatten() {
            if !matches!(incoming,Type::Union(_)){continue;}
            let binding=&fun.bindings[*parameter];
            let mut target=binding.input_ty.unwrap_or(binding.ty);
            let mut valid=true;
            for name in path {
                let Type::Record(record)=target else{valid=false;break;};
                let Some(ty)=self.program.records[record].fields.iter().find(|field|field.name.text==*name).and_then(|field|field.ty) else{valid=false;break;};
                target=ty;
            }
            if !valid{continue;}
            let mut pending=vec![(*incoming,target)];
            while let Some((source,target))=pending.pop() {
                match (source,target) {
                    (Type::Union(source),Type::Union(target))=>{
                        let source=self.program.unions[source.index()].members.iter().find(|ty|matches!(ty,Type::Record(_)));
                        let target=self.program.unions[target.index()].members.iter().find(|ty|matches!(ty,Type::Record(_)));
                        if let (Some(&source),Some(&target))=(source,target){pending.push((source,target));}
                    }
                    (Type::Record(source),Type::Record(target))=>{
                        pending.extend(self.program.records[source].fields.iter().zip(&self.program.records[target].fields).filter_map(|(source,target)|Some((source.ty?,target.ty?))));
                    }
                    (source @ (Type::Closure(_) | Type::BorrowedCallable(_)),Type::Callback(id))=>{
                        bounds.push(format!("C{id}:From<{}>",self.native_type(source)));
                    }
                    _=>{}
                }
            }
        }
        bounds.sort();bounds.dedup();bounds
    }
}
