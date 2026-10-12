use super::*;
impl Analyzer {
    #[inline(never)]
    fn spread_overwrites(&self,context:&Context,value:&Expr,key:&str,depth:usize)->bool{
        if depth>32{return false;}
        if let Some(Type::Record(id))=self.expression_type_hint(context,value){return self.fields[id].contains_key(key);}
        if let ExprKind::Record(fields)=&value.kind{return fields.iter().any(|(field,value)|field.text==key||(field.text=="record#spread"&&self.spread_overwrites(context,value,key,depth+1)));}
        false
    }
    #[inline(never)]
    fn spread_record_shape(&mut self,fields:Vec<RecordField>,span:Span)->usize{
        if let Some(id)=self.program.records.iter().position(|record|record.native_type.is_none()&&!record.is_tuple()&&record.enum_definition.is_none() && record.set_definition.is_none() && record.map_definition.is_none()&&record.fields.len()==fields.len()&&record.fields.iter().zip(&fields).all(|(a,b)|a.name.text==b.name.text&&a.ty==b.ty)){return id;}
        let id=self.program.records.len();
        self.fields.push(fields.iter().map(|field|(field.name.text.clone(),field.ty.expect("checked spread field"))).collect());
        self.program.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,application:None,name:Token{text:format!("rest#record{id}"),span,kind:TokenKind::Name},type_parameters:Vec::new(),fields});id
    }
    #[inline(never)]
    pub(super) fn object_spread(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        if let Some(Type::Record(id)) = expected
            && self.program.records[id].native_type.is_some_and(|id| !crate::native_types::declaration(id).source_constructible()) {
            return Err(Diagnostic::new("native-opaque", "Opaque native values cannot be constructed with record spread; use their module's constructors.", expr.span));
        }
        let ExprKind::Record(source)=&mut expr.kind else{unreachable!()};
        let mut parts=std::collections::VecDeque::from(std::mem::take(source));let mut args=Vec::new();let mut fields:Vec<RecordField>=Vec::new();
        while let Some((key,mut value))=parts.pop_front() {
            let spread=key.text=="record#spread";
            let part=if spread {
                let prior=self.builtin_literals;self.builtin_literals=true;
                let actual=self.expression(context,&mut value,Mode::Read,loans,None);self.builtin_literals=prior;
                let Type::Record(id)=actual? else{return Err(Diagnostic::new("type","Object spread requires a fixed record.",value.span));};
                if self.program.records[id].is_tuple(){return Err(Diagnostic::new("type","Object spread requires a fixed record.",value.span));}
                self.require_native_capability(Type::Record(id), crate::native_types::Capability::ObjectSpread, value.span, false)?;
                self.require_cloneable(Type::Record(id),value.span)?;
                id
            }else{
                let overwritten=parts.iter().any(|(later,value)|later.text==key.text||(later.text=="record#spread"&&self.spread_overwrites(context,value,&key.text,0)));
                let contextual=if !overwritten && let Some(Type::Record(id))=expected{self.fields[id].get(&key.text).copied()}else{None};
                let ty=self.expression(context,&mut value,Mode::Consume,loans,contextual)?;
                let field=RecordField{name:key.clone(),annotation:crate::type_syntax::ValueAnnotation{kind:crate::type_syntax::ValueAnnotationKind::Named(crate::type_syntax::TypeName{text:key.text.clone(),span:key.span}),span:key.span},ty:Some(ty),callable_declaration:None,optional:false};
                let id=self.spread_record_shape(vec![field],key.span);
                value=Expr::checked(ExprKind::Record(vec![(key.clone(),value)]),key.span,Some(Type::Record(id)));id
            };
            for field in self.program.records[part].fields.clone(){
                if let Some(old)=fields.iter_mut().find(|old|old.name.text==field.name.text){*old=field;}else{fields.push(field);}
            }
            if spread {value=Expr::checked(ExprKind::Unary{op:UnaryOp::Spread,operator:key.span,value:Box::new(value)},key.span,Some(Type::Record(part)));}
            args.push(value);
        }
        let result=if let Some(Type::Record(id))=expected {
            if self.program.records[id].is_tuple()||self.fields[id].len()!=fields.len(){return Err(Diagnostic::new("type","Spread record fields must match its result annotation.",expr.span));}
            for field in &fields{let Some(ty)=self.fields[id].get(&field.name.text).copied()else{return Err(Diagnostic::new("type","Spread record has a field absent from its result annotation.",field.name.span));};self.require(field.ty.expect("checked spread field"),Some(ty),field.name.span)?;}
            id
        }else{self.spread_record_shape(fields,expr.span)};
        let callee=Expr::new(ExprKind::Name("record#spread".into()),expr.span);
        expr.kind=ExprKind::Call(Box::new(callee),args);expr.target=Some(CallTarget::RecordSpread(result));Ok(Type::Record(result))
    }
}
