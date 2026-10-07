//! Native expression carriers are distinct from point-sensitive capture facts.
use super::*;
impl Analyzer {
    pub(super) fn record_value_carrier_type(&mut self,context:&Context,value:&Expr,plans:&HashMap<usize,Type>)->Result<Type,Diagnostic>{
        let checked=value.ty.expect("checked record storage value");
        if !matches!(checked,Type::Record(_) | Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)){return Ok(checked);}
        if let(Some(binding),Some((_,path)))=(Self::root_binding(value),Self::direct_fields(value)){
            let binding_info=&context.bindings[binding];
            if !binding_info.parameter && !binding_info.capture_reference && let Some(ty)=self.record_field_type(*plans.get(&binding).unwrap_or(&binding_info.ty),&path) && (matches!((checked,ty),(Type::Record(_),Type::Record(_))) || matches!((checked,ty),(Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_),Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)))){return Ok(ty);}
        }
        match &value.kind{
            ExprKind::Record(values)=>{
                let Type::Record(record)=checked else{return Ok(checked);};
                let mut fields=self.program.records[record].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
                for(name,expr)in values{
                    let index=self.program.records[record].fields.iter().position(|field|field.name.text==name.text).expect("checked record field");
                    fields[index]=self.record_value_carrier_type(context,expr,plans)?;
                }
                if fields.iter().zip(&self.program.records[record].fields).all(|(ty,field)|Some(*ty)==field.ty){return Ok(checked);}
                let instance=self.scoped_records.materialize(record,fields,&mut self.program.records);
                if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
                Ok(Type::Record(instance))
            }
            ExprKind::Sequence(values)=>self.record_value_carrier_type(context,values.last().expect("nonempty sequence"),plans),
            ExprKind::Unary{op:UnaryOp::RecordCarrier | UnaryOp::CallableStore,value:inner,..} if matches!(checked,Type::Record(_) | Type::BorrowedCallable(_))=>{
                let source=self.record_value_carrier_type(context,inner,plans)?;
                if source==checked{return Ok(checked);}
                if let(Type::Record(a),Type::Record(b))=(source,checked){return Ok(self.join_record_types(a,b,value.span)?.unwrap_or(checked));}
                if let Some(family)=self.common_borrowed_conditional_family(source,checked,value.span)?{return Ok(Type::BorrowedCallable(family));}
                Ok(checked)
            }
            _=>Ok(checked)
        }
    }
}
