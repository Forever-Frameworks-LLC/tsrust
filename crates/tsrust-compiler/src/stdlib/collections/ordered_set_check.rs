//! Ordered set keys use the checked scalar order and ordinary storage loans.
use super::*;
impl Analyzer {
    pub(super) fn ordered_set_receiver(&self,context:&Context,base:&Expr)->Option<usize>{
        let Type::Record(id)=self.expression_type_hint(context,base)? else{return None;};
        (self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedSet)).then_some(id)
    }
    pub(super) fn ordered_set_method_type(&self,id:usize,name:&str,span:Span)->Result<Type,Diagnostic>{
        let key=self.program.records[id].native_element().unwrap();
        match name {
            "add"|"clear"|"forEach"=>Ok(Type::Void),"has"|"delete"=>Ok(Type::Boolean),
            "values"|"keys"=>Ok(Type::Array(ArrayElement::from_value_type(key).unwrap())),
            "first"|"last"=>self.unions.intern([key,Type::Undefined],span),
            "entries"=>{let label=format!("[{},{}]",self.type_name(key),self.type_name(key));self.record_names.get(&label).copied().map(|pair|Type::Array(ArrayElement::Record(pair,0))).ok_or_else(||Diagnostic::new("type","OrderedSet entry tuple needs checked receiver context.",span))},
            _=>Err(Diagnostic::new("ordered-set-member","Unsupported OrderedSet method.",span))
        }
    }
    pub(super) fn ordered_set_method_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let ExprKind::Call(callee,arguments)=&mut expression.kind else{unreachable!()};
        let ExprKind::Field(base,name)=&mut callee.kind else{unreachable!()};
        let Type::Record(id)=self.expression(context,base,Mode::Read,loans,None)? else{unreachable!()};
        let key=self.program.records[id].native_element().unwrap();
        let method=match name.as_str(){"add"=>OrderedSetMethod::Add,"has"=>OrderedSetMethod::Has,"delete"=>OrderedSetMethod::Delete,"clear"=>OrderedSetMethod::Clear,"values"|"keys"=>OrderedSetMethod::Values,"first"=>OrderedSetMethod::First,"last"=>OrderedSetMethod::Last,
            "entries"=>{Self::arity(arguments.len(),0,expression.span)?;self.require_cloneable(key,base.span)?;let(result,pair)=self.set_entries_type(key,expression.span)?;self.require(result,expected,expression.span)?;expression.target=Some(CallTarget::OrderedSetMethod(OrderedSetMethod::Entries(pair)));return Ok(result);},
            _=>return Err(Diagnostic::new("ordered-set-member","Unsupported OrderedSet method.",callee.span))};
        Self::arity(arguments.len(),usize::from(matches!(method,OrderedSetMethod::Add|OrderedSetMethod::Has|OrderedSetMethod::Delete)),expression.span)?;
        let access=if matches!(method,OrderedSetMethod::Add|OrderedSetMethod::Delete|OrderedSetMethod::Clear){Mode::Mutate}else{Mode::Read};
        Self::storage_access(context,base,access,expression.span,loans)?;
        let mut active=loans.clone();for root in Self::loan_roots(base,access){active.entry(root).or_default().push((access,base.span));}
        if let Some(argument)=arguments.first_mut(){let actual=self.expression(context,argument,if method==OrderedSetMethod::Add{Mode::Consume}else{Mode::Read},&active,Some(key))?;self.require(actual,Some(key),argument.span)?;}
        let mut result=self.ordered_set_method_type(id,name,expression.span)?;
        if method==OrderedSetMethod::Values{self.require_cloneable(key,base.span)?;}
        if matches!(method,OrderedSetMethod::First|OrderedSetMethod::Last) && !result.is_copy(){
            if !Self::stable_at_receiver(base){return Err(Diagnostic::new("borrow","Borrowed OrderedSet endpoints require a stored receiver.",base.span));}
            result=Type::BorrowedUnion(result.union().unwrap());
        }
        self.require(result,expected,expression.span)?;expression.target=Some(CallTarget::OrderedSetMethod(method));Ok(result)
    }
}
