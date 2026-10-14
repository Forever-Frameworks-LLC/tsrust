//! Checked collection construction is separate from recursive expression dispatch.
use super::*;

impl Analyzer {
    pub(super) fn iterable_element(&mut self, actual: Type) -> Option<ArrayElement> {
        let actual = actual.scalar_base();
        match actual {
            Type::Array(element) => Some(element),
            Type::String => ArrayElement::from_value_type(Type::String),
            Type::Record(id) if matches!(self.program.records[id].native_type,Some(crate::native_signatures::NativeTypeId::Deque | crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet)) => ArrayElement::from_value_type(self.program.records[id].native_element()?),
            Type::Set(id) => ArrayElement::from_value_type(self.program.records[id].set_definition.as_ref()?.key),
            Type::Map(id) => self.map_iteration_entry(id, Span::default()).ok().map(|pair|ArrayElement::Record(pair,0)),
            _ => None,
        }
    }

    pub(super) fn set_receiver_hint(&self, context: &Context, base: &Expr) -> bool {
        if matches!(self.expression_type_hint(context,base),Some(Type::Set(_))) {return true;}
        let mut root=base;
        while let ExprKind::Call(callee,_) = &root.kind {
            let ExprKind::Field(receiver,name)=&callee.kind else {break};
            if !matches!(name.as_str(),"union"|"intersection"|"difference"|"symmetricDifference"){break;}
            root=receiver;
        }
        matches!(root.kind,ExprKind::SetConstructor(_, _)) || matches!(self.expression_type_hint(context,root),Some(Type::Set(_)))
    }

