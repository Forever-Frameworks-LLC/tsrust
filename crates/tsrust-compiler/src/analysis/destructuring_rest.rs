use super::*;
impl Analyzer {
    #[inline(never)]
    pub(super) fn pattern_rest(&mut self,context:&mut Context,base:&mut Expr,args:&mut [Expr],loans:&Loans,span:Span)->Result<(Type,CallTarget),Diagnostic>{
        let hinted=self.expression_type_hint(context,base);
        let source=if matches!(hinted,Some(Type::Record(id)) if self.program.records[id].is_tuple()){hinted.unwrap()}else{self.expression(context,base,Mode::Read,loans,None)?};
        let Some(start)=Self::flat_literal_depth(&args[0]) else {unreachable!("parser rest position")};
        let start=start as usize;
        if matches!(source,Type::Array(_)) {
            self.expression(context,&mut args[0],Mode::Copy,loans,Some(Type::Int))?;
            return Ok((source,CallTarget::ArraySlice));
        }
        let Type::Record(id)=source else {return Err(Diagnostic::new("type","Array rest requires an array or tuple.",base.span));};
        if !self.program.records[id].is_tuple(){return Err(Diagnostic::new("type","Array rest requires an array or tuple.",base.span));}
        base.ty=Some(source);
        for field in self.program.records[id].fields.iter().skip(start).cloned().collect::<Vec<_>>(){
            let mut selected=Expr::new(ExprKind::Field(Box::new(base.clone()),field.name.text),base.span);
            self.expression(context,&mut selected,Mode::Read,loans,None)?;
            if let ExprKind::Field(checked,_) = selected.into_kind() {*base=*checked;}
        }
        let fields=self.program.records[id].fields.iter().skip(start).cloned().enumerate().map(|(index,mut field)|{field.name.text=index.to_string();field}).collect::<Vec<_>>();
        let existing=self.program.records.iter().position(|record|record.is_tuple()&&record.fields.len()==fields.len()&&record.fields.iter().zip(&fields).all(|(a,b)|a.ty==b.ty));
        let result=if let Some(existing)=existing{existing}else{
            let mut record=self.program.records[id].clone();record.fields=fields;record.application=None;record.type_parameters.clear();
            let result=self.program.records.len();record.name.text=format!("[rest{result}]");
            self.fields.push(record.fields.iter().map(|f|(f.name.text.clone(),f.ty.expect("checked tuple field"))).collect());
            self.program.records.push(record);result
        };
        self.require_cloneable(Type::Record(result),span)?;
        Ok((Type::Record(result),CallTarget::PatternRest(result,start)))
    }

    #[inline(never)]
    pub(super) fn record_pattern_rest(&mut self,context:&mut Context,base:&mut Expr,args:&mut [Expr],loans:&Loans,span:Span)->Result<(Type,CallTarget),Diagnostic>{
        let hinted=self.expression_type_hint(context,base);
        let source=if matches!(hinted,Some(Type::Record(_))){hinted.unwrap()}else{self.expression(context,base,Mode::Read,loans,None)?};
        let Type::Record(id)=source else{return Err(Diagnostic::new("type","Record rest requires a record.",base.span));};
        if self.program.records[id].is_tuple(){return Err(Diagnostic::new("type","Record rest requires a record.",base.span));}
        self.require_native_capability(source,crate::native_types::Capability::ObjectSpread,base.span,false)?;
        let excluded=args.iter().map(|arg|{let ExprKind::String(key)=&arg.kind else{unreachable!("parser rest field")};key.clone()}).collect::<HashSet<_>>();
        let fields=self.program.records[id].fields.iter().filter(|field|!excluded.contains(&field.name.text)).cloned().collect::<Vec<_>>();
        base.ty=Some(source);
        for field in &fields {
            let mut selected=Expr::new(ExprKind::Field(Box::new(base.clone()),field.name.text.clone()),base.span);
            self.expression(context,&mut selected,Mode::Read,loans,None)?;
            if let ExprKind::Field(checked,_) = selected.into_kind() {*base=*checked;}
        }
        let result=self.spread_record_shape(fields,span);
        self.require_cloneable(Type::Record(result),span)?;
        Ok((Type::Record(result),CallTarget::PatternRest(result,usize::MAX)))
    }
}
