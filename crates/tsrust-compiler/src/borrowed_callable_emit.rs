//! Native borrowed families preserve each concrete origin's capture lifetimes.
use super::*;
impl Emitter<'_> {
    pub(super) fn borrowed_family_type(&self,id:usize)->String {
        let types=self.borrowed_payloads(id).iter().enumerate().map(|(arm,payload)|self.borrowed_payload_type(payload,&mut Vec::new(),&mut Vec::new(),arm,true)).collect::<Vec<_>>();
        format!("BorrowedCallable{id}<{}>",types.join(", "))
    }
    pub(super) fn emit_borrowed_family(&self,output:&mut String,id:usize,declaration:usize,origins:&[usize]) {
        let contract=&self.program.callback_contracts[declaration];
        let invocation=contract.invocation(&self.program.callable_shapes);
        assert!(origins.len()>=2,"borrowed alternatives retain distinct origins");
        let payloads=self.borrowed_payloads(id);
        let payload_count=payloads.len();
        crate::borrowed_callable_dispatch::emit_definition_many(output,id,payload_count);
        let mut lifetimes=Vec::new();
        let mut capture_bounds=Vec::new();
        let types=payloads.iter().enumerate().map(|(arm,payload)|self.borrowed_payload_type(payload,&mut lifetimes,&mut capture_bounds,arm,false)).collect::<Vec<_>>();
        let name=format!("BorrowedCallable{id}<{}>",types.join(", "));
        let result=self.result_storage.borrow().get(&id).map(|(name,_)|format!("Result<{},{}>",self.native_type(contract.result),name)).unwrap_or_else(||self.native_type(contract.result));
        for(position,ty)in types.iter().enumerate(){
            let arm=crate::borrowed_callable_dispatch::variant(position,payload_count);
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
            for (position,payload) in payloads.iter().enumerate() {
                let origin=payload.origin;
                let arm=crate::borrowed_callable_dispatch::variant(position,payload_count);
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
