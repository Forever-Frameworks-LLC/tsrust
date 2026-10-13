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
            let ExprKind::Unary{value:inner,..}=old.into_kind() else{unreachable!()};
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
        if self.sticky_nullable(context,binding){return Ok(());}
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
                StmtKind::Let{binding:Some(binding),value,..}=>{
                    declarations.insert((*binding,value.span.start,value.span.end));
                }
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
            if self.sticky_nullable(context,write.binding) {continue;}
            let binding=&context.bindings[write.binding];
            if write.path.is_empty() {
                // Elements of a local array join through the array's carrier type.
                if write.array_element && !binding.parameter && !binding.capture_reference {continue;}
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
            // Array element fields join through the array's carrier type.
            if array && !binding.parameter && !binding.capture_reference {continue;}
            // Clearing through a borrowed record stores no environment.
            if binding.parameter && !array && matches!(&write.value.kind,ExprKind::Inject{value,..} if matches!(value.kind,ExprKind::Null|ExprKind::Undefined)) {continue;}
            if self.nullable_scoped_record_declaration(ty).is_some() && binding.parameter && binding.mode==Mode::Mutate && !array {
                // Replacement obligations and escape checks are propagated after checking all functions.
                continue;
            }
            if self.nullable_scoped_record_declaration(ty).is_some()
                && (binding.capture_reference || !direct.contains(&(write.binding,write.span.start,write.span.end))) {
                return Err(Diagnostic::new("unsupported","Nullable callback-record field replacement needs a direct local assignment in a normal lexical sequence; branch, loop, indexed and borrowed field storage joins are not supported yet.",write.span));
            }
        }
        Ok(())
    }
}

/// Names assigned inside a branch, loop, exception region or labelled block of
/// this body, and names assigned as a whole after their declaration.
pub(super) fn control_stores(body:&[Stmt]) -> (HashSet<String>,HashSet<String>) {
    let mut nested=HashSet::new();
    let mut whole=HashSet::new();
    let mut pending=body.iter().map(|statement|(statement,false)).collect::<Vec<_>>();
    while let Some((statement,inside))=pending.pop() {
        let inside=inside || !statement.labels.is_empty();
        match &statement.kind {
            StmtKind::Assign{target,operator:None,..}=>{
                let mut root:&Expr=target;
                while let ExprKind::Field(base,_)=&root.kind {root=base;}
                let ExprKind::Name(name)=&root.kind else{continue;};
                if inside {nested.insert(name.clone());}
                if matches!(target.kind,ExprKind::Name(_)) {whole.insert(name.clone());}
            }
            StmtKind::Block(body)=>pending.extend(body.iter().map(|statement|(statement,inside))),
            StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body.iter().map(|statement|(statement,true))),
            StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body).map(|statement|(statement,true))),
            StmtKind::Loop{body,update,..}=>pending.extend(body.iter().chain(update).map(|statement|(statement,true))),
            StmtKind::Try(region)=>pending.extend(region.statements().map(|statement|(statement,true))),
            _=>{}
        }
    }
    (nested,whole)
}

impl Analyzer {
    /// Recompute the control-stored locals of a checked body. Each block marks
    /// only its own declarations, exactly as the checking pass does.
    pub(super) fn mark_control_stored(&self,context:&mut Context,body:&[Stmt]) {
        let (nested,whole)=control_stores(body);
        for statement in body {
            match &statement.kind {
                StmtKind::Let{name,binding:Some(id),..}=>{
                    if nested.contains(&name.text) || whole.contains(&name.text) && !self.nullable_record_field_paths(context.bindings[*id].ty).is_empty() {context.control_stored.insert(*id);}
                }
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>self.mark_control_stored(context,body),
                StmtKind::If{then_body,else_body,..}=>{self.mark_control_stored(context,then_body);self.mark_control_stored(context,else_body);}
                StmtKind::Loop{body,update,..}=>{self.mark_control_stored(context,body);self.mark_control_stored(context,update);}
                StmtKind::Try(region)=>{
                    self.mark_control_stored(context,&region.body);
                    if let Some(catch)=&region.catch {self.mark_control_stored(context,&catch.body);}
                    if let Some(finally)=&region.finally {self.mark_control_stored(context,finally);}
                }
                _=>{}
            }
        }
    }
    /// A local whose nullable callback records are stored through control flow.
    pub(super) fn sticky_nullable(&self,context:&Context,binding:usize)->bool {
        let info=&context.bindings[binding];
        context.control_stored.contains(&binding) && !info.parameter && !info.capture_reference
            && (self.nullable_scoped_record_declaration(info.ty).is_some() || !self.nullable_record_field_paths(info.ty).is_empty())
    }
}

