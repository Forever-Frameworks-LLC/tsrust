//! Native borrowed families preserve each concrete origin's capture lifetimes.
use super::*;
impl Emitter<'_> {
    fn borrowed_origin_type(&self,origin:usize,lifetimes:&[String])->String {
        let mut arguments = lifetimes.to_vec();
        arguments.extend(self.captured_callback_bounds(&self.program.closures[origin]).iter().map(|_| "_".to_owned()));
        format!("Closure{origin}{}", lifetime_parameters(&arguments))
    }
    pub(super) fn borrowed_family_type(&self,id:usize,lifetime:&str)->String {
        let origins=self.program.stored_contracts.origins(id).expect("borrowed origins");
        let types=origins.iter().map(|&origin|self.borrowed_origin_type(origin,&vec![lifetime.to_owned();self.program.closures[origin].lifetime_count])).collect::<Vec<_>>();
        format!("BorrowedCallable{id}<{}>",types.join(", "))
    }
    pub(super) fn emit_borrowed_family(&self,output:&mut String,id:usize,declaration:usize,origins:&[usize]) {
        let contract=&self.program.callback_contracts[declaration];
        let invocation=contract.invocation(&self.program.callable_shapes);
        assert!(origins.len()>=2,"borrowed alternatives retain distinct origins");
        crate::borrowed_callable_dispatch::emit_definition_many(output,id,origins.len());
        let mut lifetimes=Vec::new();
        let mut capture_bounds=Vec::new();
        let types=origins.iter().enumerate().map(|(arm,&origin)|{
            let names=(0..self.program.closures[origin].lifetime_count).map(|capture|format!("'b{arm}c{capture}")).collect::<Vec<_>>();
            lifetimes.extend(names.iter().cloned());
            let mut arguments = names;
            for bound in self.captured_callback_bounds(&self.program.closures[origin]) {
                let (parameter, _) = bound.split_once(':').expect("capture bound");
                let renamed = format!("B{arm}{parameter}");
                arguments.push(renamed.clone());
                capture_bounds.push(bound.replacen(parameter, &renamed, 1));
            }
            format!("Closure{origin}{}", lifetime_parameters(&arguments))
        }).collect::<Vec<_>>();
        let name=format!("BorrowedCallable{id}<{}>",types.join(", "));
        let result=self.result_storage.borrow().get(&id).map(|(name,_)|format!("Result<{},{}>",self.native_type(contract.result),name)).unwrap_or_else(||self.native_type(contract.result));
        for(position,ty)in types.iter().enumerate(){
            let arm=crate::borrowed_callable_dispatch::variant(position,origins.len());
            let _=writeln!(output,"impl{} From<{ty}> for {name} {{fn from(value:{ty})->Self{{Self::{arm}(value)}}}}",lifetime_parameters(&lifetimes.iter().chain(&capture_bounds).cloned().collect::<Vec<_>>()));
        }
        let receiver=match invocation {Mode::Read=>"&self",Mode::Consume=>"mut self",_=>"&mut self"};
        let parameters=contract.params.iter().enumerate().map(|(i,&parameter)|format!(", argument{i}: {}",self.native_argument_type(parameter,""))).collect::<String>();
        let arguments=(0..contract.params.len()).map(|i|format!("argument{i}")).collect::<Vec<_>>().join(", ");
        let _=writeln!(output,"impl{} {name} {{",lifetime_parameters(&lifetimes.iter().chain(&capture_bounds).cloned().collect::<Vec<_>>()));
        for counted in [false,true].into_iter().filter(|counted|!*counted || self.counted_calls) {
            let method=if counted {"call_with_count"}else{"call"};
            let count=if counted {", count: usize"}else{""};
            let passed=if counted {if arguments.is_empty(){"count".to_owned()}else{format!("count, {arguments}")}}else{arguments.clone()};
            let _=writeln!(output,"fn {method}({receiver}{count}{parameters})->{result} {{match self {{");
            for (position,&origin) in origins.iter().enumerate() {
                let arm=crate::borrowed_callable_dispatch::variant(position,origins.len());
                let call=self.stored_origin_result(id,origin,format!("value.{method}({passed})"));
                let mutable=if invocation==Mode::Consume {"mut "}else{""};
                let _=writeln!(output,"Self::{arm}({mutable}value)=>{call},");
            }
            output.push_str("}}\n");
        }
        output.push_str("}\n");
        let (arguments,argument_lifetimes)=self.native_argument_tuple(&contract.params);
        let forwarded=(0..contract.params.len()).map(|i|format!("args.{i}")).collect::<Vec<_>>().join(", ");
        lifetimes.extend(argument_lifetimes);
        let traits=if invocation==Mode::Consume {vec![("OnceCallback","invoke_once","self")]}else if invocation==Mode::Read {vec![("ReadCallback","invoke_read","&self"),("Callback","invoke","&mut self")]}else{vec![("Callback","invoke","&mut self")]};
        for (trait_name,method,receiver) in &traits {
            let _=writeln!(output,"impl{} {trait_name}<{arguments},{result}> for {name} {{fn {method}({receiver},args:{arguments})->{result} {{self.call({forwarded})}}",lifetime_parameters(&lifetimes.iter().chain(&capture_bounds).cloned().collect::<Vec<_>>()));
            if self.counted_calls {
                let args=if forwarded.is_empty(){"count".to_owned()}else{format!("count,{forwarded}")};
                let _=writeln!(output,"fn {method}_with_count({receiver},args:{arguments},count:usize)->{result} {{self.call_with_count({args})}}");
            }
            output.push_str("}\n");
        }
        self.callback_family_result_adapters(output,id,contract,&name,&lifetime_parameters(&lifetimes.iter().chain(&capture_bounds).cloned().collect::<Vec<_>>()),&arguments,&forwarded,&result,&traits);
    }
}
