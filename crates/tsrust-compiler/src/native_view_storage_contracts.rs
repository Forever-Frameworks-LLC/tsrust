//! Structural destinations retain the concrete lifetime-bearing storage type.
use super::*;

impl Analyzer {
    pub(super) fn scoped_native_storage_contract(&self, actual:Type, expected:Type, span:Span)->Result<bool,Diagnostic> {
        if !self.native_borrow_types || actual==expected {return Ok(false);}
        match (actual,expected) {
            (Type::StoredCallable(source),Type::StoredCallable(target))
                if self.callables.storage.scoped_native_family(source).is_some()
                    && self.callables.storage.scoped_native_family(target).is_none() => {
                let declaration=self.callables.storage.declaration(target).expect("checked storage declaration");
                self.require(actual,Some(Type::Callback(declaration)),span)?;
                Ok(true)
            }
            (Type::Array(source),Type::Array(target)) if source.depth()==target.depth()=>
                self.scoped_native_storage_contract(source.with_depth(0).ty(),target.with_depth(0).ty(),span),
            (Type::Union(source),Type::Union(target))=>{
                let (Some(actual),Some(expected))=(self.unions.payload(Type::Union(source)),self.unions.payload(Type::Union(target))) else{return Ok(false);};
                if self.unions.members(source).into_iter().filter(|ty|matches!(ty,Type::Null|Type::Undefined)).eq(self.unions.members(target).into_iter().filter(|ty|matches!(ty,Type::Null|Type::Undefined))) {
                    self.scoped_native_storage_contract(actual,expected,span)
                } else {Ok(false)}
            }
            _=>Ok(false),
        }
    }

