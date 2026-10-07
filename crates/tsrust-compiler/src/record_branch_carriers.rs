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
    fn join_record_values(&mut self,left:&mut Expr,right:&mut Expr)->Result<Option<Type>,Diagnostic>{
        let(Some(Type::Record(a)),Some(Type::Record(b)))=(left.ty,right.ty) else{return Ok(None);};
        let declaration=self.scoped_records.declaration(a);
        if declaration!=self.scoped_records.declaration(b){return Ok(None);}
        let(ExprKind::Record(left_fields),ExprKind::Record(right_fields))=(&mut left.kind,&mut right.kind) else{return Ok(None);};
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
}