impl Analyzer {
    /// A nullable record annotation names its declaration. The checked result
    /// keeps the one concrete callback environment selected by its returns, and
    /// absent returns take that same tagged type.
    pub(super) fn nullable_record_result(&mut self,body:&mut [Stmt],result:Type)->Result<Type,Diagnostic> {
        let Some(declaration)=self.nullable_scoped_record_declaration(result) else{return Ok(result);};
        if !matches!(self.unions.payload(result),Some(Type::Record(record)) if record==declaration){return Ok(result);}
        let span=body.first().map_or(Span::default(),|statement|statement.span);
        let mut values=Vec::new();
        returned_values(body,&mut values);
        let mut concrete=None;
        let mut joins=false;
        for value in &values {
            let Some(ty)=value.ty else{continue;};
            if ty==result || !matches!(self.unions.payload(ty),Some(Type::Record(record)) if self.scoped_records.declaration(record)==declaration){continue;}
            concrete=Some(match concrete {
                Some(previous) if previous!=ty=>match self.join_nullable_record_unions(previous,ty,value.span)? {
                    Some(joined)=>{joins=true;joined}
                    None=>return Err(Diagnostic::new("unsupported","The returns of this function hold callbacks that cannot share one nullable callback record.",value.span)),
                },
                _=>ty,
            });
        }
        // A function that only returns absence still needs a concrete tagged type.
        let concrete=match concrete {
            Some(concrete)=>concrete,
            None=>{
                let Some(union)=result.union() else{return Ok(result);};
                let empty=self.empty_scoped_record_environment(declaration)?;
                let members=self.unions.members(union).into_iter().map(|member|if matches!(member,Type::Record(_)){empty}else{member}).collect::<Vec<_>>();
                self.unions.intern(members,span)?
            }
        };
        for value in values {
            if value.ty==Some(result) && let ExprKind::Inject{value:inner,union}=&mut value.kind && matches!(inner.kind,ExprKind::Null|ExprKind::Undefined) {
                *union=concrete.union().expect("nullable record result");
                value.ty=Some(concrete);
            } else if joins && value.ty.is_some_and(|ty|ty!=concrete && self.nullable_scoped_record_declaration(ty)==Some(declaration)) {
                self.retarget_nullable_store(value,concrete)?;
            }
        }
        Ok(concrete)
    }
}

/// Every returned value of a checked body, in nested statements too.
    fn returned_values<'a>(body:&'a mut [Stmt],found:&mut Vec<&'a mut Expr>) {
        for statement in body {
            match &mut statement.kind {
                StmtKind::Return(Some(value))=>found.push(value),
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>returned_values(body,found),
                StmtKind::If{then_body,else_body,..}=>{returned_values(then_body,found);returned_values(else_body,found);}
                StmtKind::Loop{body,update,..}=>{returned_values(body,found);returned_values(update,found);}
                StmtKind::Try(region)=>{
                    returned_values(&mut region.body,found);
                    if let Some(catch)=&mut region.catch {returned_values(&mut catch.body,found);}
                    if let Some(finally)=&mut region.finally {returned_values(finally,found);}
                }
                _=>{}
            }
        }
    }

impl Analyzer {
    /// A record result whose returns differ only in callback environments
    /// takes their join. Each returned value is converted to that join.
    pub(super) fn joined_record_result(&mut self,body:&mut [Stmt],base:usize)->Result<Option<Type>,Diagnostic> {
        let mut values=Vec::new();
        returned_values(body,&mut values);
        let mut joined:Option<usize>=None;
        let mut differs=false;
        for value in &values {
            let Some(Type::Record(instance))=value.ty else{return Ok(None);};
            if self.scoped_records.declaration(instance)!=base{return Ok(None);}
            let Some(previous)=joined else{joined=Some(instance);continue;};
            if previous==instance{continue;}
            differs=true;
            match self.join_sticky_record_types(previous,instance,value.span)? {
                Some(Type::Record(next))=>joined=Some(next),
                _=>return Err(Diagnostic::new("unsupported","The returns of this function hold callbacks that cannot share one callback record.",value.span)),
            }
        }
        let Some(joined)=joined.filter(|_|differs) else{return Ok(None);};
        for value in values {self.retarget_nullable_store(value,Type::Record(joined))?;}
        Ok(Some(Type::Record(joined)))
    }
}