    /// The written contract of a scoped callback field whose native storage is
    /// redeclared by this replacement. The new value keeps its own concrete family.
    pub(super) fn native_view_field_contract(&self,context:&Context,target:&Expr,current:Type)->Option<Type> {
        if !self.native_borrow_types {return None;}
        // The field holds a scoped family directly or as a nullable payload.
        let Type::StoredCallable(family)=self.unions.payload(current).unwrap_or(current) else{return None;};
        self.callables.storage.scoped_native_family(family)?;
        let holder=Self::native_view_local_field(context,target)?;
        let ExprKind::Field(_,name)=&target.kind else{return None;};
        let Type::Record(record)=context.bindings[holder].ty else{return None;};
        let nominal=self.scoped_records.declaration(record);
        let written=self.program.records[nominal].fields.iter().find(|field|field.name.text==*name)?.ty?;
        (matches!(written,Type::StoredCallable(_))==matches!(current,Type::StoredCallable(_)) && matches!(self.unions.payload(written).unwrap_or(written),Type::StoredCallable(_))).then_some(written)
    }
    pub(super) fn update_native_view_field_type(&mut self,context:&mut Context,target:&Expr,actual:Type) {
        if !self.native_borrow_types || !self.native_shape_contains_view(actual) {return;}
        let Some(holder)=Self::native_view_local_field(context,target) else{return;};
        let ExprKind::Field(_,name)=&target.kind else{return;};
        // The field keeps one native variable, whose type joins every value stored in it.
        let current=self.record_field_type(context.bindings[holder].ty,std::slice::from_ref(name));
        let stored=current.and_then(|current|self.join_native_view_storage(current,actual,target.span).ok().flatten()).unwrap_or(actual);
        if let Some(root)=self.scoped_array_replace_field(context.bindings[holder].ty,std::slice::from_ref(name),stored) {context.bindings[holder].ty=root;}
    }
    /// A direct field of a local record that owns its storage. Returns the local.
    pub(super) fn native_view_local_field(context:&Context,target:&Expr)->Option<usize> {
        let ExprKind::Field(base,_)=&target.kind else{return None;};
        if !matches!(base.kind,ExprKind::Name(_)) {return None;}
        let holder=base.binding?;
        let binding=&context.bindings[holder];
        (!binding.parameter && !binding.capture_reference && binding.borrowed_place.is_none() && !binding.mutable_place).then_some(holder)
    }
    /// Joins two storage types that hold scoped callback families, directly or as
    /// the payload of the same nullable union.
    pub(super) fn join_native_view_storage(&mut self,left:Type,right:Type,span:Span)->Result<Option<Type>,Diagnostic> {
        if !self.native_borrow_types || left==right {return Ok(None);}
        if let Some(joined)=self.common_native_view_storage(left,right,span)? {return Ok(Some(joined));}
        match (left,right) {
            (Type::StoredCallable(a),Type::StoredCallable(b))=>{
                // A written lifetime-free family gives way to the scoped one.
                let scoped=|family|self.callables.storage.scoped_native_family(family).is_some();
                Ok(match (scoped(a),scoped(b)) {(false,true) if self.callables.storage.declaration(a)==self.callables.storage.declaration(b)=>Some(right),(true,false) if self.callables.storage.declaration(a)==self.callables.storage.declaration(b)=>Some(left),_=>None})
            }
            (Type::Union(a),Type::Union(b))=>{
                let (Some(first),Some(second))=(self.unions.payload(left),self.unions.payload(right)) else{return Ok(None);};
                let absent=|members:Vec<Type>|members.into_iter().filter(|ty|matches!(ty,Type::Null|Type::Undefined)).collect::<Vec<_>>();
                if absent(self.unions.members(a))!=absent(self.unions.members(b)) {return Ok(None);}
                let Some(payload)=(if first==second {None} else {self.join_native_view_storage(first,second,span)?}) else{return Ok(None);};
                let members=self.unions.members(a).into_iter().map(|member|if member==first {payload} else {member}).collect::<Vec<_>>();
                self.unions.intern(members,span).map(Some)
            }
            _=>Ok(None),
        }
    }
    /// An array of stored callbacks takes the scoped family of the callbacks appended
    /// to it. Different alternatives join in one family; the epoch pass converts the
    /// earlier values to that family.
    pub(super) fn widen_native_view_array_storage(&mut self,context:&mut Context,base:&Expr,incoming:Type,span:Span)->Result<(),Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        let Type::StoredCallable(source)=incoming else{return Ok(());};
        if self.callables.storage.scoped_native_family(source).is_none() {return Ok(());}
        let Some((binding,path))=Self::scoped_array_storage_path(base) else{return Ok(());};
        if context.bindings[binding].parameter || context.bindings[binding].capture_reference {return Ok(());}
        let Some(Type::Array(ArrayElement::StoredCallable(previous,depth)))=self.scoped_array_field_type(context.bindings[binding].ty,&path) else{return Ok(());};
        if previous==source {return Ok(());}
        let element=if self.callables.storage.scoped_native_family(previous).is_some() {
            match self.common_native_view_storage(Type::StoredCallable(previous),incoming,span)? {
                Some(Type::StoredCallable(joined))=>joined,
                _=>return Err(Diagnostic::new("type","Array callbacks that hold byte views need the same callback contract.",span)),
            }
        } else {source};
        let ty=Type::Array(ArrayElement::StoredCallable(element,depth));
        if let Some(root)=self.scoped_array_replace_field(context.bindings[binding].ty,&path,ty) {context.bindings[binding].ty=root;}
        Ok(())
    }
    /// The lifetime-free family written for the same callback declaration.
    fn written_family(&self,scoped:usize)->Option<usize> {
        let declaration=self.callables.storage.declaration(scoped)?;
        self.callables.storage.families().map(|(id,owner,_)|(id,owner)).find(|&(id,owner)|owner==declaration && id!=scoped && !self.callables.storage.is_borrowed(id) && self.callables.storage.scoped_native_family(id).is_none()).map(|(id,_)|id)
    }
    pub(super) fn native_view_element_contract(&self,context:&Context,base:&Expr,current:Type,value:&Expr)->Type {
        if !self.native_borrow_types || !matches!(value.kind,ExprKind::Closure{..}|ExprKind::Conditional{..}) {return current;}
        let Type::StoredCallable(family)=current else{return current;};
        if self.callables.storage.scoped_native_family(family).is_none() {return current;}
        let Some((binding,_))=Self::scoped_array_storage_path(base) else{return current;};
        if context.bindings[binding].parameter || context.bindings[binding].capture_reference {return current;}
        self.written_family(family).map_or(current,Type::StoredCallable)
    }
    /// A later closure of an array literal is checked against the written contract.
    pub(super) fn native_view_literal_family(&self,current:usize,value:&Expr)->usize {
        if !self.native_borrow_types || !matches!(value.kind,ExprKind::Closure{..}) || self.callables.storage.scoped_native_family(current).is_none() {return current;}
        self.written_family(current).unwrap_or(current)
    }
    /// Moves each element of a literal into the family that joins all of them.
    pub(super) fn join_native_view_literal(&self,values:&mut [Expr],element:Option<ArrayElement>) {
        let Some(ArrayElement::StoredCallable(family,0))=element else{return;};
        if !self.native_borrow_types || self.callables.storage.scoped_native_family(family).is_none() {return;}
        for value in values {
            if let Some(Type::StoredCallable(source))=value.ty && source!=family && self.callables.storage.scoped_native_family(source).is_some() {
                value.wrap_record_carrier(Type::StoredCallable(family));
            }
        }
    }
    /// Joins a view callback with an absent value, or two nullable view callback
    /// unions, into one nullable storage type.
    pub(super) fn nullable_native_view_alternatives(&mut self,left:Type,right:Type,span:Span)->Result<Option<Type>,Diagnostic> {
        if !self.native_borrow_types {return Ok(None);}
        let absent=|ty|matches!(ty,Type::Null|Type::Undefined);
        let scoped=|this:&Self,ty|matches!(ty,Type::StoredCallable(family) if this.callables.storage.scoped_native_family(family).is_some());
        if (absent(left) && scoped(self,right)) || (absent(right) && scoped(self,left)) {
            return self.unions.intern([left,right],span).map(Some);
        }
        if matches!((left,right),(Type::Union(_),Type::Union(_))) && (self.native_shape_contains_view(left) || self.native_shape_contains_view(right)) {
            return self.join_native_view_storage(left,right,span);
        }
        Ok(None)
    }
    pub(super) fn common_native_view_storage(&mut self,left:Type,right:Type,span:Span)->Result<Option<Type>,Diagnostic> {
        if !self.native_borrow_types || left==right {return Ok(None);}
        let (Type::StoredCallable(source),Type::StoredCallable(target))=(left,right) else{return Ok(None);};
        let (Some(first),Some(second))=(self.callables.storage.scoped_native_family(source),self.callables.storage.scoped_native_family(target)) else{return Ok(None);};
        if first.signature!=second.signature || first.invocation!=second.invocation {return Ok(None);}
        let origins=first.alternatives.iter().chain(&second.alternatives).map(|alternative|alternative.origin).collect::<Vec<_>>();
        let declaration=self.callables.storage.declaration(source).expect("checked scoped declaration");
        self.callables.storage.intern_scoped_native(&self.callables.shapes,&self.callables.callbacks[declaration],declaration,origins,&self.closures,span)
    }

    pub(super) fn native_view_storage_injection(&self,actual:Type,expected:UnionId,span:Span)->Result<Option<Type>,Diagnostic> {
        let Some(payload)=self.unions.payload(Type::Union(expected)) else{return Ok(None);};
        if !self.scoped_native_storage_contract(actual,payload,span)? {return Ok(None);}
        self.unions.intern(self.unions.members(expected).into_iter().map(|member|if member==payload{actual}else{member}),span).map(Some)
    }
}
