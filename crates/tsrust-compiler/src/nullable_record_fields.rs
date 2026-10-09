//! Point-specific nullable callback-record fields and pending storage diagnostics.
use super::*;

impl Analyzer {
    pub(super) fn nullable_record_field_paths(&self, ty: Type) -> Vec<Vec<String>> {
        let mut pending=vec![(ty,Vec::new())];
        let mut result=Vec::new();
        while let Some((ty,path))=pending.pop() {
            if !path.is_empty() && self.nullable_scoped_record_declaration(ty).is_some() { result.push(path);continue; }
            if let Type::Record(record)=ty {
                for field in &self.program.records[record].fields {
                    let mut path=path.clone();path.push(field.name.text.clone());
                    pending.push((field.ty.unwrap(),path));
                }
            }
        }
        result
    }

    pub(super) fn nullable_record_initializer(&mut self, value: &mut Expr) -> Option<Type> {
        let baseline=value.ty?;
        let paths=self.nullable_record_field_paths(baseline);
        if paths.is_empty(){return None;}
        while matches!(value.kind,ExprKind::Unary{op:UnaryOp::RecordCarrier,..}) {
            let span=value.span;
            let old=std::mem::replace(value,Expr::new(ExprKind::Null,span));
            let ExprKind::Unary{value:inner,..}=old.kind else{unreachable!()};
            *value=*inner;
        }
        let actual=value.ty?;
        let Type::Record(mut record)=baseline else{return None;};
        for path in paths {
            let field=self.record_field_type(actual,&path)?;
            record=self.record_type_with_field(record,&path,field)?;
        }
        let point=Type::Record(record);
        if point!=actual {value.wrap_record_carrier(point);}
        Some(point)
    }

    pub(super) fn nullable_record_field_write(&mut self, context: &mut Context, statement: &mut Stmt, results: &mut HashMap<usize,Type>) -> Result<(),Diagnostic> {
        let StmtKind::Assign{target,value,operator:None}=&mut statement.kind else{return Ok(());};
        let Some((_,path))=Self::direct_fields(target) else{return Ok(());};
        if path.is_empty(){return Ok(());}
        let Some(binding)=Self::root_binding(target) else{return Ok(());};
        let Some(Type::Record(record))=results.get(&binding).copied() else{return Ok(());};
        let Some(previous)=self.record_field_type(Type::Record(record),&path) else{return Ok(());};
        if self.nullable_scoped_record_declaration(previous).is_none(){return Ok(());}
        if let Some(next)=self.array_result_replacement_type(value,None,previous,results)? {
            if previous!=next {context.storage_exits.record_array_result_write(statement.span,next);}
            let record=self.record_type_with_field(record,&path,next).expect("checked nullable field path");
            results.insert(binding,Type::Record(record));
        }
        Ok(())
    }
}

impl Analyzer {
    /// Until native joins exist, only direct field stores in normal lexical
    /// sequences may change a nullable callback-record environment.
    pub(super) fn reject_unplanned_nullable_field_stores(&self, context: &Context, body: &[Stmt]) -> Result<(),Diagnostic> {
        let mut declarations=HashSet::new();
        let mut nested=body.iter().collect::<Vec<_>>();
        while let Some(statement)=nested.pop() {
            match &statement.kind {
                StmtKind::Let{binding:Some(binding),value,..}=>{declarations.insert((*binding,value.span.start,value.span.end));}
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>nested.extend(body),
                StmtKind::If{then_body,else_body,..}=>{nested.extend(then_body);nested.extend(else_body);}
                StmtKind::Loop{body,update,..}=>{nested.extend(body);nested.extend(update);}
                StmtKind::Try(region)=>nested.extend(region.statements()),
                _=>{}
            }
        }
        let mut direct=HashSet::new();
        let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop() {
            match &statement.kind {
                StmtKind::Block(body) if statement.labels.is_empty()=>pending.extend(body),
                StmtKind::Assign{target,value,operator:None} if statement.labels.is_empty() && Self::direct_fields(target).is_some()=>{
                    if let Some(binding)=Self::root_binding(target) {direct.insert((binding,value.span.start,value.span.end));}
                }
                _=>{}
            }
        }
        for write in record_storage_writes::collect(body) {
            let binding=&context.bindings[write.binding];
            if write.path.is_empty() {
                if !declarations.contains(&(write.binding,write.span.start,write.span.end))
                    && (!self.nullable_record_field_paths(binding.ty).is_empty()
                        || write.array_element && !self.nullable_record_field_paths(write.value.ty.unwrap()).is_empty()) {
                    return Err(Diagnostic::new("unsupported","Whole-record replacement of nullable callback fields needs live-field disposal planning; replace individual fields directly for now.",write.span));
                }
                continue;
            }
            let mut ty=binding.ty;
            let mut array=false;
            for name in &write.path {
                while let Type::Array(element)=ty {ty=element.ty();array=true;}
                let Type::Record(record)=ty else{break;};
                let Some(next)=self.program.records[record].fields.iter().find(|field|field.name.text==*name).and_then(|field|field.ty) else{break;};
                ty=next;
            }
            if self.nullable_scoped_record_declaration(ty).is_some()
                && (array || binding.parameter || binding.capture_reference || !direct.contains(&(write.binding,write.span.start,write.span.end))) {
                return Err(Diagnostic::new("unsupported","Nullable callback-record field replacement needs a direct local assignment in a normal lexical sequence; branch, loop, indexed and borrowed field storage joins are not supported yet.",write.span));
            }
        }
        Ok(())
    }
}
