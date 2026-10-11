//! Ordered set keys use the checked scalar order and ordinary storage loans.
use super::*;
impl Analyzer {
    pub(super) fn ordered_set_receiver(&self,context:&Context,base:&Expr)->Option<usize>{
        let Type::Record(id)=self.expression_type_hint(context,base)? else{return None;};
        (self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedSet)).then_some(id)
    }
    pub(super) fn ordered_set_method_type(&self,name:&str,span:Span)->Result<Type,Diagnostic>{
        match name {"add"|"clear"=>Ok(Type::Void),"has"|"delete"=>Ok(Type::Boolean),_=>Err(Diagnostic::new("ordered-set-member","OrderedSet supports add, has, delete and clear.",span))}
    }
    pub(super) fn ordered_set_size(&mut self,context:&mut Context,base:&mut Expr,name:&str,mode:Mode,loans:&Loans,span:Span)->Result<Type,Diagnostic>{
        self.expression(context,base,Mode::Read,loans,None)?;
        if name!="size"{return Err(Diagnostic::new("ordered-set-member","OrderedSet has only the size value property; call methods with parentheses.",span));}
        if mode==Mode::Mutate{return Err(Diagnostic::new("readonly","OrderedSet size is read-only.",span));}Ok(Type::Int)
    }
    pub(super) fn ordered_set_method_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let ExprKind::Call(callee,arguments)=&mut expression.kind else{unreachable!()};
        let ExprKind::Field(base,name)=&mut callee.kind else{unreachable!()};
        let method=match name.as_str(){"add"=>OrderedSetMethod::Add,"has"=>OrderedSetMethod::Has,"delete"=>OrderedSetMethod::Delete,"clear"=>OrderedSetMethod::Clear,_=>return Err(Diagnostic::new("ordered-set-member","OrderedSet supports add, has, delete and clear.",callee.span))};
        Self::arity(arguments.len(),usize::from(method!=OrderedSetMethod::Clear),expression.span)?;
        let Type::Record(id)=self.expression(context,base,Mode::Read,loans,None)? else{unreachable!()};
        let access=if method==OrderedSetMethod::Has{Mode::Read}else{Mode::Mutate};
        Self::storage_access(context,base,access,expression.span,loans)?;
        let mut active=loans.clone();for root in Self::loan_roots(base,access){active.entry(root).or_default().push((access,base.span));}
        if let Some(argument)=arguments.first_mut(){let element=self.program.records[id].native_element().unwrap();let actual=self.expression(context,argument,if method==OrderedSetMethod::Add{Mode::Consume}else{Mode::Read},&active,Some(element))?;self.require(actual,Some(element),argument.span)?;}
        let result=self.ordered_set_method_type(name,expression.span)?;self.require(result,expected,expression.span)?;expression.target=Some(CallTarget::OrderedSetMethod(method));Ok(result)
    }
}
