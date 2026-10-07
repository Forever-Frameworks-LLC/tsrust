//! Join concrete record callback environments without changing source ownership.
use super::*;
impl Analyzer {
    pub(super) fn record_branch_carriers(&mut self,context:&mut Context,left:&mut [Stmt],right:&mut [Stmt])->Result<(),Diagnostic>{
        fn assignments(body:&mut [Stmt])->Vec<(&mut Expr,&mut Expr)>{
            let mut result=Vec::new();
            for stmt in body{match &mut stmt.kind{
                StmtKind::Assign{target,value,operator:None} if matches!(target.kind,ExprKind::Name(_))=>result.push((target,value)),
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>result.extend(assignments(body)),
                StmtKind::If{then_body,else_body,..}=>{result.extend(assignments(then_body));result.extend(assignments(else_body));},
                StmtKind::Loop{body,update,..}=>{result.extend(assignments(body));result.extend(assignments(update));},
                StmtKind::Try(region)=>{result.extend(assignments(&mut region.body));if let Some(catch)=&mut region.catch{result.extend(assignments(&mut catch.body));}if let Some(finally)=&mut region.finally{result.extend(assignments(finally));}},
                _=>{}
            }}result
        }
        let mut left=assignments(left);let mut right=assignments(right);
        for(a_target,a_value) in &mut left{
            let Some(binding)=a_target.binding else{continue;};
            for(b_target,b_value) in &mut right{
                if b_target.binding!=Some(binding){continue;}
                if let Some(ty)=self.join_record_values(a_value,b_value)?{context.bindings[binding].ty=ty;a_target.ty=Some(ty);b_target.ty=Some(ty);}
            }
        }
        Ok(())
    }
    pub(super) fn record_sequence_carriers(&mut self,context:&mut Context,body:&mut[Stmt])->Result<(),Diagnostic>{
        let mut writes=Vec::new();
        for stmt in body{match &mut stmt.kind{
            StmtKind::Let{binding:Some(id),value,..} if !value.is_deferred()=>writes.push((*id,value)),
            StmtKind::Assign{target,value,operator:None} if matches!(target.kind,ExprKind::Name(_))=>{if let Some(id)=target.binding{writes.push((id,value));}},
            _=>{}
        }}
        for index in 0..writes.len(){let(before,after)=writes.split_at_mut(index);let(id,value)=&mut after[0];
            if context.bindings[*id].parameter || context.bindings[*id].capture_reference{continue;}
            for(previous,initializer) in before{if previous!=id{continue;}if let Some(ty)=self.join_record_values(initializer,value)?{context.bindings[*id].ty=ty;}}
        }
        Ok(())
    }
    fn join_record_values(&mut self,left:&mut Expr,right:&mut Expr)->Result<Option<Type>,Diagnostic>{
        let(Some(Type::Record(a)),Some(Type::Record(b)))=(left.ty,right.ty) else{return Ok(None);};
        let declaration=self.scoped_records.declaration(a);
        if declaration!=self.scoped_records.declaration(b){return Ok(None);}
        if !matches!((&left.kind,&right.kind),(ExprKind::Record(_),ExprKind::Record(_))){
            let Some(ty)=self.join_record_types(a,b,left.span)? else{return Ok(None);};
            for value in [left,right]{if value.ty==Some(ty){continue;}value.wrap_record_carrier(ty);}
            return Ok(Some(ty));
        }
        let(ExprKind::Record(left_fields),ExprKind::Record(right_fields))=(&mut left.kind,&mut right.kind) else{unreachable!()};
        let mut fields=self.program.records[declaration].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
        let mut changed=false;
        for(index,field) in self.program.records[declaration].fields.clone().iter().enumerate(){
            let Some((_,l))=left_fields.iter_mut().find(|(key,_)|key.text==field.name.text) else{continue;};
            let Some((_,r))=right_fields.iter_mut().find(|(key,_)|key.text==field.name.text) else{continue;};
            let lt=l.ty.unwrap();let rt=r.ty.unwrap();
            if let Some(ty)=self.join_record_values(l,r)?{fields[index]=ty;changed=true;}
            else if lt!=rt && let Some(family)=self.common_borrowed_conditional_family(lt,rt,l.span)?{
                for value in [&mut *l,&mut *r]{if !matches!(value.kind,ExprKind::Unary{op:UnaryOp::CallableStore,..}){value.wrap_storage(family);}value.ty=Some(Type::BorrowedCallable(family));}let ty=Type::BorrowedCallable(family);l.ty=Some(ty);r.ty=Some(ty);fields[index]=ty;changed=true;
            }else if lt==rt{fields[index]=lt;}
        }
        if !changed{return Ok(None);}
        let instance=self.scoped_records.materialize(declaration,fields,&mut self.program.records);
        if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
        let ty=Type::Record(instance);left.ty=Some(ty);right.ty=Some(ty);Ok(Some(ty))
    }
    fn join_record_types(&mut self,a:usize,b:usize,span:Span)->Result<Option<Type>,Diagnostic>{
        let declaration=self.scoped_records.declaration(a);if declaration!=self.scoped_records.declaration(b){return Ok(None);}
        let mut fields=Vec::new();let mut changed=false;
        let pairs=self.program.records[a].fields.iter().zip(&self.program.records[b].fields).map(|(a,b)|(a.ty.unwrap(),b.ty.unwrap())).collect::<Vec<_>>();
        for(left,right) in pairs{
            let joined=if left==right{left}else if let(Type::Record(a),Type::Record(b))=(left,right){self.join_record_types(a,b,span)?.unwrap_or(left)}else if let Some(family)=self.common_borrowed_conditional_family(left,right,span)?{Type::BorrowedCallable(family)}else{return Ok(None);};
            changed|=joined!=left || joined!=right;fields.push(joined);
        }
        if !changed{return Ok(None);}
        let instance=self.scoped_records.materialize(declaration,fields,&mut self.program.records);
        if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
        Ok(Some(Type::Record(instance)))
    }

}
