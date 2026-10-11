//! Keep partial field ownership visible without executing cleanup inside body scopes.
use super::*;
use crate::capture_loan_owners::Place;

pub(super) struct FieldPlan {pub(super) input_fields:Vec<String>,pub(super) output_fields:Vec<String>}
impl Emitter<'_> {
    pub(super) fn field_completion_transport(&self,fun:&CheckedFunction,statement:&Stmt,region:&ExceptionRegion,first:usize,incoming_name:&str,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>)->Option<String> {
        let cleanup=region.finally.as_ref()?;
        let catch=region.catch.as_ref().filter(|handler|handler.error_contract.as_ref().is_some_and(|errors|errors.runtime || errors.opaque || !errors.explicit.is_empty()));
        let FieldPlan{input_fields,output_fields}=self.completion_field_plan(fun,statement,region)?;
        self.completion_next.set(self.completion_next.get().max(first+4));
        let values=tuple(&input_fields);
        let (pattern,restore)=restoration(&input_fields,&format!("input_field{first}"));
        let output_values=tuple(&output_fields);
        let (output_pattern,output_restore)=restoration(&output_fields,&format!("normal_field{first}"));
        let pending=if let Some(handler)=catch {
            let incoming=handler.error_contract.as_deref().expect("checked catch contract");
            let body=self.field_completion_body(fun,&region.body,first,incoming_name,incoming,runtime,"()");
            let handled=self.field_completion_body(fun,&handler.body,first+1,error_name,contract,runtime,&values);
            let setup=if let Some(binding)=handler.binding {
                let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
                let value=exception_contract_emit::checked_catch_value("caught_error",incoming_name,incoming,&unions,runtime,statement.span).expect("checked field catch value");
                format!("let {}value{binding}={value};",if fun.bindings[binding].mutable{"mut "}else{""})
            }else{"drop(caught_error);".into()};
            format!("match {body}{{TsrCompletion::Normal(())=>TsrCompletion::Normal({values}),TsrCompletion::Throw(caught_error)=>{{{setup}{handled}}},TsrCompletion::Return(value)=>TsrCompletion::Return(value),_=>unreachable!(\"checked field catch control\")}}")
        }else{self.field_completion_body(fun,&region.body,first,error_name,contract,runtime,&values)};
        let normal_cleanup=self.field_completion_body(fun,cleanup,first+2,error_name,contract,runtime,&output_values);
        let returned_cleanup=self.field_completion_body(fun,cleanup,first+3,error_name,contract,runtime,"()");
        let returned=exception_emit::finally_region("TsrCompletion::Return(pending_return)",&returned_cleanup);
        let thrown=exception_emit::finally_region("TsrCompletion::Throw(pending_error)",&returned_cleanup);
        let joined=format!("match {pending}{{TsrCompletion::Normal({pattern})=>{{{restore}{normal_cleanup}}},TsrCompletion::Return(pending_return)=>{returned},TsrCompletion::Throw(pending_error)=>{thrown},_=>unreachable!(\"checked field completion control\")}}");
        let (outer_name,outer_contract,_)=self.exception_regions.current_transport().expect("outer field completion");
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        let widened=exception_contract_emit::checked_completion_conversion(&joined,error_name,&outer_name,contract,&outer_contract,&unions,runtime,statement.span).expect("checked field error conversion");
        let returned=self.exception_regions.return_value("value",statement.span);
        let thrown=self.exception_regions.native_throw("error");
        Some(format!("match {widened}{{TsrCompletion::Normal({output_pattern})=>{{{output_restore}}},TsrCompletion::Return(value)=>{{{returned}}},TsrCompletion::Throw(error)=>{{{thrown}}},_=>unreachable!(\"checked field cleanup control\")}}"))
    }
    pub(super) fn completion_field_plan(&self,fun:&CheckedFunction,statement:&Stmt,region:&ExceptionRegion)->Option<FieldPlan> {
        let cleanup=region.finally.as_ref()?;
        let catch=region.catch.as_ref().filter(|handler|handler.error_contract.as_ref().is_some_and(|errors|errors.runtime || errors.opaque || !errors.explicit.is_empty()));
        // Control exits need their own surviving-field payloads.
        if !exception_contract_emit::escaping_controls(&region.body).is_empty()
            || catch.is_some_and(|handler|!exception_contract_emit::escaping_controls(&handler.body).is_empty())
            || !exception_contract_emit::escaping_controls(cleanup).is_empty() {return None;}
        let mut body_places=partial_places(fun,&region.body);
        if let Some(handler)=catch {body_places.extend(partial_places(fun,&handler.body));}
        let mut places=body_places.clone();places.extend(partial_places(fun,cleanup));
        if places.is_empty(){return None;}
        // With no field transfer in the throwing body, original storage remains
        // initialized at catch entry. Body transfers need precise error payloads.
        if catch.is_some() && let Some(start)=region.body.first() {
            let end=crate::error_effects::body_expressions(&region.body).iter().map(|value|value.span.end).chain(region.body.iter().map(|statement|statement.span.end)).max().unwrap_or(start.span.end);
            if fun.storage_exits.expression_points().filter(|point|point.0>=start.span.start && point.1<=end).any(|point|
                body_places.iter().any(|place|fun.storage_exits.transfer_at_expression(point,place.binding,&place.fields).is_some())) {return None;}
        }
        if crate::completion::definitely_abrupt(cleanup,0) || crate::completion::definitely_abrupt(&region.body,0) && catch.is_none_or(|handler|crate::completion::definitely_abrupt(&handler.body,0)) {
            return Some(FieldPlan{input_fields:vec![],output_fields:vec![]});
        }
        normalize(fun,statement.span.start,&mut body_places);
        normalize(fun,statement.span.start,&mut places);
        let boundary=region.body.first().or_else(||cleanup.first())?.span.start;
        use crate::analysis::MapCompletionPhase::{Handled,Cleaned};
        let mut input_fields=vec![];let mut output_fields=vec![];
        for (selection,phase,fields) in [(&body_places,Handled,&mut input_fields),(&places,Cleaned,&mut output_fields)] {
            for place in selection {
                let binding=&fun.bindings[place.binding];
                if binding.capture_reference || binding.borrowed_place.is_some() || binding.mutable_place
                    || binding.parameter && (place.binding>=fun.parameter_count || binding.mode!=Mode::Consume)
                    || self.restored_capture(fun,place.binding) || self.owned_record_layout(fun,place.binding).is_some() {return None;}
                if !fun.storage_exits.completion_field_available(boundary,phase,None,place)?{continue;}
                let mut ty=binding.ty;let mut native=format!("value{}",place.binding);
                for name in &place.fields {
                    let Type::Record(record)=ty else{return None;};
                    let index=self.program.records[record].fields.iter().position(|field|field.name.text==*name)?;
                    ty=self.program.records[record].fields[index].ty?;let _=write!(native,".field{index}");
                }
                fields.push(native);
            }
        }
        Some(FieldPlan{input_fields,output_fields})
    }
    fn field_completion_body(&self,fun:&CheckedFunction,body:&[Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,normal:&str)->String {
        self.completion_next.set(self.completion_next.get().max(id+1));
        let _region=self.exception_regions.enter_typed(id,exception_contract_emit::local_control_targets(body),error_name.into(),contract.clone(),runtime);
        let mut statements=String::new();self.block(&mut statements,fun,body,1);
        format!("{{let completion:TsrCompletion<_,{},{}>='exception_region{id}:{{{statements}TsrCompletion::Normal({normal})}};completion}}",self.native_type(fun.result),error_name)
    }
}
pub(super) fn normalize(fun:&CheckedFunction,start:usize,places:&mut Vec<Place>) {
    places.retain(|place|fun.bindings[place.binding].declaration_span.is_none_or(|span|span.start<start));
    places.sort();places.dedup();
    let all=places.clone();places.retain(|place|!all.iter().any(|parent|parent.binding==place.binding && parent.fields.len()<place.fields.len() && place.fields.starts_with(&parent.fields)));
}
pub(super) fn restoration(fields:&[String],prefix:&str)->(String,String) {
    let names=(0..fields.len()).map(|index|format!("{prefix}_{index}")).collect::<Vec<_>>();
    (tuple(&names),fields.iter().zip(&names).map(|(field,value)|format!("{field}={value};")).collect())
}

pub(super) fn tuple(values:&[String])->String {if values.is_empty(){"()".into()}else{format!("({},)",values.join(","))}}
pub(super) fn partial_places(fun:&CheckedFunction,body:&[Stmt])->Vec<Place> {
    let mut places=vec![];let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop(){match &statement.kind {
        StmtKind::Try(region)=>pending.extend(region.statements()),
        StmtKind::Return(Some(value))=>places.extend(fun.storage_exits.partial_places_within((statement.span.start,statement.span.end),value.span)),
        StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
        StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);},
        StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
        _=>{}
    }}places
}
