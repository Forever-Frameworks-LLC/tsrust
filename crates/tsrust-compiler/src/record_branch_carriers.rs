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
        fn expression_writes(expr:&mut Expr)->Vec<(usize,Vec<String>,&mut Expr)>{
            let mut result=Vec::new();
            match &mut expr.kind{
                ExprKind::Assign{target,value,..}=>{
                    let selected=Analyzer::root_binding(target).zip(Analyzer::direct_fields(target)).filter(|(_,(_,path))|matches!(value.ty,Some(Type::Record(_))) || !path.is_empty() && matches!(value.ty,Some(Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_))));
                    if let Some((id,(_,path)))=selected{result.push((id,path,value.as_mut()));}
                    else{result.extend(expression_writes(target));result.extend(expression_writes(value));}
                },
                ExprKind::Sequence(values)|ExprKind::Array(values)|ExprKind::ArrayOf(values,_)|ExprKind::SetConstructor(values,_)|ExprKind::MapConstructor(values,_)|ExprKind::Template(values)=>{for value in values{result.extend(expression_writes(value));}},
                ExprKind::Unary{value,..}|ExprKind::OptionalChain(value)|ExprKind::TypeAssertion{value,..}|ExprKind::Project{value,..}|ExprKind::Inject{value,..}|ExprKind::Field(value,_)|ExprKind::Update{target:value,..}=>result.extend(expression_writes(value)),
                ExprKind::Binary{left,right,..}|ExprKind::Index(left,right)|ExprKind::CompoundAssign{target:left,value:right,..}|ExprKind::CoalesceAssign{target:left,value:right,..}|ExprKind::LogicalAssign{target:left,value:right,..}|ExprKind::Optional{base:left,body:right,..}=>{result.extend(expression_writes(left));result.extend(expression_writes(right));},
                ExprKind::Conditional{predicate,consequent,alternate}=>{result.extend(expression_writes(predicate));result.extend(expression_writes(consequent));result.extend(expression_writes(alternate));},
                ExprKind::Call(callee,args)=>{result.extend(expression_writes(callee));for value in args{result.extend(expression_writes(value));}},
                ExprKind::Record(fields)=>{for(_,value)in fields{result.extend(expression_writes(value));}},
                ExprKind::SpreadCall(plan)=>{for value in plan.expressions_mut(){result.extend(expression_writes(value));}},
                _=>{}
            }result
        }
        fn writes(body:&mut[Stmt])->Vec<(usize,Vec<String>,&mut Expr)>{
            let mut result=Vec::new();
            for stmt in body{match &mut stmt.kind{
                StmtKind::Let{binding,value,..}=>{if let Some(id)=binding && matches!(value.ty,Some(Type::Record(_))) && !value.is_deferred(){result.push((*id,Vec::new(),value));}else{result.extend(expression_writes(value));}},
                StmtKind::Expr(value)|StmtKind::Throw(value)|StmtKind::Return(Some(value))=>result.extend(expression_writes(value)),
                StmtKind::Assign{target,value,operator:None}=>{if let(Some(id),Some((_,path)))=(Analyzer::root_binding(target),Analyzer::direct_fields(target)){if matches!(value.ty,Some(Type::Record(_))) || !path.is_empty() && matches!(value.ty,Some(Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_))){result.push((id,path,value));}}},
                StmtKind::Block(body)=>result.extend(writes(body)),
                StmtKind::While{condition,body}=>{result.extend(expression_writes(condition));result.extend(writes(body));},
                StmtKind::ForOf{iterable,body,..}=>{result.extend(expression_writes(iterable));result.extend(writes(body));},
                StmtKind::If{condition,then_body,else_body}=>{result.extend(expression_writes(condition));result.extend(writes(then_body));result.extend(writes(else_body));},
                StmtKind::Loop{condition,body,update,post_test,..}=>{if !*post_test{if let Some(condition)=condition{result.extend(expression_writes(condition));}result.extend(writes(body));result.extend(writes(update));}else{result.extend(writes(body));result.extend(writes(update));if let Some(condition)=condition{result.extend(expression_writes(condition));}}},
                StmtKind::Try(region)=>{result.extend(writes(&mut region.body));if let Some(catch)=&mut region.catch{result.extend(writes(&mut catch.body));}if let Some(finally)=&mut region.finally{result.extend(writes(finally));}},
                _=>{}
            }}result
        }
        let mut writes=writes(body);
        for index in 0..writes.len(){let(before,after)=writes.split_at_mut(index);let(id,path,value)=&mut after[0];
            if context.bindings[*id].parameter || context.bindings[*id].capture_reference{continue;}
            for(previous,previous_path,initializer) in before{
                if previous!=id{continue;}
                if path.is_empty() && previous_path.is_empty(){if let Some(ty)=self.join_record_values(initializer,value)?{context.bindings[*id].ty=ty;}}
                else if previous_path.is_empty(){if let Some(ty)=self.join_record_field_write(initializer,path,value)?{context.bindings[*id].ty=ty;}}
                else if path.is_empty(){if let Some(ty)=self.join_record_field_write(value,previous_path,initializer)?{context.bindings[*id].ty=ty;}}
                else if path==previous_path && let Some(family)=self.common_borrowed_conditional_family(initializer.ty.unwrap(),value.ty.unwrap(),value.span)?{
                    for expr in [&mut **initializer,&mut **value]{Self::retarget_record_callback(expr,family);}
                    if let Type::Record(record)=context.bindings[*id].ty && let Some(record)=self.record_type_with_field(record,path,Type::BorrowedCallable(family)){context.bindings[*id].ty=Type::Record(record);}
                }
            }
        }
        Ok(())
    }
    fn retarget_record_callback(value:&mut Expr,family:usize){
        if !matches!(value.kind,ExprKind::Unary{op:UnaryOp::CallableStore,..}){value.wrap_storage(family);}
        value.ty=Some(Type::BorrowedCallable(family));
    }
    fn join_record_field_write(&mut self,record:&mut Expr,path:&[String],value:&mut Expr)->Result<Option<Type>,Diagnostic>{
        let Some(Type::Record(root))=record.ty else{return Ok(None);};
        let mut ty=Type::Record(root);
        for name in path{let Type::Record(id)=ty else{return Ok(None);};let Some(field)=self.program.records[id].fields.iter().find(|field|&field.name.text==name) else{return Ok(None);};ty=field.ty.unwrap();}
        let Some(family)=self.common_borrowed_conditional_family(ty,value.ty.unwrap(),value.span)? else{return Ok(None);};
        let Some(id)=self.record_type_with_field(root,path,Type::BorrowedCallable(family))else{return Ok(None);};
        let ty=Type::Record(id);if record.ty!=Some(ty){record.wrap_record_carrier(ty);}
        Self::retarget_record_callback(value,family);
        Ok(Some(ty))
    }
    pub(super) fn local_record_field_contract(&self,context:&Context,target:&Expr)->Option<Type>{
        let id=Self::root_binding(target)?;let binding=&context.bindings[id];
        if binding.parameter || binding.capture_reference{return None;}
        let(_,path)=Self::direct_fields(target)?;if path.is_empty(){return None;}
        let mut ty=binding.ty;
        for name in path{let Type::Record(record)=ty else{return None;};let record=self.scoped_records.declaration(record);ty=self.program.records[record].fields.iter().find(|field|field.name.text==name)?.ty?;}
        matches!(ty,Type::Callback(_)).then_some(ty)
    }
    fn record_type_with_field(&mut self,record:usize,path:&[String],ty:Type)->Option<usize>{
        let name=path.first()?;let index=self.program.records[record].fields.iter().position(|field|&field.name.text==name)?;
        let mut fields=self.program.records[record].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
        fields[index]=if path.len()==1{ty}else{let Type::Record(child)=fields[index]else{return None;};Type::Record(self.record_type_with_field(child,&path[1..],ty)?)};
        let id=self.scoped_records.materialize(record,fields,&mut self.program.records);
        if self.fields.len()<=id{self.fields.push(self.program.records[id].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
        Some(id)
    }
    pub(super) fn update_local_record_field_type(&mut self,context:&mut Context,target:&Expr,ty:Type)->Result<(),Diagnostic>{
        if self.local_record_field_contract(context,target).is_none(){return Ok(());}
        let id=Self::root_binding(target).unwrap();let(_,path)=Self::direct_fields(target).unwrap();
        let Type::Record(record)=context.bindings[id].ty else{return Ok(());};
        if let Some(record)=self.record_type_with_field(record,&path,ty){context.bindings[id].ty=Type::Record(record);}
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