impl Analyzer {
    /// Conditional branches that hold callback records of one declaration, or
    /// a callback record and absence, take one joined environment.
    pub(super) fn conditional_record_join(&mut self,consequent:&mut Expr,alternate:&mut Expr,left:Type,right:Type)->Result<Option<Type>,Diagnostic> {
        fn absent(value:&Expr)->bool {matches!(&value.kind,ExprKind::Inject{value,..} if matches!(value.kind,ExprKind::Null|ExprKind::Undefined))}
        let joined=match(left,right){
            (Type::Record(a),Type::Record(b)) if self.scoped_records.declaration(a)==self.scoped_records.declaration(b)
                && self.aggregate_callable_facts(Type::Record(self.scoped_records.declaration(a))).scoped=>self.join_sticky_record_types(a,b,consequent.span)?,
            _ if self.nullable_scoped_record_declaration(left).is_some() && self.nullable_scoped_record_declaration(left)==self.nullable_scoped_record_declaration(right)=>{
                if absent(alternate){Some(left)}else if absent(consequent){Some(right)}else{self.join_nullable_record_unions(left,right,consequent.span)?}
            }
            _=>None,
        };
        let Some(joined)=joined else{return Ok(None);};
        self.retarget_nullable_store(consequent,joined)?;
        self.retarget_nullable_store(alternate,joined)?;
        Ok((consequent.ty==Some(joined) && alternate.ty==Some(joined)).then_some(joined))
    }
}

