//! Cache reads retain an exclusive owner loan and expose only read-only values.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer {
 pub(super) fn cache_constructor(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::NativeConstructor(args,Some(annotation))=&mut expression.kind else{return Ok(None);};
  let ty=self.resolve_value_annotation(annotation,false)?;
  let Type::Record(record)=ty else{return Ok(None);};
  if self.program.records[record].native_type!=Some(NativeTypeId::LruCache){return Ok(None);}
  Self::arity(args.len(),1,expression.span)?;
  self.cache_payload(record,expression.span)?;
  let actual=self.expression(context,&mut args[0],Mode::Read,loans,Some(Type::Int))?;
  self.require(actual,Some(Type::Int),args[0].span)?;
  self.require(ty,expected,expression.span)?;
  expression.target=Some(CallTarget::NativeBinding(crate::native_bindings::by_symbol("@native:std:cache#__create").unwrap().0));
  Ok(Some(ty))
 }
 fn cache_payload(&self,record:usize,span:Span)->Result<(),Diagnostic>{
  let payload=self.program.records[record].application.as_ref().unwrap().resolved_arguments[1];
  crate::stdlib::cache::validate_payload(payload,&self.program.records,|id|self.unions.members(id),span)?;
  if self.native_shape_contains_view(payload)||self.aggregate_callable_facts(payload).scoped {
   return Err(Diagnostic::new("cache-payload","Cache values must own their storage; borrowed views and scoped callbacks cannot be retained.",span));
  }
  Ok(())
 }
 pub(super) fn cache_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None);};
  if let ExprKind::Field(base,name)=&callee.kind{
   let Some(Type::Record(record))=self.expression_type_hint(context,base) else{return Ok(None);};
   if self.program.records[record].native_type!=Some(NativeTypeId::LruCache){return Ok(None);}
   if !matches!(name.as_str(),"get"|"peek"|"set"|"remove"|"size"|"clear"){return Err(Diagnostic::new("cache-member","Unknown LruCache operation.",callee.span));}
   let name=format!("@native:std:cache#__{name}");args.insert(0,(**base).clone());callee.kind=ExprKind::Name(name);
  }
  let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,b)|b.module=="std:cache") else{return Ok(None);};
  if !binding.accepts_arity(args.len()){return Err(Diagnostic::new("arity","Invalid argument count for this cache operation.",expression.span));}
  let Type::Record(record)=self.expression(context,&mut args[0],Mode::Mutate,loans,None)? else{unreachable!()};
  self.cache_payload(record,expression.span)?;
  let types=self.program.records[record].application.as_ref().unwrap().resolved_arguments.clone();
  let (key,value)=(types[0],types[1]);
  let mut active=loans.clone();
  for root in Self::loan_roots(&args[0],Mode::Mutate){active.entry(root).or_default().push((Mode::Mutate,args[0].span));}
  for (index,arg) in args.iter_mut().enumerate().skip(1){
   let wanted=if index==1{key}else if index==2{value}else{Type::Int};
   let access=if binding.name=="__set"&&index<3{Mode::Consume}else{Mode::Read};
   let actual=self.expression(context,arg,access,&active,Some(wanted))?;self.require(actual,Some(wanted),arg.span)?;
   if index==2 && (self.native_shape_contains_view(actual)||self.aggregate_callable_facts(actual).scoped){return Err(Diagnostic::new("cache-payload","Cache values must own their storage.",arg.span));}
  }
  let result=match binding.name{
   "__size"=>Type::Int,"__clear"=>Type::Void,
   "__get"|"__peek"=>{
    let result=self.unions.intern([value,Type::Undefined],expression.span)?;
    if !result.is_copy(){
     if !Self::stable_at_receiver(&args[0]){return Err(Diagnostic::new("borrow","Borrowed cache reads require a stored receiver.",args[0].span));}
     for place in native_view_places::sources(context,&args[0])?{context.cache_read_owners.insert((place.root,place.fields));}
     context.native_view_calls=true;Type::BorrowedUnion(result.union().unwrap())
    }else{result}
   },
   _=>{
    let entry=self.thread_application(NativeTypeId::CacheEntry,&types,expression.span)?;
    let result=self.unions.intern([entry,Type::Undefined],expression.span)?;
    if binding.name=="__remove"{result}else{
     let error=self.thread_application(NativeTypeId::CacheInsertError,&types,expression.span)?;
     self.thread_result(result,error,expression.span)?
    }
   }
  };
  self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
}

// Acyclic local bodies have a conservative source-order last-use proof. Backedges,
// deferred callbacks and exception cleanup retain the existing lexical boundary.
pub(super) fn last_uses(body:&[Stmt])->Option<HashMap<String,usize>>{
 let mut blocked=false;
 let mut inspect=|statement:&Stmt|{blocked|=matches!(statement.kind,StmtKind::Loop{..}|StmtKind::While{..}|StmtKind::ForOf{..}|StmtKind::Try(_)|StmtKind::LocalFunction(_));};
 crate::fixed_storage::visit_statements(body,&mut inspect);
 if blocked{return None;}
 let expressions=crate::error_effects::body_expressions(body);
 let mut last=HashMap::<String,usize>::new();
 for value in expressions{
  match &value.kind{
   ExprKind::Closure{..}=>return None,
   ExprKind::Match{body,..}=>{crate::fixed_storage::visit_statements(body,&mut |statement|{blocked|=matches!(statement.kind,StmtKind::Loop{..}|StmtKind::While{..}|StmtKind::ForOf{..}|StmtKind::Try(_)|StmtKind::LocalFunction(_));});},
   ExprKind::Name(name)=>{last.entry(name.clone()).and_modify(|end|*end=(*end).max(value.span.end)).or_insert(value.span.end);},
   _=>{},
  }
 }
 (!blocked).then_some(last)
}
