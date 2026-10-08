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
    /// A callback moved out of a field keeps the carrier that the field holds
    /// once every store into that field is planned.
    fn record_field_move_carriers(&self,context:&mut Context,body:&mut[Stmt]){
        for stmt in body{match &mut stmt.kind{
            StmtKind::Let{binding:Some(id),value,..}=>{
                if !matches!(context.bindings[*id].ty,Type::Closure(_) | Type::BorrowedCallable(_)){continue;}
                let(Some(root),Some((_,path)))=(Self::root_binding(value),Self::direct_fields(value))else{continue;};
                if path.is_empty() || context.bindings[root].parameter || context.bindings[root].capture_reference{continue;}
                let Some(carrier @ Type::BorrowedCallable(_))=self.record_field_type(context.bindings[root].ty,&path)else{continue;};
                context.bindings[*id].ty=carrier;value.ty=Some(carrier);
            }
            StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>self.record_field_move_carriers(context,body),
            StmtKind::If{then_body,else_body,..}=>{self.record_field_move_carriers(context,then_body);self.record_field_move_carriers(context,else_body);}
            StmtKind::Loop{body,update,..}=>{self.record_field_move_carriers(context,body);self.record_field_move_carriers(context,update);}
            StmtKind::Try(region)=>{
                self.record_field_move_carriers(context,&mut region.body);
                if let Some(catch)=&mut region.catch{self.record_field_move_carriers(context,&mut catch.body);}
                if let Some(finally)=&mut region.finally{self.record_field_move_carriers(context,finally);}
            }
            _=>{}
        }}
    }
    pub(super) fn record_sequence_carriers(&mut self,context:&mut Context,body:&mut[Stmt])->Result<(),Diagnostic>{
        let writes=record_storage_writes::collect(body).into_iter().filter(|write|!write.array_element).collect::<Vec<_>>();
        let mut plans=HashMap::new();
        // Carrier sets only widen. A moved source can gain origins after its
        // consumer was visited, so propagate storage dependencies to a fixed point.
        loop {
            let mut changed=false;
            for write in &writes {
                let binding=&context.bindings[write.binding];
                if binding.parameter || binding.capture_reference{continue;}
                let ty=*plans.entry(write.binding).or_insert(binding.ty);
                let Type::Record(record)=ty else{continue;};
                if write.value.ty.is_none(){continue;}
                let incoming=self.record_value_carrier_type(context,write.value,&plans)?;
                let joined=if write.path.is_empty(){
                    let Type::Record(incoming)=incoming else{continue;};
                    self.join_record_types(record,incoming,write.span)?
                }else{
                    let Some(previous)=self.record_field_type(ty,&write.path)else{continue;};
                    if previous==incoming{None}
                    else if let Some(family)=self.common_borrowed_conditional_family(previous,incoming,write.span)?{
                        self.record_type_with_field(record,&write.path,Type::BorrowedCallable(family)).map(Type::Record)
                    }else{None}
                };
                if let Some(joined)=joined && joined!=ty{plans.insert(write.binding,joined);changed=true;}
            }
            if !changed{break;}
        }
        let mut destinations=HashMap::new();
        for write in writes {
            let Some(&plan)=plans.get(&write.binding)else{continue;};
            let Some(destination)=self.record_field_type(plan,&write.path)else{continue;};
            destinations.insert((write.span.start,write.span.end),destination);
        }
        for(&id,&ty)in &plans{context.bindings[id].ty=ty;}
        self.record_field_move_carriers(context,body);
        // Uses of a retyped local dispatch through its carrier.
        crate::source_walk::body(body,&mut |_|Ok(()),&mut |value|{
            if let Some(CallTarget::Closure(binding))=value.target && matches!(context.bindings[binding].ty,Type::BorrowedCallable(_)){
                value.target=Some(CallTarget::StoredCallable(binding));
            }
            if let(ExprKind::Name(_),Some(binding),Some(Type::Closure(_)))=(&value.kind,value.binding,value.ty) && let carrier @ Type::BorrowedCallable(_)=context.bindings[binding].ty{
                value.ty=Some(carrier);
            }
            Ok(())
        })?;
        crate::source_walk::body(body,&mut |_|Ok(()),&mut |value|{
            if value.ty.is_none(){return Ok(());}
            let stored_family=if let ExprKind::Unary{op:UnaryOp::CallableStore,value:inner,..}=&value.kind{
                matches!(value.ty,Some(Type::BorrowedCallable(_))) && matches!(self.record_value_carrier_type(context,inner,&plans)?,Type::BorrowedCallable(_))
            }else{false};
            if stored_family && let ExprKind::Unary{op,..}=&mut value.kind{*op=UnaryOp::RecordCarrier;}
            if matches!(value.kind,ExprKind::Record(_) | ExprKind::Unary{op:UnaryOp::RecordCarrier,..}){
                value.ty=Some(self.record_value_carrier_type(context,value,&plans)?);
            }
            let Some(destination)=destinations.remove(&(value.span.start,value.span.end))else{return Ok(());};
            let source=self.record_value_carrier_type(context,value,&plans)?;
            if source==destination{return Ok(());}
            match(source,destination){
                (Type::Closure(_),Type::BorrowedCallable(family))=>Self::retarget_record_callback(value,family),
                (Type::Record(_),Type::Record(_))|(Type::BorrowedCallable(_),Type::BorrowedCallable(_))=>value.wrap_record_carrier(destination),
                _=>{}
            }
            Ok(())
        })?;
        self.scoped_map_carriers(context,body)?;
        self.record_array_carriers(context,body)
    }
    pub(super) fn record_field_type(&self,mut ty:Type,path:&[String])->Option<Type>{
        for name in path{let Type::Record(record)=ty else{return None;};ty=self.program.records[record].fields.iter().find(|field|field.name.text==*name)?.ty?;}
        Some(ty)
    }
    fn retarget_record_callback(value:&mut Expr,family:usize){
        if !matches!(value.kind,ExprKind::Unary{op:UnaryOp::CallableStore,..}){value.wrap_storage(family);}
        value.ty=Some(Type::BorrowedCallable(family));
    }
    pub(super) fn local_record_field_contract(&self,context:&Context,target:&Expr)->Option<Type>{
        let id=Self::root_binding(target)?;let binding=&context.bindings[id];
        if binding.parameter || binding.capture_reference{return None;}
        let(_,path)=Self::scoped_array_storage_path(target)?;if path.is_empty(){return None;}
        let mut ty=binding.ty;
        for name in path{while let Type::Array(element)=ty{ty=element.ty();}let Type::Record(record)=ty else{return None;};let record=self.scoped_records.declaration(record);ty=self.program.records[record].fields.iter().find(|field|field.name.text==name)?.ty?;}
        (matches!(ty,Type::Callback(_)) || matches!(ty,Type::Map(_)) && self.aggregate_callable_facts(ty).scoped).then_some(ty)
    }
    /// A local record field declared as a borrowing callback or an absent value.
    /// Its two states have different native layouts; replacement is not planned yet.
    pub(super) fn local_record_nullable_callback(&self,context:&Context,target:&Expr)->bool{
        let Some(id)=Self::root_binding(target)else{return false;};let binding=&context.bindings[id];
        if binding.parameter || binding.capture_reference{return false;}
        let Some((_,path))=Self::direct_fields(target)else{return false;};if path.is_empty(){return false;}
        let mut ty=binding.ty;
        for name in path{
            let Type::Record(record)=ty else{return false;};let record=self.scoped_records.declaration(record);
            let Some(next)=self.program.records[record].fields.iter().find(|field|field.name.text==name).and_then(|field|field.ty)else{return false;};ty=next;
        }
        ty.union().is_some_and(|union|self.unions.members(union).into_iter().any(|member|matches!(member,Type::Callback(callback) if !self.callables.callbacks[callback].kind.is_owned())))
    }
    pub(super) fn record_type_with_field(&mut self,record:usize,path:&[String],ty:Type)->Option<usize>{
        let name=path.first()?;let index=self.program.records[record].fields.iter().position(|field|&field.name.text==name)?;
        let mut fields=self.program.records[record].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
        fields[index]=if path.len()==1{ty}else{let Type::Record(child)=fields[index]else{return None;};Type::Record(self.record_type_with_field(child,&path[1..],ty)?)};
        let id=self.scoped_records.materialize(record,fields,&mut self.program.records);
        if self.fields.len()<=id{self.fields.push(self.program.records[id].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
        Some(id)
    }
    pub(super) fn update_local_record_field_type(&mut self,context:&mut Context,target:&Expr,ty:Type)->Result<(),Diagnostic>{
        if self.local_record_field_contract(context,target).is_none(){return Ok(());}
        let id=Self::root_binding(target).unwrap();let(_,path)=Self::scoped_array_storage_path(target).unwrap();
        if let Some(root)=self.scoped_array_replace_field(context.bindings[id].ty,&path,ty){context.bindings[id].ty=root;}
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
            else if lt!=rt && let Some(ty)=self.join_scoped_map_types(lt,rt,l.span)?{
                for value in [&mut *l,&mut *r]{if value.ty!=Some(ty){value.wrap_record_carrier(ty);}}
                fields[index]=ty;changed=true;
            }
            else if lt!=rt && let Some(family)=self.common_borrowed_conditional_family(lt,rt,l.span)?{
                for value in [&mut *l,&mut *r]{if !matches!(value.kind,ExprKind::Unary{op:UnaryOp::CallableStore,..}){value.wrap_storage(family);}value.ty=Some(Type::BorrowedCallable(family));}let ty=Type::BorrowedCallable(family);l.ty=Some(ty);r.ty=Some(ty);fields[index]=ty;changed=true;
            }else if lt==rt{fields[index]=lt;}
        }
        if !changed{return Ok(None);}
        let instance=self.scoped_records.materialize(declaration,fields,&mut self.program.records);
        if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
        let ty=Type::Record(instance);left.ty=Some(ty);right.ty=Some(ty);Ok(Some(ty))
    }
    pub(super) fn join_record_types(&mut self,a:usize,b:usize,span:Span)->Result<Option<Type>,Diagnostic>{
        let declaration=self.scoped_records.declaration(a);if declaration!=self.scoped_records.declaration(b){return Ok(None);}
        let mut fields=Vec::new();let mut changed=false;
        let pairs=self.program.records[a].fields.iter().zip(&self.program.records[b].fields).map(|(a,b)|(a.ty.unwrap(),b.ty.unwrap())).collect::<Vec<_>>();
        for(left,right) in pairs{
            let joined=if left==right{left}else if let(Type::Record(a),Type::Record(b))=(left,right){self.join_record_types(a,b,span)?.unwrap_or(left)}else if let(Type::Array(ArrayElement::Record(a, depth)),Type::Array(ArrayElement::Record(b, other_depth)))=(left,right) {
                if depth != other_depth { return Ok(None); }
                let Some(Type::Record(record)) = self.join_record_types(a,b,span)? else { return Ok(None); };
                Type::Array(ArrayElement::Record(record,depth))
            }else if let Some(map)=self.join_scoped_map_types(left,right,span)?{map}else if let Some(family)=self.common_borrowed_conditional_family(left,right,span)?{Type::BorrowedCallable(family)}else{return Ok(None);};
            changed|=joined!=left || joined!=right;fields.push(joined);
        }
        if !changed{return Ok(None);}
        let instance=self.scoped_records.materialize(declaration,fields,&mut self.program.records);
        if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
        Ok(Some(Type::Record(instance)))
    }

}