impl Analyzer {
    /// The type stored at a field path that can pass through array elements.
    fn nullable_slot_type(&self,mut ty:Type,path:&[String])->Option<Type> {
        let mut path=path;
        loop {
            while let Type::Array(element)=ty {ty=element.ty();}
            let Some((name,rest))=path.split_first() else{return Some(ty);};
            let Type::Record(record)=ty else{return None;};
            ty=self.program.records[record].fields.iter().find(|field|field.name.text==*name)?.ty?;
            path=rest;
        }
    }
    fn with_nullable_slot(&mut self,ty:Type,path:&[String],slot:Type)->Option<Type> {
        match ty {
            Type::Array(ArrayElement::Union(_,depth)) if path.is_empty()=>Some(Type::Array(ArrayElement::Union(slot.union()?,depth))),
            Type::Array(ArrayElement::Record(record,depth))=>{
                let Type::Record(record)=self.with_nullable_slot(Type::Record(record),path,slot)? else{return None;};
                Some(Type::Array(ArrayElement::Record(record,depth)))
            }
            Type::Record(record)=>{
                let (name,rest)=path.split_first()?;
                let current=self.program.records[record].fields.iter().find(|field|field.name.text==*name)?.ty?;
                let field=if rest.is_empty(){
                    // The slot is the field itself, or each element of an array field.
                    if let Type::Array(ArrayElement::Union(_,depth))=current {Type::Array(ArrayElement::Union(slot.union()?,depth))} else {slot}
                }else{self.with_nullable_slot(current,rest,slot)?};
                self.record_type_with_field(record,std::slice::from_ref(name),field).map(Type::Record)
            }
            _=>None,
        }
    }
    /// Nullable callback-record fields of array elements share one carrier
    /// type per array: every indexed store joins into the element type.
    pub(super) fn nullable_array_slot_carriers(&mut self,context:&mut Context,body:&mut [Stmt])->Result<(),Diagnostic> {
        let mut plans=HashMap::<usize,Type>::new();
        let mut stores=Vec::new();
        for write in record_storage_writes::collect(body) {
            let binding=&context.bindings[write.binding];
            if binding.parameter || binding.capture_reference{continue;}
            // An array whose elements are nullable callback records joins every stored element.
            if write.path.is_empty() {
                let Type::Array(ArrayElement::Union(element,_))=binding.ty else{continue;};
                if self.nullable_scoped_record_declaration(Type::Union(element)).is_none(){continue;}
                let mut values=vec![write.value];
                while let Some(value)=values.pop() {
                    match &value.kind {
                        ExprKind::Array(elements) if !write.array_element=>values.extend(elements),
                        ExprKind::Inject{value:inner,..}=>{
                            let incoming=match inner.ty {
                                Some(payload @ Type::Record(_))=>{
                                    let members=self.unions.members(element).into_iter().map(|member|if matches!(member,Type::Record(_)){payload}else{member}).collect::<Vec<_>>();
                                    Some(self.unions.intern(members,write.span)?)
                                }
                                _=>None,
                            };
                            stores.push((write.binding,Vec::new(),incoming,write.span));
                        }
                        _=>{}
                    }
                }
                continue;
            }
            let through_array=write.array_element || matches!(binding.ty,Type::Array(_)) || {
                let mut ty=binding.ty;let mut array=false;
                for name in &write.path {
                    while let Type::Array(element)=ty {ty=element.ty();array=true;}
                    let Type::Record(record)=ty else{break;};
                    let Some(next)=self.program.records[record].fields.iter().find(|field|field.name.text==*name).and_then(|field|field.ty) else{break;};
                    ty=next;
                }
                array
            };
            if !through_array{continue;}
            let Some(slot)=self.nullable_slot_type(binding.ty,&write.path) else{continue;};
            if self.nullable_scoped_record_declaration(slot).is_none(){continue;}
            // An indexed store is checked against the declaration; its payload names the concrete environment.
            let incoming=match &write.value.kind {
                ExprKind::Inject{value:inner,..}=>match inner.ty {
                    Some(payload @ Type::Record(_))=>{
                        let members=self.unions.members(slot.union().expect("nullable slot")).into_iter().map(|member|if matches!(member,Type::Record(_)){payload}else{member}).collect::<Vec<_>>();
                        Some(self.unions.intern(members,write.span)?)
                    }
                    _=>None,
                },
                _=>write.value.ty,
            };
            stores.push((write.binding,write.path.clone(),incoming,write.span));
        }
        // Declarations of nullable-element arrays are not storage writes of a
        // record type; collect their initial elements here.
        let mut declarations=Vec::new();
        let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop() {
            match &statement.kind {
                StmtKind::Let{binding:Some(id),value,..}=>{
                    let Type::Array(ArrayElement::Union(element,_))=context.bindings[*id].ty else{continue;};
                    if self.nullable_scoped_record_declaration(Type::Union(element)).is_none(){continue;}
                    declarations.push((*id,value.span));
                    let mut values=vec![value];
                    while let Some(value)=values.pop() {
                        match &value.kind {
                            ExprKind::Array(elements)=>values.extend(elements),
                            ExprKind::Inject{value:inner,..}=>if let Some(payload @ Type::Record(_))=inner.ty {
                                let members=self.unions.members(element).into_iter().map(|member|if matches!(member,Type::Record(_)){payload}else{member}).collect::<Vec<_>>();
                                stores.push((*id,Vec::new(),Some(self.unions.intern(members,value.span)?),value.span));
                            },
                            _=>{}
                        }
                    }
                }
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);}
                StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);}
                StmtKind::Try(region)=>pending.extend(region.statements()),
                _=>{}
            }
        }
        if stores.is_empty(){return Ok(());}
        loop {
            let mut changed=false;
            for (binding,path,incoming,span) in &stores {
                let plan=*plans.entry(*binding).or_insert(context.bindings[*binding].ty);
                let Some(slot)=self.nullable_slot_type(plan,path) else{continue;};
                let Some(incoming)=incoming else{continue;};
                let Some(joined)=self.join_nullable_record_unions(slot,*incoming,*span)? else{continue;};
                if joined==slot{continue;}
                if let Some(next)=self.with_nullable_slot(plan,path,joined) && next!=plan {plans.insert(*binding,next);changed=true;}
            }
            if !changed{break;}
        }
        for (&binding,&plan) in &plans {context.bindings[binding].ty=plan;}
        let roots=plans.keys().copied().collect::<HashSet<_>>();
        // A for-of variable over a planned array reads the carrier element type.
        let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop() {
            match &statement.kind {
                StmtKind::ForOf{binding:Some(id),iterable,body,..}=>{
                    if let Some(Type::Array(element))=self.planned_record_array_type(iterable,&plans) {
                        plans.insert(*id,element.ty());context.bindings[*id].ty=element.ty();
                    }
                    pending.extend(body);
                }
                // A view of a planned element reads the carrier union.
                StmtKind::Let{binding:Some(id),value,..} if matches!(value.kind,ExprKind::Index(_,_)|ExprKind::Field(_,_)|ExprKind::Call(_,_)) && !plans.contains_key(id)=>{
                    if let Some(planned)=self.planned_record_array_type(value,&plans) && let Some(union)=planned.union() {
                        let ty=if matches!(context.bindings[*id].ty,Type::BorrowedUnion(_)){Type::BorrowedUnion(union)}else{planned};
                        if context.bindings[*id].ty.union().is_some() {plans.insert(*id,ty);context.bindings[*id].ty=ty;}
                    }
                }
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);}
                StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);}
                StmtKind::Try(region)=>pending.extend(region.statements()),
                _=>{}
            }
        }
        let mut destinations=stores.iter().filter(|(_,path,_,_)|!path.is_empty()).filter_map(|(binding,path,_,span)|Some(((span.start,span.end),self.nullable_slot_type(*plans.get(binding)?,path)?))).collect::<HashMap<_,_>>();
        for (binding,span) in declarations {
            if let Some(&plan)=plans.get(&binding) {destinations.entry((span.start,span.end)).or_insert(plan);}
        }
        // Whole values stored into a planned root take the carrier type in place.
        for write in record_storage_writes::collect(body) {
            let Some(&plan)=plans.get(&write.binding) else{continue;};
            if !write.path.is_empty() || !roots.contains(&write.binding){continue;}
            let destination=if write.array_element {match plan {Type::Array(element)=>element.ty(),other=>other}} else {plan};
            destinations.entry((write.span.start,write.span.end)).or_insert(destination);
        }
        let mut failure=None;
        crate::source_walk::body(body,&mut |_|Ok(()),&mut |value|{
            if let Some(destination)=destinations.remove(&(value.span.start,value.span.end)) && failure.is_none()
                && let Err(error)=self.retarget_nullable_store(value,destination) {failure=Some(error);}
            self.retype_carrier_read(value,&plans);
            Ok(())
        })?;
        failure.map_or(Ok(()),Err)
    }
}

