//! Native expression carriers are distinct from point-sensitive capture facts.
use super::*;
impl Analyzer {
    pub(super) fn record_value_carrier_type(&mut self,context:&Context,value:&Expr,plans:&HashMap<usize,Type>)->Result<Type,Diagnostic>{
        enum Task<'a> {
            Value(&'a Expr),
            Record {
                record:usize,
                checked:Type,
                fields:Vec<Type>,
                remaining:std::slice::Iter<'a,(Token,Expr)>,
                pending:Option<usize>,
            },
            Wrapper(Type,Span),
            Inject(Type,Span),
        }
        let mut work=vec![Task::Value(value)];
        let mut result=Type::Void;
        while let Some(task)=work.pop(){
            match task {
                Task::Inject(checked,span)=>{
                    if let (Type::Record(actual),Some(Type::Record(expected)))=(result,self.unions.payload(checked))
                        && self.scoped_records.declaration(actual)==self.scoped_records.declaration(expected) {
                        let members=self.unions.members(checked.union().unwrap()).into_iter().map(|ty|if matches!(ty,Type::Record(_)){result}else{ty}).collect::<Vec<_>>();
                        result=self.unions.intern(members,span)?;
                    }else{result=checked;}
                }
                Task::Wrapper(checked,span)=>{
                    if result==checked{continue;}
                    result=if let(Type::Record(a),Type::Record(b))=(result,checked){
                        self.join_record_types(a,b,span)?.unwrap_or(checked)
                    }else if let Some(joined)=self.join_nullable_record_unions(result,checked,span)?{joined
                    }else if let Some(family)=self.common_borrowed_conditional_family(result,checked,span)?{
                        Type::BorrowedCallable(family)
                    }else{checked};
                }
                Task::Record{record,checked,mut fields,mut remaining,pending}=>{
                    if let Some(index)=pending{fields[index]=result;}
                    if let Some((name,expr))=remaining.next(){
                        let index=self.program.records[record].fields.iter().position(|field|field.name.text==name.text).expect("checked record field");
                        work.push(Task::Record{record,checked,fields,remaining,pending:Some(index)});
                        work.push(Task::Value(expr));
                    }else if fields.iter().zip(&self.program.records[record].fields).all(|(ty,field)|Some(*ty)==field.ty){
                        result=checked;
                    }else{
                        let instance=self.scoped_records.materialize(record,fields,&mut self.program.records);
                        if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
                        result=Type::Record(instance);
                    }
                }
                Task::Value(value)=>{
                    // An expression of an unchecked generic body has no storage to plan.
                    let Some(checked)=value.ty else{result=Type::Void;continue;};
                    result=checked;
                    if !matches!(checked,Type::Map(_) | Type::Record(_) | Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) && !matches!(self.unions.payload(checked),Some(Type::Record(_))){continue;}
                    if let(Some(binding),Some((_,path)))=(Self::root_binding(value),Self::direct_fields(value)){
                        let binding_info=&context.bindings[binding];
                        if !binding_info.parameter && !binding_info.capture_reference && let Some(ty)=self.record_field_type(*plans.get(&binding).unwrap_or(&binding_info.ty),&path) && (matches!((checked,ty),(Type::Record(_),Type::Record(_)) | (Type::Map(_),Type::Map(_))) || matches!((checked,ty),(Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_),Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)))){result=ty;continue;}
                    }
                    match &value.kind{
                        ExprKind::Inject{value:inner,..} if matches!(self.unions.payload(checked),Some(Type::Record(_)))=>{
                            work.push(Task::Inject(checked,value.span));work.push(Task::Value(inner));
                        }
                        ExprKind::Record(values)=>{
                            let Type::Record(record)=checked else{continue;};
                            let fields=self.program.records[record].fields.iter().map(|field|field.ty.unwrap()).collect();
                            work.push(Task::Record{record,checked,fields,remaining:values.iter(),pending:None});
                        }
                        ExprKind::Sequence(values)=>work.push(Task::Value(values.last().expect("nonempty sequence"))),
                        ExprKind::Unary{op:UnaryOp::RecordCarrier | UnaryOp::CallableStore,value:inner,..} if matches!(checked,Type::Record(_) | Type::BorrowedCallable(_)) || matches!(self.unions.payload(checked),Some(Type::Record(_)))=>{
                            work.push(Task::Wrapper(checked,value.span));
                            work.push(Task::Value(inner));
                        }
                        _=>{}
                    }
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyzer()->Analyzer {
        Analyzer::new(crate::syntax::parse("type Item={value:int;keep:int;};function main(){}").unwrap()).unwrap()
    }

    #[test]
    fn deep_carrier_wrappers_preserve_type_without_recursive_walk() {
        let mut analyzer=analyzer();
        let context=Context::new("walk".into());
        let mut value=Expr::new(ExprKind::Name("item".into()),Span::default());
        value.ty=Some(Type::Record(0));
        for _ in 0..16384 {
            value=Expr::new(ExprKind::Unary{op:UnaryOp::RecordCarrier,operator:Span::default(),value:Box::new(value)},Span::default());
            value.ty=Some(Type::Record(0));
        }
        let count=analyzer.program.records.len();
        assert_eq!(analyzer.record_value_carrier_type(&context,&value,&HashMap::new()).unwrap(),Type::Record(0));
        assert_eq!(analyzer.program.records.len(),count);
        drop(value);
    }

    #[test]
    fn record_carrier_fields_complete_before_materialization() {
        let mut analyzer=analyzer();
        let context=Context::new("walk".into());
        let names=analyzer.program.records[0].fields.iter().map(|field|field.name.clone()).collect::<Vec<_>>();
        let mut changed=Expr::new(ExprKind::Null,Span::default());changed.ty=Some(Type::Boolean);
        let mut kept=Expr::new(ExprKind::Null,Span::default());kept.ty=Some(Type::Int);
        let mut value=Expr::new(ExprKind::Record(vec![(names[0].clone(),changed),(names[1].clone(),kept)]),Span::default());value.ty=Some(Type::Record(0));
        let Type::Record(result)=analyzer.record_value_carrier_type(&context,&value,&HashMap::new()).unwrap() else{panic!("record carrier")};
        assert_eq!(analyzer.program.records[result].fields[0].ty,Some(Type::Boolean));
        assert_eq!(analyzer.program.records[result].fields[1].ty,Some(Type::Int));
        assert_eq!(analyzer.program.records[0].fields[0].ty,Some(Type::Int));
        assert_eq!(value.ty,Some(Type::Record(0)));
    }

    #[test]
    fn unchecked_values_and_sequence_prefixes_keep_existing_rules() {
        let mut analyzer=analyzer();let context=Context::new("walk".into());
        let unchecked=Expr::new(ExprKind::Null,Span::default());
        assert_eq!(analyzer.record_value_carrier_type(&context,&unchecked,&HashMap::new()).unwrap(),Type::Void);
        let mut last=Expr::new(ExprKind::Null,Span::default());last.ty=Some(Type::Int);
        let mut sequence=Expr::new(ExprKind::Sequence(vec![unchecked,last]),Span::default());sequence.ty=Some(Type::Record(0));
        assert_eq!(analyzer.record_value_carrier_type(&context,&sequence,&HashMap::new()).unwrap(),Type::Int);
    }
}

#[cfg(test)]
mod nullable_injection_tests {
    use super::*;
    #[test]
    fn injected_record_physical_carrier_is_not_hidden_by_checked_union(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Item={value:int;};function main(){}").unwrap()).unwrap();
        let live=analyzer.scoped_records.materialize(0,vec![Type::Boolean],&mut analyzer.program.records);
        let span=Span::default();let checked=analyzer.unions.intern([Type::Record(0),Type::Undefined,Type::Null],span).unwrap();
        let mut payload=Expr::new(ExprKind::Name("physical".into()),span);payload.ty=Some(Type::Record(live));
        let mut value=Expr::new(ExprKind::Inject{value:Box::new(payload),union:checked.union().unwrap()},span);value.ty=Some(checked);
        let result=analyzer.record_value_carrier_type(&Context::new("probe".into()),&value,&HashMap::new()).unwrap();
        let actual=analyzer.unions.members(result.union().unwrap());assert_eq!(actual.len(),3);for ty in [Type::Record(live),Type::Null,Type::Undefined]{assert!(actual.contains(&ty));}
        assert_eq!(value.ty,Some(checked),"query must not publish a new checked expression");
    }
}
