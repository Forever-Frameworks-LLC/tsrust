//! Join concrete record callback environments without changing source ownership.
use super::*;
fn assignments(body:&mut [Stmt])->Vec<(&mut Expr,&mut Expr)>{
    let mut result=Vec::new();
    let mut work=body.iter_mut().rev().collect::<Vec<_>>();
    while let Some(stmt)=work.pop(){match &mut stmt.kind{
        StmtKind::Assign{target,value,operator:None} if matches!(target.kind,ExprKind::Name(_))=>result.push((target,value)),
        StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>work.extend(body.iter_mut().rev()),
        StmtKind::If{then_body,else_body,..}=>{work.extend(else_body.iter_mut().rev());work.extend(then_body.iter_mut().rev());},
        StmtKind::Loop{body,update,..}=>{work.extend(update.iter_mut().rev());work.extend(body.iter_mut().rev());},
        StmtKind::Try(region)=>{if let Some(finally)=&mut region.finally{work.extend(finally.iter_mut().rev());}if let Some(catch)=&mut region.catch{work.extend(catch.body.iter_mut().rev());}work.extend(region.body.iter_mut().rev());},
        _=>{}
    }}result
}

impl Analyzer {
    pub(super) fn record_branch_carriers(&mut self,context:&mut Context,left:&mut [Stmt],right:&mut [Stmt])->Result<(),Diagnostic>{
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
        let mut work=body.iter_mut().rev().collect::<Vec<_>>();
        while let Some(stmt)=work.pop(){match &mut stmt.kind{
            StmtKind::Let{binding:Some(id),value,..}=>{
                if !matches!(context.bindings[*id].ty,Type::Closure(_) | Type::BorrowedCallable(_)){continue;}
                let(Some(root),Some((_,path)))=(Self::root_binding(value),Self::direct_fields(value))else{continue;};
                if path.is_empty() || context.bindings[root].parameter || context.bindings[root].capture_reference{continue;}
                let Some(carrier @ Type::BorrowedCallable(_))=self.record_field_type(context.bindings[root].ty,&path)else{continue;};
                context.bindings[*id].ty=carrier;value.ty=Some(carrier);
            }
            StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>work.extend(body.iter_mut().rev()),
            StmtKind::If{then_body,else_body,..}=>{work.extend(else_body.iter_mut().rev());work.extend(then_body.iter_mut().rev());}
            StmtKind::Loop{body,update,..}=>{work.extend(update.iter_mut().rev());work.extend(body.iter_mut().rev());}
            StmtKind::Try(region)=>{
                if let Some(finally)=&mut region.finally{work.extend(finally.iter_mut().rev());}
                if let Some(catch)=&mut region.catch{work.extend(catch.body.iter_mut().rev());}
                work.extend(region.body.iter_mut().rev());
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
                let sticky=self.sticky_nullable(context,write.binding);
                if sticky && write.path.is_empty() && ty.union().is_some() {
                    let Some(incoming)=write.value.ty else{continue;};
                    if let Some(joined)=self.join_nullable_record_unions(ty,incoming,write.span)? && joined!=ty{plans.insert(write.binding,joined);changed=true;}
                    continue;
                }
                let Type::Record(record)=ty else{continue;};
                if write.value.ty.is_none(){continue;}
                let incoming=self.record_value_carrier_type(context,write.value,&plans)?;
                let joined=if write.path.is_empty(){
                    let Type::Record(incoming)=incoming else{continue;};
                    if sticky {self.join_sticky_record_types(record,incoming,write.span)?}
                    else {self.join_record_types(record,incoming,write.span)?}
                }else{
                    let Some(previous)=self.record_field_type(ty,&write.path)else{continue;};
                    if previous==incoming{None}
                    else if sticky && let Some(joined)=self.join_nullable_record_unions(previous,incoming,write.span)?{
                        self.record_type_with_field(record,&write.path,joined).map(Type::Record)
                    }
                    else if let Some(family)=self.common_borrowed_conditional_family(previous,incoming,write.span)?{
                        self.record_type_with_field(record,&write.path,Type::BorrowedCallable(family)).map(Type::Record)
                    }
                    else if let Some(joined)=self.join_native_view_storage(previous,incoming,write.span)?{
                        self.record_type_with_field(record,&write.path,joined).map(Type::Record)
                    }else{None}
                };
                if let Some(joined)=joined && joined!=ty{plans.insert(write.binding,joined);changed=true;}
            }
            if !changed{break;}
        }
        let mut destinations=HashMap::new();
        let mut sticky_destinations=HashSet::new();
        for write in writes {
            let Some(&plan)=plans.get(&write.binding)else{continue;};
            let Some(destination)=self.record_field_type(plan,&write.path)else{continue;};
            destinations.insert((write.span.start,write.span.end),destination);
            if self.sticky_nullable(context,write.binding) {sticky_destinations.insert((write.span.start,write.span.end));}
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
            if sticky_destinations.contains(&(value.span.start,value.span.end)) {
                return self.retarget_nullable_store(value,destination);
            }
            match(source,destination){
                (Type::Closure(_),Type::BorrowedCallable(family))=>Self::retarget_record_callback(value,family),
                (Type::Record(_),Type::Record(_))|(Type::BorrowedCallable(_),Type::BorrowedCallable(_))=>{
                    if self.distinct_owned_closures(source,destination) {
                        return Err(Diagnostic::new("unsupported","Different callbacks that own all their captures cannot share one record variable yet. Capture a local array or record, or keep each callback in its own variable.",value.span));
                    }
                    value.wrap_record_carrier(destination)
                }
                // A view callback, or its nullable union, moves into the joined storage.
                (Type::StoredCallable(_),Type::StoredCallable(_))|(Type::Union(_),Type::Union(_)) if self.native_borrow_types && self.native_shape_contains_view(destination)=>value.wrap_record_carrier(destination),
                _=>{}
            }
            Ok(())
        })?;
        // Reads of a carrier-typed nullable local use the carrier type.
        let sticky=plans.iter().filter(|(id,_)|self.sticky_nullable(context,**id)).map(|(&id,&ty)|(id,ty)).collect::<HashMap<_,_>>();
        if !sticky.is_empty() {
            crate::source_walk::body(body,&mut |_|Ok(()),&mut |value|{
                self.retype_carrier_read(value,&sticky);
                Ok(())
            })?;
        }
        self.nullable_array_slot_carriers(context,body)?;
        self.scoped_map_carriers(context,body)?;
        self.record_array_carriers(context,body)?;
        self.scoped_array_storage_epochs(context,body)?;
        self.native_view_storage_epochs(context,body)?;
        let results=context.bindings.iter().enumerate().map(|(id,binding)|(id,binding.ty)).collect();
        self.scoped_map_entry_epochs(context,body,results)?;
        self.scoped_deque_carriers(context, body)
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
        if self.nullable_scoped_record_declaration(ty).is_some() {
            let (_,path)=Self::direct_fields(target)?;
            return self.record_field_type(binding.ty,&path);
        }
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
    pub(super) fn record_type_with_field(&mut self,mut record:usize,path:&[String],mut ty:Type)->Option<usize>{
        if path.is_empty(){return None;}
        let mut parents=Vec::new();
        for (offset,name) in path.iter().enumerate(){
            let index=self.program.records[record].fields.iter().position(|field|&field.name.text==name)?;
            let fields=self.program.records[record].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
            let child=fields[index];
            parents.push((record,index,fields));
            if offset+1<path.len(){let Type::Record(child)=child else{return None;};record=child;}
        }
        // Materialize from the replaced leaf outward, preserving parent order.
        for (record,index,mut fields) in parents.into_iter().rev(){
            fields[index]=ty;
            let recursive=self.native_borrow_types && fields.iter().any(|ty| self.native_shape_contains_view(*ty))
                && crate::recursive_view_shapes::Index::new(&self.program.records,&|id|self.unions.members(id)).recursive(self.scoped_records.declaration(record));
            let id=if recursive {self.scoped_records.materialize_recursive(record,fields,&mut self.program.records,&self.unions).ok()?}
                else {self.scoped_records.materialize(record,fields,&mut self.program.records)};
            while self.fields.len()<self.program.records.len(){let id=self.fields.len();self.fields.push(self.program.records[id].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
            ty=Type::Record(id);
        }
        let Type::Record(id)=ty else{unreachable!()};Some(id)
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
            else if let Some(ty)=self.common_native_view_storage(lt,rt,l.span)? {
                for value in [&mut *l,&mut *r] {value.wrap_record_carrier(ty);}
                fields[index]=ty;changed=true;
            }
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
    /// Give a stored value the carrier type of its destination. Tagged values
    /// and record literals are retyped in place; other values are converted.
    /// Whether generated code can convert a stored value into its carrier.
    fn carrier_convertible(&self,source:Type,target:Type)->bool{
        enum Task {
            Check(Type,Type),
            Pairs(std::vec::IntoIter<(Type,Type)>,bool),
            AfterPair(std::vec::IntoIter<(Type,Type)>,bool),
            Members(std::vec::IntoIter<Type>,std::rc::Rc<Vec<Type>>),
            AfterMember(std::vec::IntoIter<Type>,std::rc::Rc<Vec<Type>>),
        }
        let mut work=vec![Task::Check(source,target)];
        let mut result=true;
        while let Some(task)=work.pop() {
            match task {
                Task::Pairs(mut pairs,all)=>{
                    if let Some((source,target))=pairs.next(){
                        work.push(Task::AfterPair(pairs,all));
                        work.push(Task::Check(source,target));
                    }else{result=all;}
                }
                Task::AfterPair(pairs,all)=>{
                    if result==all{work.push(Task::Pairs(pairs,all));}
                }
                Task::Members(mut members,targets)=>{
                    if let Some(member)=members.next(){
                        // Union alternatives only admit identity or record conversion.
                        let pairs=targets.iter().copied().filter(|&candidate|candidate==member || matches!((member,candidate),(Type::Record(_),Type::Record(_)))).map(|candidate|(member,candidate)).collect::<Vec<_>>();
                        work.push(Task::AfterMember(members,targets));
                        work.push(Task::Pairs(pairs.into_iter(),false));
                    }else{result=true;}
                }
                Task::AfterMember(members,targets)=>{
                    if result{work.push(Task::Members(members,targets));}
                }
                Task::Check(source,target)=>{
                    if source==target{result=true;continue;}
                    result=match(source,target){
                        (Type::StoredCallable(source),Type::StoredCallable(target))=>{
                            match (self.callables.storage.scoped_native_family(source),self.callables.storage.scoped_native_family(target)) {
                                (Some(source),Some(target))=>source.signature==target.signature && source.alternatives.iter().all(|alternative|target.alternatives.contains(alternative)),
                                _=>false,
                            }
                        }
                        (Type::Array(source),Type::Array(target))=>{
                            work.push(Task::Check(source.ty(),target.ty()));continue;
                        }
                        (Type::Record(source),Type::Record(target))=>{
                            let pairs=self.program.records[source].fields.iter().zip(&self.program.records[target].fields).map(|(source,target)|(source.ty.unwrap(),target.ty.unwrap())).collect::<Vec<_>>();
                            work.push(Task::Pairs(pairs.into_iter(),true));continue;
                        }
                        (Type::Closure(origin),Type::BorrowedCallable(family))=>self.callables.storage.origins(family).is_some_and(|origins|origins.contains(&origin)),
                        (Type::BorrowedCallable(source),Type::BorrowedCallable(target))=>match(self.callables.storage.origins(source),self.callables.storage.origins(target)){(Some(source),Some(target))=>source.iter().all(|origin|target.contains(origin)),_=>false},
                        (Type::Callback(id),_)=>self.absent_callback_payloads.values().any(|&empty|empty==id),
                        (Type::Union(source),Type::Union(target))=>{
                            work.push(Task::Members(self.unions.members(source).into_iter(),std::rc::Rc::new(self.unions.members(target))));continue;
                        }
                        _=>false,
                    };
                }
            }
        }
        result
    }
    /// A conversion that would turn one concrete closure into another.
    fn distinct_owned_closures(&self,source:Type,target:Type)->bool{
        let mut work=vec![(source,target)];
        while let Some((source,target))=work.pop() {
            match(source,target){
                (Type::Closure(source),Type::Closure(target)) if source!=target=>return true,
                (Type::Record(source),Type::Record(target))=>{
                    work.extend(self.program.records[source].fields.iter().zip(&self.program.records[target].fields).rev().map(|(source,target)|(source.ty.unwrap(),target.ty.unwrap())));
                }
                _=>{}
            }
        }
        false
    }
    fn checked_carrier(&self,value:&mut Expr,destination:Type)->Result<(),Diagnostic>{
        if value.ty.is_some_and(|source|!self.carrier_convertible(source,destination)) {
            return Err(Diagnostic::new("unsupported","Different callbacks that own all their captures cannot share one record variable yet. Capture a local array or record, or keep each callback in its own variable.",value.span));
        }
        value.wrap_record_carrier(destination);
        Ok(())
    }
    pub(super) fn retarget_nullable_store(&mut self,value:&mut Expr,destination:Type)->Result<(),Diagnostic>{
        if value.ty==Some(destination){return Ok(());}
        match &mut value.kind {
            ExprKind::Inject{value:inner,union:tag} if destination.union().is_some()=>{
                if let(Some(Type::Record(_)),Some(payload))=(inner.ty,self.unions.payload(destination)){self.retarget_nullable_store(inner,payload)?;}
                *tag=destination.union().unwrap();value.ty=Some(destination);
            }
            ExprKind::Array(elements) if matches!(destination,Type::Array(_))=>{
                let Type::Array(element)=destination else{unreachable!()};
                for item in elements.iter_mut(){self.retarget_nullable_store(item,element.ty())?;}
                value.ty=Some(destination);
            }
            ExprKind::Record(values)=>{
                let(Type::Record(target),Some(Type::Record(mut current)))=(destination,value.ty)else{return self.checked_carrier(value,destination);};
                for(name,field)in values.iter_mut(){
                    let Some(wanted)=self.program.records[target].fields.iter().find(|candidate|candidate.name.text==name.text).and_then(|candidate|candidate.ty)else{continue;};
                    if field.ty==Some(wanted) || !(wanted.union().is_some() || matches!(wanted,Type::Record(_)|Type::Array(_))){continue;}
                    self.retarget_nullable_store(field,wanted)?;
                    if field.ty==Some(wanted) && let Some(next)=self.record_type_with_field(current,std::slice::from_ref(&name.text),wanted){current=next;}
                }
                value.ty=Some(Type::Record(current));
                if current!=target{self.checked_carrier(value,destination)?;}
            }
            ExprKind::Unary{op:UnaryOp::RecordCarrier,value:inner,..}=>{self.retarget_nullable_store(inner,destination)?;if inner.ty==Some(destination){let span=value.span;let old=std::mem::replace(value,Expr::new(ExprKind::Null,span));let ExprKind::Unary{value:inner,..}=old.into_kind() else{unreachable!()};*value=*inner;}else{value.ty=Some(destination);}}
            _=>self.checked_carrier(value,destination)?,
        }
        Ok(())
    }
    /// Join two nullable callback-record unions of one declaration. The result
    /// keeps the nullish members and holds one environment for every origin.
    pub(super) fn join_nullable_record_unions(&mut self,left:Type,right:Type,span:Span)->Result<Option<Type>,Diagnostic>{
        if left==right{return Ok(None);}
        let(Some(union),Some(_))=(left.union(),right.union())else{return Ok(None);};
        let(Some(Type::Record(a)),Some(Type::Record(b)))=(self.unions.payload(left),self.unions.payload(right))else{return Ok(None);};
        if self.scoped_records.declaration(a)!=self.scoped_records.declaration(b){return Ok(None);}
        // An absent value holds no callback instance; the other environment is the join.
        // The declaration itself names no environment either.
        let declaration=self.scoped_records.declaration(a);
        let empty=self.empty_scoped_record_environment(declaration)?;
        let payload=if a==b || empty==Type::Record(b) || b==declaration{Type::Record(a)}else if empty==Type::Record(a) || a==declaration{Type::Record(b)}else{
            let Some(joined)=self.join_sticky_record_types(a,b,span)? else{return Ok(None);};
            joined
        };
        let members=self.unions.members(union).into_iter().map(|member|if matches!(member,Type::Record(_)){payload}else{member}).collect::<Vec<_>>();
        self.unions.intern(members,span).map(Some)
    }
    /// Record join that also joins nullable callback-record fields.
    pub(super) fn join_sticky_record_types(&mut self,a:usize,b:usize,span:Span)->Result<Option<Type>,Diagnostic>{
        let(mut left,mut right)=(a,b);
        for path in self.nullable_record_field_paths(Type::Record(a)) {
            let(Some(first),Some(second))=(self.record_field_type(Type::Record(left),&path),self.record_field_type(Type::Record(right),&path))else{continue;};
            let Some(joined)=self.join_nullable_record_unions(first,second,span)? else{continue;};
            left=self.record_type_with_field(left,&path,joined).expect("checked nullable field path");
            right=self.record_type_with_field(right,&path,joined).expect("checked nullable field path");
        }
        if left==right{return Ok((left!=a || right!=b).then_some(Type::Record(left)));}
        Ok(self.join_record_types(left,right,span)?.or((left!=a).then_some(Type::Record(left))))
    }


}

#[cfg(test)]
mod traversal_tests {
    use super::*;

    fn nested_record_pair(analyzer:&mut Analyzer,depth:usize,right_leaf:Type)->(Type,Type) {
        let template=analyzer.program.records[0].clone();
        let (mut left,mut right)=(Type::Int,right_leaf);
        for _ in 0..depth {
            let mut record=template.clone();record.fields[0].ty=Some(left);
            left=Type::Record(analyzer.program.records.len());analyzer.program.records.push(record);
            let mut record=template.clone();record.fields[0].ty=Some(right);
            right=Type::Record(analyzer.program.records.len());analyzer.program.records.push(record);
        }
        (left,right)
    }

    #[test]
    fn carrier_compatibility_checks_deep_record_pairs_without_recursion() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Item={value:int;};function main(){}").unwrap()).unwrap();
        let (left,right)=nested_record_pair(&mut analyzer,16384,Type::Int);
        assert!(analyzer.carrier_convertible(left,right));
        let (left,right)=nested_record_pair(&mut analyzer,16384,Type::Boolean);
        assert!(!analyzer.carrier_convertible(left,right));
    }

    #[test]
    fn carrier_union_alternatives_preserve_existential_and_universal_results() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Item={value:int;};function main(){}").unwrap()).unwrap();
        let (source,good)=nested_record_pair(&mut analyzer,2,Type::Int);
        let (_,bad)=nested_record_pair(&mut analyzer,2,Type::Boolean);
        let from=analyzer.unions.intern(vec![source,Type::Undefined],Span::default()).unwrap();
        let to=analyzer.unions.intern(vec![bad,good,Type::Undefined],Span::default()).unwrap();
        assert!(analyzer.carrier_convertible(from,to));
        let missing=analyzer.unions.intern(vec![bad,good,Type::Null],Span::default()).unwrap();
        assert!(!analyzer.carrier_convertible(from,missing));
        let wrong=analyzer.unions.intern(vec![bad,Type::Undefined],Span::default()).unwrap();
        assert!(!analyzer.carrier_convertible(from,wrong));
    }

    #[test]
    fn record_field_replacement_materializes_parents_after_children() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Leaf={value:int;};type Outer={child:Leaf;keep:int;};function main(){}").unwrap()).unwrap();
        let outer=analyzer.program.records.iter().position(|record|record.name.text=="Outer").unwrap();
        let path=vec!["child".to_owned(),"value".to_owned()];
        let updated=analyzer.record_type_with_field(outer,&path,Type::Boolean).unwrap();
        assert_eq!(analyzer.record_field_type(Type::Record(updated),&path),Some(Type::Boolean));
        assert_eq!(analyzer.record_field_type(Type::Record(outer),&path),Some(Type::Int));
        assert_eq!(analyzer.record_field_type(Type::Record(updated),&["keep".to_owned()]),Some(Type::Int));
        let count=analyzer.program.records.len();
        assert!(analyzer.record_type_with_field(outer,&["child".to_owned(),"missing".to_owned()],Type::Boolean).is_none());
        assert_eq!(analyzer.program.records.len(),count);
    }

    #[test]
    fn carrier_assignments_keep_lexical_branch_and_exception_order() {
        let mut program=crate::syntax::parse("function main(){a=1;{b=2;}if(true){c=3;}else{d=4;}try{e=5;}catch(error){f=6;}finally{g=7;}h=8;}").unwrap();
        let found=assignments(&mut program.functions[0].body);
        let names=found.iter().map(|(target,_)|match &target.kind{ExprKind::Name(name)=>name.as_str(),_=>panic!("named target")}).collect::<Vec<_>>();
        assert_eq!(names,vec!["a","b","c","d","e","f","g","h"]);
    }

    #[test]
    fn carrier_statement_walks_handle_deep_blocks_on_normal_stack() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("walk".into());
        let mut body=Vec::new();
        for _ in 0..16384 {
            body=vec![Stmt{labels:Vec::new(),control_target:None,kind:StmtKind::Block(body),span:Span::default()}];
        }
        assert!(assignments(&mut body).is_empty());
        analyzer.record_field_move_carriers(&mut context,&mut body);
        analyzer.record_branch_carriers(&mut context,&mut body,&mut []).unwrap();
        analyzer.mark_control_stored(&mut context,&body);
        assert!(context.control_stored.is_empty());
        drop(body);
    }
}