impl Analyzer {
    /// A call through a callback field that carrier planning retyped uses the
    /// planned environment, not the one seen when the call was checked.
    pub(super) fn retarget_planned_callback_call(&self,value:&mut Expr,plans:&HashMap<usize,Type>) {
        let ExprKind::Call(callee,_)=&mut value.kind else{return;};
        let planned=self.planned_record_array_type(callee,plans);
        Self::retarget_callback_call(value,planned);
    }
    fn retarget_callback_call(value:&mut Expr,planned:Option<Type>) {
        let ExprKind::Call(callee,_)=&mut value.kind else{return;};
        if !matches!(value.target,Some(CallTarget::ConditionalCallback(_)|CallTarget::ConditionalClosure(_)|CallTarget::StoredCallableField(_))){return;}
        match planned {
            Some(ty @ Type::BorrowedCallable(family))=>{callee.ty=Some(ty);value.target=Some(CallTarget::StoredCallableField(family));}
            Some(Type::Closure(origin))=>{callee.ty=Some(Type::Closure(origin));value.target=Some(CallTarget::ConditionalClosure(origin));}
            _=>{}
        }
    }
    /// Give one expression that reads carrier-planned storage the planned
    /// type: names, fields, indexed elements, narrowed payloads, optional
    /// chains and calls through callback fields.
    pub(super) fn retype_carrier_read(&self,value:&mut Expr,plans:&HashMap<usize,Type>) {
        self.retarget_planned_callback_call(value,plans);
        // An optional chain selects through the carrier union of its base.
        if let ExprKind::Optional{base,body,slot,union,..}=&mut value.kind
            && let Some(carrier)=self.planned_record_array_type(base,plans) && let Some(id)=carrier.union()
        {
            *union=Some(id);
            let slot=*slot;
            let Some(payload)=self.unions.payload(carrier) else{return;};
            // Types below the selected payload follow its record fields.
            fn selected(analyzer:&Analyzer,node:&mut Expr,slot:usize,payload:Type)->Option<Type> {
                let ty=match &mut node.kind {
                    ExprKind::Selected{slot:found,..} if *found==slot=>Some(payload),
                    ExprKind::Field(base,name)=>{
                        let Type::Record(record)=selected(analyzer,base,slot,payload)? else{return None;};
                        analyzer.program.records[record].fields.iter().find(|field|field.name.text==*name)?.ty
                    }
                    ExprKind::Call(callee,_)=>{
                        let planned=selected(analyzer,callee,slot,payload);
                        Analyzer::retarget_callback_call(node,planned);
                        return None;
                    }
                    _=>None,
                }?;
                node.ty=Some(ty);
                Some(ty)
            }
            selected(self,body,slot,payload);
        }
        let removal=matches!(value.kind,ExprKind::Call(_,_)) && matches!(value.target,Some(CallTarget::ArrayPop|CallTarget::ArrayShift));
        if (removal || matches!(value.kind,ExprKind::Name(_)|ExprKind::Field(_,_)|ExprKind::Index(_,_)|ExprKind::Project{..}))
            && let Some(ty)=self.planned_record_array_type(value,plans) && value.ty.is_some_and(|checked|checked!=ty && checked.union().is_some()==ty.union().is_some())
        {
            if let ExprKind::Project{value:inner,union,narrowed}=&mut value.kind {
                if let Some(carrier)=self.planned_record_array_type(inner,plans).and_then(|ty|ty.union()) {*union=carrier;}
                *narrowed=ty;
            }
            value.ty=Some(ty);
        }
    }
}

