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
        let Type::StoredCallable(family)=current else{return None;};
        self.callables.storage.scoped_native_family(family)?;
        let holder=Self::native_view_field_redeclared(context,target)?;
        let ExprKind::Field(_,name)=&target.kind else{return None;};
        let Type::Record(record)=context.bindings[holder].ty else{return None;};
        let nominal=self.scoped_records.declaration(record);
        self.program.records[nominal].fields.iter().find(|field|field.name.text==*name)?.ty.filter(|ty|matches!(ty,Type::StoredCallable(_)))
    }
    pub(super) fn update_native_view_field_type(&mut self,context:&mut Context,target:&Expr,actual:Type) {
        if !self.native_borrow_types || !self.native_shape_contains_view(actual) {return;}
        let Some(holder)=Self::native_view_field_redeclared(context,target) else{return;};
        let ExprKind::Field(_,name)=&target.kind else{return;};
        if let Some(root)=self.scoped_array_replace_field(context.bindings[holder].ty,std::slice::from_ref(name),actual) {context.bindings[holder].ty=root;}
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
