//! Borrowed families adapt checked failure payloads at callback boundaries.
use super::*;
impl Emitter<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn borrowed_callback_result_adapters(&self,output:&mut String,id:usize,contract:&crate::analysis::CallbackContract,name:&str,generics:&str,arguments:&str,forwarded:&str,native_result:&str,traits:&[(&str,&str,&str)]) {
        let Some(source)=self.program.error_effects.storage_errors.get(&id) else {return;};
        let source_name=self.result_storage.borrow().get(&id).map(|(name,_)|name.clone());
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let registry=self.result_callbacks.borrow();
        let mut callbacks=registry.iter().collect::<Vec<_>>();callbacks.sort_by_key(|(id,_)|**id);
        let mut emitted=std::collections::HashSet::new();
        for (callback,(destination_name,destination)) in callbacks {
            let checked=&self.program.callback_contracts[*callback];
            if checked.result!=contract.result || checked.params.len()!=contract.params.len() || !checked.params.iter().zip(&contract.params).all(|(a,b)|a.ty==b.ty && a.mode==b.mode && a.rest==b.rest) {continue;}
            let result=format!("Result<{},{}>",self.native_type(checked.result),destination_name);
            if result==native_result || !emitted.insert(result.clone()) {continue;}
            let Ok(mapping)=source.native_injection(destination,&unions,runtime,crate::Span::default()) else {continue;};
            let convert=|call:String|if let Some(source_name)=&source_name {
                let conversion=exception_emit::error_conversion("error",source_name,destination_name,&mapping);
                format!("{call}.map_err(|error|{conversion})")
            }else{format!("Ok({call})")};
            for &(trait_name,method,receiver) in traits {
                let body=convert(format!("self.call({forwarded})"));
                let _=writeln!(output,"impl{generics} {trait_name}<{arguments},{result}> for {name} {{fn {method}({receiver},args:{arguments})->{result} {{{body}}}");
                if self.counted_calls {
                    let body=convert(format!("self.call_with_count(count,{forwarded})"));
                    let _=writeln!(output,"fn {method}_with_count({receiver},args:{arguments},count:usize)->{result} {{{body}}}");
                }
                output.push_str("}\n");
            }
        }
    }
}