impl Analyzer {
    /// Whether a value can hold a callback that borrows local storage. A
    /// callback family whose environments own every capture does not.
    pub(super) fn borrows_local_storage(&self,ty:Type)->bool {
        let mut pending=vec![ty];
        let mut seen=HashSet::new();
        while let Some(ty)=pending.pop() {
            if !seen.insert(ty){continue;}
            match ty {
                Type::Closure(id)=>{
                    let closure=&self.closures[id];
                    if self.native_borrow_types {
                        if closure.captures.iter().any(|capture|matches!(capture.storage,Mode::Read|Mode::Mutate)) {return true;}
                        pending.extend(closure.captures.iter().map(|capture|closure.function.bindings[capture.inner].ty));
                    } else if closure.borrowed || closure.lifetime_count>0 {return true;}
                },
                Type::BorrowedCallable(family)=>match self.callables.storage.origins(family) {
                    Some(origins)=>if origins.iter().any(|&id|self.closures[id].borrowed || self.closures[id].lifetime_count>0) {return true;},
                    None=>return true,
                },
                Type::Callback(_)=>if !self.absent_callable_type(ty) {return true;},
                Type::Record(id)=>{ pending.extend(self.program.records[id].fields.iter().filter_map(|field|field.ty)); pending.extend(self.program.records[id].native_element()); },
                Type::Array(element)=>pending.push(element.ty()),
                Type::Map(id)=>pending.extend(self.program.records[id].map_definition.as_ref().map(|map|map.value)),
                Type::Union(id)|Type::BorrowedUnion(id)=>pending.extend(self.unions.members(id)),
                _=>{}
            }
        }
        false
    }
    /// An array result of capturing records keeps the joined element
    /// environment of its returns.
    pub(super) fn joined_array_result(&mut self,body:&mut [Stmt],base:usize,depth:u8)->Result<Option<Type>,Diagnostic> {
        if depth!=0 || self.scoped_records.declaration(base)!=base || !self.aggregate_callable_facts(Type::Record(base)).scoped {return Ok(None);}
        let mut values=Vec::new();
        returned_values(body,&mut values);
        let mut joined:Option<usize>=None;
        for value in &values {
            let Some(Type::Array(ArrayElement::Record(instance,0)))=value.ty else{return Ok(None);};
            if self.scoped_records.declaration(instance)!=base{return Ok(None);}
            if instance==base{continue;}
            joined=Some(match joined {
                Some(previous) if previous!=instance=>match self.join_sticky_record_types(previous,instance,value.span)? {
                    Some(Type::Record(next))=>next,
                    _=>return Err(Diagnostic::new("unsupported","The returns of this function hold callbacks that cannot share one array element type.",value.span)),
                },
                _=>instance,
            });
        }
        let Some(joined)=joined else{return Ok(None);};
        let result=Type::Array(ArrayElement::Record(joined,0));
        for value in values {
            if value.ty==Some(result){continue;}
            if let ExprKind::Array(elements)=&mut value.kind {
                Self::apply_scoped_array_carrier(elements,joined,0);
                value.ty=Some(result);
            } else {value.wrap_record_carrier(result);}
        }
        Ok(Some(result))
    }
}
