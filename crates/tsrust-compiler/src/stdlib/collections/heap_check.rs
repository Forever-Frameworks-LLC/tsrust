//! Heap operations use the declared scalar order and ordinary ownership checks.
use super::*;
impl Analyzer {
    pub(super) fn heap_receiver(&self, context: &Context, base: &Expr) -> Option<usize> {
        let Type::Record(id)=self.expression_type_hint(context,base)? else{return None;};
        (self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Heap)).then_some(id)
    }
    pub(super) fn heap_method_type(&self,id:usize,name:&str,span:Span)->Result<Type,Diagnostic>{
        match name {
            "push"|"clear"=>Ok(Type::Void),
            "peek"|"pop"=>self.unions.intern([self.program.records[id].native_element().unwrap(),Type::Undefined],span),
            _=>Err(Diagnostic::new("heap-member","Heap supports push, peek, pop and clear.",span)),
        }
    }
    pub(super) fn heap_method_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let ExprKind::Call(callee,arguments)=&mut expression.kind else{unreachable!()};
        let ExprKind::Field(base,name)=&mut callee.kind else{unreachable!()};
        let method=match name.as_str(){"push"=>HeapMethod::Push,"peek"=>HeapMethod::Peek,"pop"=>HeapMethod::Pop,"clear"=>HeapMethod::Clear,_=>return Err(Diagnostic::new("heap-member","Heap supports push, peek, pop and clear.",callee.span))};
        Self::arity(arguments.len(),usize::from(method==HeapMethod::Push),expression.span)?;
        let Type::Record(id)=self.expression(context,base,Mode::Read,loans,None)? else{unreachable!()};
        let read=method==HeapMethod::Peek;
        if !read{Self::storage_access(context,base,Mode::Mutate,expression.span,loans)?;}
        let mut active=loans.clone();
        for root in Self::loan_roots(base,if read{Mode::Read}else{Mode::Mutate}){active.entry(root).or_default().push((if read{Mode::Read}else{Mode::Mutate},base.span));}
        if let Some(argument)=arguments.first_mut(){let element=self.program.records[id].native_element().unwrap();let actual=self.expression(context,argument,Mode::Consume,&active,Some(element))?;self.require(actual,Some(element),argument.span)?;}
        let mut result=self.heap_method_type(id,name,expression.span)?;
        if read && !result.is_copy(){
            if !Self::stable_at_receiver(base){return Err(Diagnostic::new("borrow","Borrowed Heap reads require a stored receiver.",base.span));}
            result=Type::BorrowedUnion(result.union().unwrap());
        }
        self.require(result,expected,expression.span)?;expression.target=Some(CallTarget::HeapMethod(method));Ok(result)
    }
}