    pub(super) fn set_method_receiver(&self, context: &Context, expression: &Expr) -> bool {
        matches!(&expression.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Field(base,_) if self.set_receiver_hint(context,base)))
    }

    pub(super) fn set_method_type(&self, id: usize, name: &str) -> Option<Type> {
        match name {
            "add" | "clear" | "forEach" => Some(Type::Void),
            "intersection" | "difference" => Some(Type::Set(id)),
            "has" | "delete" | "isSubsetOf" | "isSupersetOf" | "isDisjointFrom" => Some(Type::Boolean),
            "values" | "keys" => ArrayElement::from_value_type(self.program.records[id].set_definition.as_ref()?.key).map(Type::Array),
            "entries" => {
                let key=self.program.records[id].set_definition.as_ref()?.key;
                let label=format!("[{},{}]",self.type_name(key),self.type_name(key));
                self.record_names.get(&label).copied().map(|pair|Type::Array(ArrayElement::Record(pair,0)))
            },
            _ => None,
        }
    }


    #[inline(never)]
    pub(super) fn set_method_call(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let ExprKind::Call(callee,arguments) = &mut expression.kind else { unreachable!() };
        let ExprKind::Field(base,name) = &mut callee.kind else { unreachable!() };
        if matches!(name.as_str(),"union"|"intersection"|"difference"|"symmetricDifference") {
            return self.set_algebra_call(context,expression,loans,expected);
        }
        if matches!(name.as_str(),"isSubsetOf"|"isSupersetOf"|"isDisjointFrom") {
            return self.set_relation_call(context,expression,loans,expected);
        }

        let Type::Set(id) = self.expression(context,base,Mode::Read,loans,None)? else { unreachable!("Set receiver hint") };
        if name=="entries" {
            Self::arity(arguments.len(),0,expression.span)?;
            let (result,pair)=self.set_entries_type(id,expression.span)?;
            self.require(result,expected,expression.span)?;
            expression.target=Some(CallTarget::SetMethod(SetMethod::Entries(pair)));
            return Ok(result);
        }
        let method = match name.as_str() {
            "add"=>SetMethod::Add,"has"=>SetMethod::Has,"delete"=>SetMethod::Delete,
            "clear"=>SetMethod::Clear,"values"|"keys"=>SetMethod::Values,
            _=>return Err(Diagnostic::new("set-member","Unsupported Set method.",callee.span)),
        };
        Self::arity(arguments.len(),if matches!(method,SetMethod::Add|SetMethod::Has|SetMethod::Delete) {1}else{0},expression.span)?;
        if matches!(method,SetMethod::Add|SetMethod::Delete|SetMethod::Clear) {
            Self::storage_access(context,base,Mode::Mutate,expression.span,loans)?;
        }
        let mut active=loans.clone();
        for root in Self::loan_roots(base,Mode::Read) { active.entry(root).or_default().push((Mode::Read,base.span)); }
        if let Some(argument)=arguments.first_mut() {
            let key=self.program.records[id].set_definition.as_ref().expect("set definition").key;
            let actual=self.expression(context,argument,if method==SetMethod::Add {Mode::Consume}else{Mode::Read},&active,Some(key))?;
            self.require(actual,Some(key),argument.span)?;
        }
        let result=self.set_method_type(id,name).expect("supported Set method");
        self.require(result,expected,expression.span)?;
        expression.target=Some(CallTarget::SetMethod(method));
        Ok(result)
    }


    #[inline(never)]
    fn set_algebra_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let ExprKind::Call(callee,arguments)=&mut expression.kind else{unreachable!()};
        let ExprKind::Field(base,name)=&mut callee.kind else{unreachable!()};
        Self::arity(arguments.len(),1,expression.span)?;
        let Type::Set(left)=self.expression(context,base,Mode::Read,loans,None)? else{unreachable!("checked Set algebra receiver")};
        let mut active=loans.clone();
        for root in Self::loan_roots(base,Mode::Read){active.entry(root).or_default().push((Mode::Read,base.span));}
        let Type::Set(right)=self.expression(context,&mut arguments[0],Mode::Read,&active,None)? else{return Err(Diagnostic::new("type","Set algebra methods require another Set.",arguments[0].span));};
        let method=match name.as_str(){"union"=>SetMethod::Union,"intersection"=>SetMethod::Intersection,"difference"=>SetMethod::Difference,_=>SetMethod::SymmetricDifference};
        let a=self.program.records[left].set_definition.as_ref().unwrap().key;
        let b=self.program.records[right].set_definition.as_ref().unwrap().key;
        let key=if matches!(method,SetMethod::Union|SetMethod::SymmetricDifference){self.unions.intern([a,b],expression.span)?}else{a};
        self.require_cloneable(key,expression.span)?;
        let key_annotation=crate::generic_inference::Types{ callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions}.annotation(key,expression.span)?;
        let mut annotation=crate::type_syntax::ValueAnnotation{span:expression.span,kind:crate::type_syntax::ValueAnnotationKind::Applied{name:crate::type_syntax::TypeName{text:"Set".into(),span:expression.span},arguments:vec![key_annotation]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
        while self.fields.len()<self.program.records.len(){self.fields.push(HashMap::new());}
        let result=self.resolve_value_annotation(&annotation,false)?;
        self.require(result,expected,expression.span)?;
        expression.target=Some(CallTarget::SetMethod(method));Ok(result)
    }

    #[inline(never)]
    fn set_relation_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let ExprKind::Call(callee,arguments)=&mut expression.kind else {unreachable!()};
        let ExprKind::Field(base,name)=&mut callee.kind else {unreachable!()};
        Self::arity(arguments.len(),1,expression.span)?;
        self.expression(context,base,Mode::Read,loans,None)?;
        let mut active=loans.clone();
        for root in Self::loan_roots(base,Mode::Read){active.entry(root).or_default().push((Mode::Read,base.span));}
        let other=self.expression(context,&mut arguments[0],Mode::Read,&active,None)?;
        if !matches!(other,Type::Set(_)){return Err(Diagnostic::new("type","Set relation methods require another Set.",arguments[0].span));}
        self.require(Type::Boolean,expected,expression.span)?;
        let method=match name.as_str(){"isSubsetOf"=>SetMethod::IsSubsetOf,"isSupersetOf"=>SetMethod::IsSupersetOf,_=>SetMethod::IsDisjointFrom};
        expression.target=Some(CallTarget::SetMethod(method));
        Ok(Type::Boolean)
    }

    #[inline(never)]
    fn set_entries_type(&mut self,id:usize,span:Span)->Result<(Type,usize),Diagnostic>{
        let key=self.program.records[id].set_definition.as_ref().unwrap().key;
        self.require_cloneable(key,span)?;
        let types=crate::generic_inference::Types{ callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        let mut annotation=crate::type_syntax::ValueAnnotation{span,kind:crate::type_syntax::ValueAnnotationKind::Tuple(vec![types.annotation(key,span)?,types.annotation(key,span)?])};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let Type::Record(pair)=self.resolve_value_annotation(&annotation,false)? else {unreachable!("Set entry tuple")};
        Ok((Type::Array(ArrayElement::Record(pair,0)),pair))
    }

}
